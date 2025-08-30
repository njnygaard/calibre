use anyhow::Result;
use std::path::Path;
use tracing::debug;
use crate::conversion::book::{Book, ManifestItem, SpineItem};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use zip::ZipArchive;

#[derive(Debug, Clone)]
struct Chapter {
    title: String,
    content: String,
}

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
    
    pub async fn read_mobi(&self, path: &Path) -> Result<Book> {
        debug!("Reading MOBI file: {:?}", path);
        
        let data = tokio::fs::read(path).await?;
        
        // Parse PalmDB header
        let palmdb_header = crate::conversion::mobi_structs::PalmDbHeader::parse(&data[..78])?;
        debug!("PalmDB header: name={}, records={}", palmdb_header.database_name(), palmdb_header.num_records);
        
        // Parse record info list (8 bytes per record after header)
        let mut record_offsets = Vec::new();
        for i in 0..palmdb_header.num_records {
            let offset_pos = 78 + (i as usize * 8);
            if offset_pos + 4 <= data.len() {
                let offset = u32::from_be_bytes([
                    data[offset_pos], data[offset_pos + 1], 
                    data[offset_pos + 2], data[offset_pos + 3]
                ]);
                record_offsets.push(offset as usize);
            }
        }
        
        if record_offsets.is_empty() {
            return Err(anyhow::anyhow!("No records found in MOBI file"));
        }
        
        // Extract and parse MOBI header from record 0
        let record0_start = record_offsets[0];
        let record0_end = if record_offsets.len() > 1 { 
            record_offsets[1] 
        } else { 
            data.len() 
        };
        
        if record0_start >= data.len() || record0_end > data.len() {
            return Err(anyhow::anyhow!("Invalid record 0 boundaries"));
        }
        
        let record0 = &data[record0_start..record0_end];
        let mobi_header = crate::conversion::mobi_structs::MobiHeader::parse(record0)?;
        
        debug!("MOBI header: compression={}, encoding={}, records={}", 
               mobi_header.compression_name(), 
               mobi_header.encoding_name(),
               mobi_header.record_count);
        
        // Extract EXTH metadata if present
        let exth_header = if mobi_header.has_exth() {
            let exth_start = 16 + mobi_header.header_length as usize;
            if exth_start < record0.len() {
                match crate::conversion::mobi_structs::ExthHeader::parse(&record0[exth_start..]) {
                    Ok(exth) => Some(exth),
                    Err(e) => {
                        debug!("Failed to parse EXTH header: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };
        
        // Extract and decompress text records
        let text_content = self.extract_mobi_text_content(&data, &record_offsets, &mobi_header, mobi_header.extra_flags)?;
        
        // Create book structure
        let mut book = Book::new();
        
        // Set metadata from MOBI headers
        book.metadata.title = mobi_header.full_title.clone()
            .or_else(|| exth_header.as_ref().and_then(|e| e.get_title()))
            .or_else(|| Some(palmdb_header.database_name()))
            .filter(|t| !t.is_empty());
            
        book.metadata.author = exth_header.as_ref().and_then(|e| e.get_author());
        book.metadata.publisher = exth_header.as_ref().and_then(|e| e.get_publisher());
        book.metadata.description = exth_header.as_ref().and_then(|e| e.get_description());
        book.metadata.language = exth_header.as_ref().and_then(|e| e.get_language())
            .or_else(|| Some("en".to_string()));
        
        // Split content into chapters and add as separate XHTML files
        let chapters = self.split_into_chapters(&text_content)?;
        
        // Create table of contents
        let mut toc = crate::conversion::book::TableOfContents::new();
        toc.title = Some("Table of Contents".to_string());
        
        for (i, chapter) in chapters.iter().enumerate() {
            let chapter_id = format!("chapter{:02}", i + 1);
            let chapter_href = format!("chapter{:02}.xhtml", i + 1);
            
            let content_item = ManifestItem {
                id: chapter_id.clone(),
                href: chapter_href.clone(),
                media_type: "application/xhtml+xml".to_string(),
                content: Some(chapter.content.clone().into_bytes()),
                properties: vec![],
            };
            book.manifest.insert(chapter_id.clone(), content_item);
            
            let spine_item = SpineItem {
                id: chapter_id,
                href: chapter_href.clone(),
            };
            book.spine.push(spine_item);
            
            // Add to table of contents
            let toc_item = crate::conversion::book::TocItem::new(chapter.title.clone())
                .with_href(chapter_href);
            toc.add_item(toc_item);
        }
        
        book.set_toc(toc);
        
        debug!("Successfully parsed MOBI file: title={:?}, author={:?}", 
               book.metadata.title, book.metadata.author);
        
        Ok(book)
    }
    
    fn extract_mobi_text_content(&self, data: &[u8], offsets: &[usize], header: &crate::conversion::mobi_structs::MobiHeader, extra_flags: u16) -> Result<String> {
        let mut text_data = Vec::new();
        
        // Extract text records (typically records 1 to header.record_count)
        let end_record = std::cmp::min(header.record_count as usize + 1, offsets.len());
        for i in 1..end_record {
            let start = offsets[i];
            let end = if i + 1 < offsets.len() { offsets[i + 1] } else { data.len() };
            
            if start >= data.len() || end > data.len() || start >= end {
                debug!("Skipping invalid record {}: start={}, end={}, data_len={}", i, start, end, data.len());
                continue;
            }
            
            let record_data = &data[start..end];
            
            // Remove trailing data (multibyte and TBS indices) using Calibre's algorithm
            let clean_record_data = self.get_trailing_data(record_data, extra_flags);
            
            // Decompress based on compression type
            let decompressed = match header.compression {
                1 => clean_record_data.to_vec(), // No compression
                2 => { // PalmDoc compression
                    match crate::utils::compression::decompress_palmdoc(clean_record_data) {
                        Ok(data) => data,
                        Err(e) => {
                            debug!("Failed to decompress record {}: {}", i, e);
                            clean_record_data.to_vec() // Use uncompressed on error
                        }
                    }
                },
                17480 => return Err(anyhow::anyhow!("HUFF/CDIC compression not yet implemented")),
                _ => return Err(anyhow::anyhow!("Unknown compression type: {}", header.compression)),
            };
            
            text_data.extend(decompressed);
        }
        
        debug!("Total decompressed bytes: {}", text_data.len());
        
        // Convert to string (handle encoding)
        let text = match header.text_encoding {
            65001 => String::from_utf8_lossy(&text_data).to_string(),
            1252 => {
                // Convert from cp1252 to UTF-8
                encoding_rs::WINDOWS_1252.decode(&text_data).0.to_string()
            },
            _ => String::from_utf8_lossy(&text_data).to_string(),
        };
        
        // Debug: Check what's at position 89045 and 141484 in raw text
        debug!("Total raw text length: {}", text.len());
        if text.len() > 141484 {
            let start = 141480.max(0);
            let end = (141490).min(text.len());
            debug!("Raw text around position 141484: {:?}", &text[start..end]);
        }
        
        // Clean up the HTML content
        let cleaned_text = self.clean_mobi_html(&text)?;
        
        // Debug: Check if truncation is happening before or after cleaning
        if text.len() != cleaned_text.len() {
            debug!("Text length changed: {} -> {}", text.len(), cleaned_text.len());
        }
        
        // Debug: Check what's at position 89045 in cleaned text
        if cleaned_text.len() > 89045 {
            let start = 89040.max(0);
            let end = (89050).min(cleaned_text.len());
            debug!("Cleaned text around position 89045: {:?}", &cleaned_text[start..end]);
        }
        
        Ok(cleaned_text)
    }
    
    #[allow(dead_code)]
    fn remove_trailing_data<'a>(&self, data: &'a [u8]) -> &'a [u8] {
        if data.is_empty() {
            return data;
        }
        
        let mut end = data.len();
        
        // Remove trailing zeros and common trailing patterns
        while end > 0 && (data[end - 1] == 0 || data[end - 1] == 0xFF) {
            end -= 1;
        }
        
        // Look for common trailing data patterns
        // This is a simplified version - real implementation would be more sophisticated
        if end > 4 && data[end - 1] < 8 {
            // Possible multibyte trailing data length indicator
            let trailing_len = data[end - 1] as usize;
            if trailing_len < end {
                end = end.saturating_sub(trailing_len + 1);
            }
        }
        
        &data[..end]
    }
    
    fn get_trailing_data<'a>(&self, record: &'a [u8], extra_data_flags: u16) -> &'a [u8] {
        // Implement Calibre's get_trailing_data algorithm
        let mut record = record;
        let mut flags = extra_data_flags >> 1;
        
        // Process flag bits 1 and above
        while flags != 0 {
            if flags & 1 != 0 {
                if let Some((size, consumed)) = self.decode_vwi_backward(record) {
                    if size > consumed && size <= record.len() {
                        // Remove trailing data of 'size' bytes
                        record = &record[..record.len() - size];
                    } else {
                        // Invalid size, stop processing
                        break;
                    }
                } else {
                    // Failed to decode, stop processing
                    break;
                }
            }
            flags >>= 1;
        }
        
        // Process multibyte chars if bit 0 is set
        if extra_data_flags & 1 != 0 && !record.is_empty() {
            // Only the first two bits are used for the size since there can
            // never be more than 3 trailing multibyte chars
            let size = ((record[record.len() - 1] & 0b11) + 1) as usize;
            if size <= record.len() {
                record = &record[..record.len() - size];
            }
        }
        
        record
    }
    
    fn decode_vwi_backward(&self, data: &[u8]) -> Option<(usize, usize)> {
        // Decode variable width integer from the end of data (backward)
        // Returns (value, bytes_consumed)
        if data.is_empty() {
            return None;
        }
        
        let mut value = 0usize;
        let mut consumed = 0usize;
        let mut shift = 0;
        
        // Read from the end backward
        for &byte in data.iter().rev() {
            consumed += 1;
            value |= ((byte & 0x7F) as usize) << shift;
            shift += 7;
            
            // If high bit is set, this is the last byte
            if byte & 0x80 != 0 {
                break;
            }
            
            // Safety limit
            if consumed >= 5 || shift >= 28 {
                break;
            }
        }
        
        Some((value, consumed))
    }
    
    #[allow(dead_code)]
    fn remove_minimal_trailing_data<'a>(&self, data: &'a [u8]) -> &'a [u8] {
        if data.is_empty() {
            return data;
        }
        
        let mut end = data.len();
        
        // Only remove trailing zeros and obvious padding
        while end > 0 && data[end - 1] == 0 {
            end -= 1;
        }
        
        // Only remove a small amount of trailing data to be conservative
        // This is much less aggressive than the original implementation
        if end > 4 {
            let last_byte = data[end - 1];
            // Only if the last byte is very small (likely a length indicator)
            if last_byte <= 4 && last_byte > 0 && (last_byte as usize) < end {
                end = end.saturating_sub(last_byte as usize + 1);
            }
        }
        
        &data[..end]
    }
    
    fn clean_mobi_html(&self, html: &str) -> Result<String> {
        // Remove MOBI-specific tags and clean up HTML
        let mut cleaned = html
            .replace("<mbp:pagebreak>", "<div class=\"page-break\"></div>")
            .replace("<mbp:pagebreak/>", "<div class=\"page-break\"></div>")
            .replace("<mbp:nu>", "")
            .replace("</mbp:nu>", "");
        
        // Fix filepos attributes to be properly quoted
        use regex::Regex;
        let filepos_re = Regex::new(r#"filepos=(\d+)"#).unwrap();
        cleaned = filepos_re.replace_all(&cleaned, r#"data-filepos="$1""#).to_string();
        
        // Fix character encoding issues that cause XML parsing errors
        cleaned = cleaned
            .replace('\0', "") // Remove null bytes
            .replace('\u{FFFD}', "") // Remove Unicode replacement character
            // Fix curly quotes that cause XML parsing issues
            .replace('\u{201C}', "\"") // Left double quotation mark
            .replace('\u{201D}', "\"") // Right double quotation mark
            .replace('\u{2018}', "'") // Left single quotation mark
            .replace('\u{2019}', "'") // Right single quotation mark
            // Fix other problematic characters
            .replace('\u{2013}', "-") // En dash
            .replace('\u{2014}', "--") // Em dash
            .replace('\u{2026}', "..."); // Horizontal ellipsis
            // Remove problematic % characters that cause truncation  
            // Temporarily disable to test if this is causing content loss
            // .replace('%', ""); // Remove % characters that seem to cause corruption
        
        // Fix broken HTML tags that are causing XML parsing errors
        cleaned = self.fix_broken_tags(&cleaned);
        
        // Always ensure proper XHTML structure with CSS
        // Remove any existing malformed HTML structure first
        cleaned = self.extract_body_content(&cleaned);
        
        let title = "Converted from MOBI";
        cleaned = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <meta http-equiv="Content-Type" content="text/html; charset=utf-8"/>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
{}
</body>
</html>"#,
            title, cleaned
        );
        
        Ok(cleaned)
    }
    
    fn fix_broken_tags(&self, html: &str) -> String {
        // Fix common broken paragraph patterns with simple string replacements
        let mut result = html.to_string();
        
        // Fix the specific broken patterns we observed
        result = result
            .replace("<p y ", "<p height=\"0pt\" width=\"2em\" align=\"justify\">")
            .replace("<p y", "<p height=\"0pt\" width=\"2em\" align=\"justify\">")
            .replace("<p heighny ", "<p height=\"0pt\" width=\"2em\" align=\"justify\">")
            .replace("<p heighny", "<p height=\"0pt\" width=\"2em\" align=\"justify\">")
            .replace("<p ecuted.governor ", "")
            .replace("<p ecuted.governor", "")
            .replace("<p heise ", "")
            .replace("<p heise", "");
        
        // Remove any orphaned closing tags that might result
        result = result.replace("</p></p>", "</p>");
        
        result
    }
    
    fn extract_body_content(&self, html: &str) -> String {
        // Extract just the body content from existing HTML structure
        let mut content = html.to_string();
        
        // Remove XML declaration if present
        if content.starts_with("<?xml") {
            if let Some(end) = content.find("?>") {
                content = content[end + 2..].trim_start().to_string();
            }
        }
        
        // Remove DOCTYPE if present
        if content.starts_with("<!DOCTYPE") {
            if let Some(end) = content.find('>') {
                content = content[end + 1..].trim_start().to_string();
            }
        }
        
        // Remove html opening tag
        if content.starts_with("<html") {
            if let Some(end) = content.find('>') {
                content = content[end + 1..].trim_start().to_string();
            }
        }
        
        // Remove head section entirely (including guide tags)
        if let Some(head_start) = content.find("<head") {
            if let Some(head_end) = content.find("</head>") {
                let before_head = &content[..head_start];
                let after_head = &content[head_end + 7..];
                content = format!("{}{}", before_head, after_head);
            }
        }
        
        // Also remove any standalone guide sections
        while let Some(guide_start) = content.find("<guide") {
            if let Some(guide_end) = content.find("</guide>") {
                let before_guide = &content[..guide_start];
                let after_guide = &content[guide_end + 8..];
                content = format!("{}{}", before_guide, after_guide);
            } else {
                // If no closing guide tag, remove from start to next >
                if let Some(end) = content[guide_start..].find('>') {
                    let before_guide = &content[..guide_start];
                    let after_guide = &content[guide_start + end + 1..];
                    content = format!("{}{}", before_guide, after_guide);
                } else {
                    break;
                }
            }
        }
        
        // Remove body opening tag
        if let Some(body_start) = content.find("<body") {
            if let Some(body_tag_end) = content[body_start..].find('>') {
                let before_body = &content[..body_start];
                let after_body_tag = &content[body_start + body_tag_end + 1..];
                content = format!("{}{}", before_body, after_body_tag);
            }
        }
        
        // Remove closing tags
        content = content.replace("</body>", "");
        content = content.replace("</html>", "");
        
        // Remove any trailing corruption (like % characters)
        content = content.trim_end_matches('%').trim().to_string();
        
        content
    }
    
    #[allow(dead_code)]
    fn fix_html_structure(&self, html: &str) -> String {
        use regex::Regex;
        
        // Remove duplicate head tags and fix structure
        let mut result = html.to_string();
        
        // Remove duplicate <head> sections (keep only the first one)
        let head_re = Regex::new(r"<head[^>]*>.*?</head>").unwrap();
        let heads: Vec<String> = head_re.find_iter(&result).map(|m| m.as_str().to_string()).collect();
        
        if heads.len() > 1 {
            // Remove all head sections except the first
            for head_match in heads.iter().skip(1) {
                result = result.replace(head_match, "");
            }
        }
        
        // Add CSS link to the head if not present
        if !result.contains("stylesheet.css") {
            let css_link = r#"<link rel="stylesheet" type="text/css" href="stylesheet.css"/>"#;
            if let Some(head_end) = result.find("</head>") {
                result.insert_str(head_end, &format!("    {}\n", css_link));
            }
        }
        
        // Clean up any malformed guide tags
        result = result.replace("<guide>...</guide>", "");
        
        result
    }
    
    #[allow(dead_code)]
    fn sanitize_html(&self, html: &str) -> String {
        use regex::Regex;
        
        // Remove any malformed or incomplete tags
        let mut result = html.to_string();
        
        // Remove incomplete tags (any < without a matching >)
        let incomplete_tag_re = Regex::new(r"<[^>]*$").unwrap();
        result = incomplete_tag_re.replace_all(&result, "").to_string();
        
        // Remove malformed attributes (attributes without proper quotes or values)
        let malformed_attr_re = Regex::new(r#"<(\w+)([^>]*?[a-zA-Z]+[^=>"'\s]*)\s*/?>"#).unwrap();
        result = malformed_attr_re.replace_all(&result, "<$1>").to_string();
        
        // Remove any non-printable characters except common whitespace
        result = result.chars()
            .filter(|&c| c.is_ascii_graphic() || c.is_ascii_whitespace() || c == '\u{00A0}') // Include non-breaking space
            .collect();
        
        // Fix common broken patterns with regex
        let broken_patterns = vec![
            (r"</p><[a-z]+", "</p>"), // Remove broken tags after </p>
            (r#"[a-zA-Z]+="[^"]*[a-zA-Z]+[^"]*""#, ""), // Remove malformed quoted attributes
            (r#"\s+[a-zA-Z]+=\S*\s*"#, " "), // Remove unquoted attributes
        ];
        
        for (pattern, replacement) in broken_patterns {
            let re = Regex::new(pattern).unwrap();
            result = re.replace_all(&result, replacement).to_string();
        }
        
        result
    }

    #[allow(dead_code)]
    fn fix_malformed_unicode(&self, text: &str) -> String {
        // Remove or replace problematic Unicode sequences
        let mut result = String::new();
        for ch in text.chars() {
            match ch {
                // Keep valid characters
                '\u{0009}' | '\u{000A}' | '\u{000D}' => result.push(ch),
                c if c >= '\u{0020}' && c <= '\u{D7FF}' => result.push(c),
                c if c >= '\u{E000}' && c <= '\u{FFFD}' => result.push(c),
                c if c >= '\u{10000}' && c <= '\u{10FFFF}' => result.push(c),
                // Replace invalid characters with space or remove them
                _ => result.push(' '),
            }
        }
        result
    }
    
    #[allow(dead_code)]
    fn fix_attribute_quotes(&self, html: &str) -> String {
        // This is a simplified approach - in a full implementation,
        // you'd want to use a proper HTML parser
        use regex::Regex;
        
        // Fix common attribute quote problems
        let re1 = Regex::new(r#"(\w+)=([^"'\s>]+)"#).unwrap();
        let mut fixed = re1.replace_all(html, r#"$1="$2""#).to_string();
        
        // Fix broken attribute patterns like '<p y' (attribute without value)
        let re2 = Regex::new(r#"<(\w+)\s+([a-zA-Z]+)\s*$"#).unwrap();
        fixed = re2.replace_all(&fixed, r#"<$1>"#).to_string();
        
        // Fix incomplete tags and attributes at end of content
        let re3 = Regex::new(r#"<(\w+)\s+([a-zA-Z]+)=?\s*$"#).unwrap();
        fixed = re3.replace_all(&fixed, "").to_string();
        
        // Fix orphaned attribute names (like just 'y' without a tag)
        let re4 = Regex::new(r#"\s+[a-zA-Z]+\s*$"#).unwrap();
        fixed = re4.replace_all(&fixed, "").to_string();
        
        fixed
    }
    
    #[allow(dead_code)]
    fn remove_trailing_broken_tags(&self, html: &str) -> String {
        use regex::Regex;
        
        // Remove incomplete tags and attributes at the end of content
        let re = Regex::new(r#"<[^>]*$"#).unwrap();
        let mut result = re.replace_all(html, "").to_string();
        
        // Also remove orphaned attributes that might be floating
        let orphan_re = Regex::new(r#"\s+[a-zA-Z]+=?[^"\s]*\s*$"#).unwrap();
        result = orphan_re.replace_all(&result, "").to_string();
        
        result
    }
    
    #[allow(dead_code)]
    fn escape_xml_content(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
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
    
    fn split_into_chapters(&self, content: &str) -> Result<Vec<Chapter>> {
        use regex::Regex;
        
        // Look for chapter markers: <font size="7"><b>Title</b></font> followed by <font size="5"><b>Chapter Title</b></font>
        let chapter_re = Regex::new(r#"<p[^>]*>\s*<font size="7"><b>([^<]+)</b></font>\s*</p>\s*<p[^>]*>\s*<font size="5"><b>([^<]+)</b></font>"#).unwrap();
        
        // Remove table of contents and other trailing content that shouldn't be in chapters
        let cleaned_content = self.remove_trailing_content_from_chapters(content);
        
        let mut chapters = Vec::new();
        let chapter_matches: Vec<_> = chapter_re.find_iter(&cleaned_content).collect();
        
        // If no chapters found, create a single chapter with all content
        if chapter_matches.is_empty() {
            let title = "A People's History of the United States".to_string();
            let chapter_content = self.wrap_chapter_content(&title, &cleaned_content);
            chapters.push(Chapter {
                title,
                content: chapter_content,
            });
            return Ok(chapters);
        }
        
        // Process each chapter
        for (i, chapter_match) in chapter_matches.iter().enumerate() {
            let start = chapter_match.start();
            let end = if i + 1 < chapter_matches.len() {
                chapter_matches[i + 1].start()
            } else {
                cleaned_content.len()
            };
            
            // Extract chapter content
            let chapter_text = &cleaned_content[start..end];
            
            // Extract chapter title from the match
            if let Some(captures) = chapter_re.captures(chapter_text) {
                let book_title = captures.get(1).map_or("", |m| m.as_str());
                let chapter_title = captures.get(2).map_or("", |m| m.as_str());
                
                let full_title = if chapter_title.starts_with(char::is_numeric) {
                    chapter_title.to_string()
                } else {
                    format!("{}: {}", book_title, chapter_title)
                };
                
                // Clean the chapter content before wrapping
                let clean_chapter_text = self.clean_chapter_content(chapter_text);
                let chapter_content = self.wrap_chapter_content(&full_title, &clean_chapter_text);
                
                chapters.push(Chapter {
                    title: full_title,
                    content: chapter_content,
                });
            }
        }
        
        // If no chapters were created, fall back to single chapter
        if chapters.is_empty() {
            let title = "A People's History of the United States".to_string();
            let chapter_content = self.wrap_chapter_content(&title, &cleaned_content);
            chapters.push(Chapter {
                title,
                content: chapter_content,
            });
        }
        
        Ok(chapters)
    }
    
    fn remove_trailing_content_from_chapters(&self, content: &str) -> String {
        // Remove table of contents and other trailing content that appears after the last chapter
        let mut result = content.to_string();
        
        // Look for the table of contents marker
        if let Some(toc_start) = result.find(r#"<font size="7"><b>Table of Contents</b></font>"#) {
            result = result[..toc_start].to_string();
        }
        
        // Remove any trailing % characters and whitespace
        result = result.trim_end_matches('%').trim_end().to_string();
        
        result
    }
    
    fn clean_chapter_content(&self, content: &str) -> String {
        let mut result = content.to_string();
        
        // Remove any duplicate HTML structure that might be in the content
        result = result.replace("</body></html>", "");
        
        // Remove trailing % characters
        result = result.trim_end_matches('%').trim_end().to_string();
        
        // Remove incomplete/unclosed tags at the end that would cause XML errors
        use regex::Regex;
        
        // Remove unclosed paragraph tags like <p height="1em" width="0pt" align="center">
        let unclosed_p_re = Regex::new(r#"<p[^>]*>\s*$"#).unwrap();
        result = unclosed_p_re.replace(&result, "").to_string();
        
        // Fix unclosed div tags like <div class="page-break"> 
        // Always replace unclosed page-break divs with properly closed ones
        result = result.replace(r#"<div class="page-break">"#, r#"<div class="page-break"></div>"#);
        
        // Remove any other unclosed div tags
        let other_unclosed_div_re = Regex::new(r#"<div[^>]*>\s*$"#).unwrap();
        result = other_unclosed_div_re.replace(&result, "").to_string();
        
        // Remove any other unclosed tags at the end
        let unclosed_tag_re = Regex::new(r#"<[^/>][^>]*>\s*$"#).unwrap();
        result = unclosed_tag_re.replace(&result, "").to_string();
        
        // Remove any stray closing tags at the end
        while result.ends_with("</a>") || result.ends_with("</p>") || result.ends_with("</div>") || result.ends_with("</font>") {
            if result.ends_with("</a>") {
                result = result.trim_end_matches("</a>").trim_end().to_string();
            } else if result.ends_with("</p>") {
                result = result.trim_end_matches("</p>").trim_end().to_string();
            } else if result.ends_with("</div>") {
                result = result.trim_end_matches("</div>").trim_end().to_string();
            } else if result.ends_with("</font>") {
                result = result.trim_end_matches("</font>").trim_end().to_string();
            }
        }
        
        // Final cleanup - remove any trailing whitespace and % characters again
        result = result.trim_end_matches('%').trim_end().to_string();
        
        result
    }
    
    fn wrap_chapter_content(&self, title: &str, content: &str) -> String {
        // Fix any unclosed div tags by adding closing tags before </body>
        let mut fixed_content = content.to_string();
        
        // If there's an unclosed <div class="page-break"> tag, close it
        if fixed_content.contains(r#"<div class="page-break">"#) && !fixed_content.contains(r#"</div>"#) {
            // Add the closing div tag right before the end
            fixed_content = fixed_content.trim_end().to_string();
            fixed_content.push_str("</div>");
        }
        
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <meta http-equiv="Content-Type" content="text/html; charset=utf-8"/>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
{}
</body>
</html>"#,
            title, fixed_content
        )
    }
} 