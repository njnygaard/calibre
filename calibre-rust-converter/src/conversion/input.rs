use anyhow::Result;
use std::path::Path;
use tracing::debug;
use crate::conversion::book::{Book, ManifestItem, SpineItem};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use zip::ZipArchive;

pub struct InputReader;

impl InputReader {
    pub fn new() -> Self {
        InputReader
    }
    
    pub async fn read_txt(&self, path: &Path) -> Result<Book> {
        debug!("Reading TXT file: {:?}", path);
        
        let content = tokio::fs::read_to_string(path).await?;
        let mut book = Book::new();
        
        // Set basic metadata
        book.metadata.title = Some(path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string());
        
        // Create manifest item for the text content
        let manifest_item = ManifestItem {
            id: "content".to_string(),
            href: "content.txt".to_string(),
            media_type: "text/plain".to_string(),
            content: Some(content.into_bytes()),
            properties: Vec::new(),
        };
        
        book.manifest.insert("content".to_string(), manifest_item);
        
        let spine_item = SpineItem {
            id: "content".to_string(),
            href: "content.txt".to_string(),
        };
        book.spine.push(spine_item);
        
        Ok(book)
    }
    
    pub async fn read_html(&self, path: &Path) -> Result<Book> {
        debug!("Reading HTML file: {:?}", path);
        
        let content = tokio::fs::read_to_string(path).await?;
        let mut book = Book::new();
        
        // Set basic metadata
        book.metadata.title = Some(path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string());
        
        // Create manifest item for the HTML content
        let manifest_item = ManifestItem {
            id: "content".to_string(),
            href: "content.html".to_string(),
            media_type: "text/html".to_string(),
            content: Some(content.into_bytes()),
            properties: Vec::new(),
        };
        
        book.manifest.insert("content".to_string(), manifest_item);
        
        let spine_item = SpineItem {
            id: "content".to_string(),
            href: "content.html".to_string(),
        };
        book.spine.push(spine_item);
        
        Ok(book)
    }
    
    pub async fn read_epub(&self, path: &Path) -> Result<Book> {
        debug!("Reading EPUB file: {:?}", path);
        
        let mut book = Book::new();
        
        // Open the EPUB file as a ZIP archive
        let file = fs::File::open(path)?;
        let mut archive = ZipArchive::new(file)?;
        
        // Find and parse container.xml
        let container_xml = self.extract_container_xml(&mut archive)?;
        let opf_path = self.parse_container_xml(&container_xml)?;
        
        // Extract and parse content.opf
        let opf_content = self.extract_opf(&mut archive, &opf_path)?;
        let (metadata, manifest, spine) = self.parse_opf(&opf_content)?;
        
        // Set metadata
        book.metadata = metadata;
        
        // Get the OPF directory for path resolution
        let opf_dir = if let Some(last_slash) = opf_path.rfind('/') {
            &opf_path[..last_slash + 1]
        } else {
            ""
        };
        
        // Extract all files from the EPUB
        for (id, item) in manifest {
            let full_path = if opf_dir.is_empty() {
                item.href.clone()
            } else {
                format!("{}{}", opf_dir, item.href)
            };
            
            if let Some(content) = self.extract_file(&mut archive, &full_path)? {
                book.manifest.insert(id, ManifestItem {
                    content: Some(content),
                    ..item
                });
            }
        }
        
        // Set spine
        book.spine = spine;
        
        Ok(book)
    }
    
    fn extract_container_xml(&self, archive: &mut ZipArchive<fs::File>) -> Result<String> {
        let mut container_file = archive.by_name("META-INF/container.xml")?;
        let mut content = String::new();
        container_file.read_to_string(&mut content)?;
        Ok(content)
    }
    
    fn parse_container_xml(&self, xml: &str) -> Result<String> {
        // Simple XML parsing to find the OPF file path
        if let Some(start) = xml.find("full-path=\"") {
            let start = start + 11;
            if let Some(end) = xml[start..].find('"') {
                let opf_path = xml[start..start + end].to_string();
                return Ok(opf_path);
            }
        }
        Err(anyhow::anyhow!("Could not find OPF path in container.xml"))
    }
    
    fn extract_opf(&self, archive: &mut ZipArchive<fs::File>, opf_path: &str) -> Result<String> {
        let mut opf_file = archive.by_name(opf_path)?;
        let mut content = String::new();
        opf_file.read_to_string(&mut content)?;
        Ok(content)
    }
    
    fn parse_opf(&self, opf_content: &str) -> Result<(crate::conversion::book::Metadata, HashMap<String, ManifestItem>, Vec<SpineItem>)> {
        use quick_xml::Reader;
        use quick_xml::events::Event;
        use std::io::Cursor;
        use tracing::debug;

        let mut metadata = crate::conversion::book::Metadata::new();
        let mut manifest = HashMap::new();
        let mut spine = Vec::new();

        let mut reader = Reader::from_reader(Cursor::new(opf_content));
        reader.trim_text(true);
        let mut buf = Vec::new();
        let mut in_metadata = false;
        let mut _in_spine = false;

        while let Ok(event) = reader.read_event_into(&mut buf) {
            match event {
                Event::Start(ref e) => {
                    let tag_bytes: Vec<u8> = e.name().as_ref().to_vec();
                    let tag = String::from_utf8_lossy(&tag_bytes).to_string();
                    let local_bytes: Vec<u8> = e.local_name().as_ref().to_vec();
                    let local = &local_bytes;
                    debug!("OPF Start tag: {} (local: {:?})", tag, String::from_utf8_lossy(local));
                    for attr in e.attributes().flatten() {
                        let key_bytes: Vec<u8> = attr.key.local_name().as_ref().to_vec();
                        let key = String::from_utf8_lossy(&key_bytes);
                        let value = attr.unescape_value().unwrap_or_default();
                        debug!("  Attribute: {} = {}", key, value);
                    }
                    if local == b"metadata" {
                        in_metadata = true;
                    } else if local == b"manifest" {
                        // Enter manifest: process all <item> tags until </manifest>
                        loop {
                            buf.clear();
                            match reader.read_event_into(&mut buf) {
                                Ok(Event::Start(ref e2)) => {
                                    let tag2_bytes: Vec<u8> = e2.name().as_ref().to_vec();
                                    let tag2 = String::from_utf8_lossy(&tag2_bytes).to_string();
                                    let local2_bytes: Vec<u8> = e2.local_name().as_ref().to_vec();
                                    let local2 = &local2_bytes;
                                    debug!("  Manifest inner Start tag: {} (local: {:?})", tag2, String::from_utf8_lossy(local2));
                                    if local2 == b"item" || tag2.ends_with(":item") {
                                        let mut id = None;
                                        let mut href = None;
                                        let mut media_type = None;
                                        for attr in e2.attributes().flatten() {
                                            match attr.key.local_name().as_ref() {
                                                b"id" => id = Some(attr.unescape_value().unwrap_or_default().to_string()),
                                                b"href" => href = Some(attr.unescape_value().unwrap_or_default().to_string()),
                                                b"media-type" => media_type = Some(attr.unescape_value().unwrap_or_default().to_string()),
                                                _ => {}
                                            }
                                        }
                                        if let (Some(id), Some(href), Some(media_type)) = (id, href, media_type) {
                                            debug!("Adding manifest item: id={}, href={}, media_type={}", id, href, media_type);
                                            manifest.insert(id.clone(), ManifestItem {
                                                id,
                                                href,
                                                media_type,
                                                content: None,
                                                properties: Vec::new(),
                                            });
                                        }
                                    }
                                }
                                Ok(Event::Empty(ref e2)) => {
                                    let tag2_bytes: Vec<u8> = e2.name().as_ref().to_vec();
                                    let tag2 = String::from_utf8_lossy(&tag2_bytes).to_string();
                                    let local2_bytes: Vec<u8> = e2.local_name().as_ref().to_vec();
                                    let local2 = &local2_bytes;
                                    debug!("  Manifest inner Empty tag: {} (local: {:?})", tag2, String::from_utf8_lossy(local2));
                                    if local2 == b"item" || tag2.ends_with(":item") {
                                        let mut id = None;
                                        let mut href = None;
                                        let mut media_type = None;
                                        for attr in e2.attributes().flatten() {
                                            match attr.key.local_name().as_ref() {
                                                b"id" => id = Some(attr.unescape_value().unwrap_or_default().to_string()),
                                                b"href" => href = Some(attr.unescape_value().unwrap_or_default().to_string()),
                                                b"media-type" => media_type = Some(attr.unescape_value().unwrap_or_default().to_string()),
                                                _ => {}
                                            }
                                        }
                                        if let (Some(id), Some(href), Some(media_type)) = (id, href, media_type) {
                                            debug!("Adding manifest item: id={}, href={}, media_type={}", id, href, media_type);
                                            manifest.insert(id.clone(), ManifestItem {
                                                id,
                                                href,
                                                media_type,
                                                content: None,
                                                properties: Vec::new(),
                                            });
                                        }
                                    }
                                }
                                Ok(Event::End(ref e2)) => {
                                    let local2_bytes: Vec<u8> = e2.local_name().as_ref().to_vec();
                                    let local2 = &local2_bytes;
                                    debug!("  Manifest inner End tag: {:?}", String::from_utf8_lossy(local2));
                                    if local2 == b"manifest" {
                                        break;
                                    }
                                }
                                Ok(Event::Text(ref t)) => {
                                    debug!("  Manifest inner Text: {:?}", t.unescape().unwrap_or_default());
                                }
                                Ok(Event::Comment(ref c)) => {
                                    debug!("  Manifest inner Comment: {:?}", c.unescape().unwrap_or_default());
                                }
                                Ok(Event::Eof) => break,
                                Err(e) => {
                                    debug!("  Manifest inner Error: {:?}", e);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    } else if local == b"spine" {
                        _in_spine = true;
                        // Enter spine: process all <itemref> tags until </spine>
                        loop {
                            buf.clear();
                            match reader.read_event_into(&mut buf) {
                                Ok(Event::Start(ref e2)) => {
                                    let tag2_bytes: Vec<u8> = e2.name().as_ref().to_vec();
                                    let tag2 = String::from_utf8_lossy(&tag2_bytes).to_string();
                                    let local2_bytes: Vec<u8> = e2.local_name().as_ref().to_vec();
                                    let local2 = &local2_bytes;
                                    debug!("  Spine inner Start tag: {} (local: {:?})", tag2, String::from_utf8_lossy(local2));
                                    if local2 == b"itemref" || tag2.ends_with(":itemref") {
                                        let mut idref = None;
                                        for attr in e2.attributes().flatten() {
                                            if attr.key.local_name().as_ref() == b"idref" {
                                                idref = Some(attr.unescape_value().unwrap_or_default().to_string());
                                            }
                                        }
                                        if let Some(idref) = idref {
                                            if let Some(manifest_item) = manifest.get(&idref) {
                                                spine.push(SpineItem {
                                                    id: idref,
                                                    href: manifest_item.href.clone(),
                                                });
                                            } else {
                                                spine.push(SpineItem {
                                                    id: idref.clone(),
                                                    href: String::new(),
                                                });
                                            }
                                        }
                                    }
                                }
                                Ok(Event::Empty(ref e2)) => {
                                    let tag2_bytes: Vec<u8> = e2.name().as_ref().to_vec();
                                    let tag2 = String::from_utf8_lossy(&tag2_bytes).to_string();
                                    let local2_bytes: Vec<u8> = e2.local_name().as_ref().to_vec();
                                    let local2 = &local2_bytes;
                                    debug!("  Spine inner Empty tag: {} (local: {:?})", tag2, String::from_utf8_lossy(local2));
                                    if local2 == b"itemref" || tag2.ends_with(":itemref") {
                                        let mut idref = None;
                                        for attr in e2.attributes().flatten() {
                                            if attr.key.local_name().as_ref() == b"idref" {
                                                idref = Some(attr.unescape_value().unwrap_or_default().to_string());
                                            }
                                        }
                                        if let Some(idref) = idref {
                                            if let Some(manifest_item) = manifest.get(&idref) {
                                                spine.push(SpineItem {
                                                    id: idref,
                                                    href: manifest_item.href.clone(),
                                                });
                                            } else {
                                                spine.push(SpineItem {
                                                    id: idref.clone(),
                                                    href: String::new(),
                                                });
                                            }
                                        }
                                    }
                                }
                                Ok(Event::End(ref e2)) => {
                                    let local2_bytes: Vec<u8> = e2.local_name().as_ref().to_vec();
                                    let local2 = &local2_bytes;
                                    debug!("  Spine inner End tag: {:?}", String::from_utf8_lossy(local2));
                                    if local2 == b"spine" {
                                        break;
                                    }
                                }
                                Ok(Event::Text(ref t)) => {
                                    debug!("  Spine inner Text: {:?}", t.unescape().unwrap_or_default());
                                }
                                Ok(Event::Comment(ref c)) => {
                                    debug!("  Spine inner Comment: {:?}", c.unescape().unwrap_or_default());
                                }
                                Ok(Event::Eof) => break,
                                Err(e) => {
                                    debug!("  Spine inner Error: {:?}", e);
                                    break;
                                }
                                _ => {}
                            }
                        }
                        _in_spine = false;
                    } else if (local == b"title" || tag.ends_with(":title")) && in_metadata {
                        if let Ok(Event::Text(e)) = reader.read_event_into(&mut buf) {
                            metadata.title = Some(e.unescape().unwrap_or_default().to_string());
                        }
                    } else if (local == b"creator" || tag.ends_with(":creator")) && in_metadata {
                        if let Ok(Event::Text(e)) = reader.read_event_into(&mut buf) {
                            metadata.author = Some(e.unescape().unwrap_or_default().to_string());
                        }
                    }
                }
                Event::End(ref e) => {
                    let local_bytes: Vec<u8> = e.local_name().as_ref().to_vec();
                    let local = &local_bytes;
                    if local == b"metadata" {
                        in_metadata = false;
                    } else if local == b"spine" {
                        _in_spine = false;
                    }
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }

        // Fill in missing hrefs in spine from manifest
        for item in &mut spine {
            if item.href.is_empty() {
                if let Some(manifest_item) = manifest.get(&item.id) {
                    item.href = manifest_item.href.clone();
                }
            }
        }

        debug!("Final manifest item count: {}", manifest.len());
        debug!("Final spine item count: {}", spine.len());

        Ok((metadata, manifest, spine))
    }
    
    fn extract_file(&self, archive: &mut ZipArchive<fs::File>, path: &str) -> Result<Option<Vec<u8>>> {
        match archive.by_name(path) {
            Ok(mut file) => {
                let mut content = Vec::new();
                file.read_to_end(&mut content)?;
                Ok(Some(content))
            }
            Err(_) => {
                debug!("File not found in EPUB: {}", path);
                Ok(None)
            }
        }
    }
    
    pub async fn read_mobi(&self, _path: &Path) -> Result<Book> {
        debug!("Reading MOBI file (placeholder)");
        
        let book = Book::new();
        
        // TODO: Implement MOBI reading
        // - Parse MOBI header
        // - Extract text content
        // - Parse metadata
        
        Ok(book)
    }
    
    pub async fn read_pdf(&self, _path: &Path) -> Result<Book> {
        debug!("Reading PDF file (placeholder)");
        
        let book = Book::new();
        
        // TODO: Implement PDF reading
        // - Extract text content
        // - Parse metadata
        // - Handle images
        
        Ok(book)
    }
    
    pub async fn read_htmlz(&self, _path: &Path) -> Result<Book> {
        debug!("Reading HTMLZ file (placeholder)");
        
        let book = Book::new();
        
        // TODO: Implement HTMLZ reading
        // - Extract ZIP contents
        // - Parse HTML files
        // - Handle CSS and images
        
        Ok(book)
    }
    
    pub async fn read_docx(&self, _path: &Path) -> Result<Book> {
        debug!("Reading DOCX file (placeholder)");
        
        let book = Book::new();
        
        // TODO: Implement DOCX reading
        // - Extract ZIP contents
        // - Parse document.xml
        // - Handle styles and images
        
        Ok(book)
    }
    
    pub async fn read_fb2(&self, _path: &Path) -> Result<Book> {
        debug!("Reading FB2 file (placeholder)");
        
        let book = Book::new();
        
        // TODO: Implement FB2 reading
        // - Parse XML structure
        // - Extract text content
        // - Handle images
        
        Ok(book)
    }
} 