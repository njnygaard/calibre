use anyhow::Result;
use std::path::Path;
use tracing::debug;
use crate::conversion::book::Book;
use std::io::Write;


pub struct OutputWriter;

impl OutputWriter {
    pub fn new() -> Self {
        OutputWriter
    }
    
    pub async fn write_txt(&self, book: &Book, path: &Path) -> Result<()> {
        debug!("Writing TXT file: {:?}", path);
        
        let mut content = String::new();
        
        // Extract text content from book
        for spine_item in &book.spine {
            if let Some(manifest_item) = book.manifest.get(&spine_item.id) {
                if let Some(ref content_bytes) = manifest_item.content {
                    if let Ok(text) = String::from_utf8(content_bytes.clone()) {
                        content.push_str(&text);
                        content.push('\n');
                    }
                }
            }
        }
        
        // Write to file
        tokio::fs::write(path, content).await?;
        Ok(())
    }
    
    pub async fn write_html(&self, book: &Book, path: &Path) -> Result<()> {
        debug!("Writing HTML file: {:?}", path);
        
        let mut html = String::new();
        
        // Create HTML header
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<meta charset=\"utf-8\"/>\n");
        html.push_str(&format!("<title>{}</title>\n", 
            book.metadata.title.as_deref().unwrap_or("Untitled")));
        html.push_str("</head>\n<body>\n");
        
        // Add content
        for spine_item in &book.spine {
            if let Some(manifest_item) = book.manifest.get(&spine_item.id) {
                if let Some(ref content_bytes) = manifest_item.content {
                    if let Ok(text) = String::from_utf8(content_bytes.clone()) {
                        html.push_str(&text);
                    }
                }
            }
        }
        
        html.push_str("\n</body>\n</html>");
        
        // Write to file
        tokio::fs::write(path, html).await?;
        Ok(())
    }
    
    pub async fn write_epub(&self, book: &Book, path: &Path) -> Result<()> {
        debug!("Writing EPUB file: {:?}", path);
        
        use zip::{ZipWriter, CompressionMethod};
        use std::fs::File;
        use std::io::Write;
        
        let file = File::create(path)?;
        let mut zip = ZipWriter::new(file);
        
        // Add mimetype (uncompressed, first file)
        let options = zip::write::FileOptions::default()
            .compression_method(CompressionMethod::Stored); // No compression for mimetype
        zip.start_file("mimetype", options)?;
        zip.write_all(b"application/epub+zip")?;
        
        // Add META-INF/container.xml
        let options = zip::write::FileOptions::default()
            .compression_method(CompressionMethod::Deflated);
        zip.start_file("META-INF/container.xml", options)?;
        zip.write_all(self.create_container_xml().as_bytes())?;
        
        // Add OEBPS/content.opf
        zip.start_file("OEBPS/content.opf", options)?;
        zip.write_all(self.create_content_opf(book).as_bytes())?;
        
        // Add OEBPS/toc.ncx
        zip.start_file("OEBPS/toc.ncx", options)?;
        zip.write_all(self.create_toc_ncx(book).as_bytes())?;
        
        // Add OEBPS/stylesheet.css
        zip.start_file("OEBPS/stylesheet.css", options)?;
        zip.write_all(self.create_stylesheet().as_bytes())?;
        
        // Add content files from manifest
        for (_id, item) in &book.manifest {
            if let Some(content) = &item.content {
                let file_path = format!("OEBPS/{}", item.href);
                zip.start_file(&file_path, options)?;
                zip.write_all(content)?;
                debug!("Added content file: {} ({} bytes)", file_path, content.len());
            }
        }
        
        zip.finish()?;
        debug!("Successfully created EPUB file: {:?}", path);
        Ok(())
    }
    
    fn create_container_xml(&self) -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
    <rootfiles>
        <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
    </rootfiles>
</container>"#.to_string()
    }
    
    fn create_content_opf(&self, book: &Book) -> String {
        use uuid::Uuid;
        let book_id = Uuid::new_v4().to_string();
        
        let mut opf = String::new();
        opf.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        opf.push_str("<package version=\"2.0\" xmlns=\"http://www.idpf.org/2007/opf\" unique-identifier=\"BookId\">\n");
        
        // Metadata section
        opf.push_str("  <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:opf=\"http://www.idpf.org/2007/opf\">\n");
        opf.push_str(&format!("    <dc:identifier id=\"BookId\" opf:scheme=\"UUID\">{}</dc:identifier>\n", book_id));
        opf.push_str(&format!("    <dc:title>{}</dc:title>\n", 
            self.escape_xml(book.metadata.title.as_deref().unwrap_or("Untitled"))));
        
        if let Some(author) = &book.metadata.author {
            opf.push_str(&format!("    <dc:creator opf:role=\"aut\">{}</dc:creator>\n", self.escape_xml(author)));
        }
        
        if let Some(publisher) = &book.metadata.publisher {
            opf.push_str(&format!("    <dc:publisher>{}</dc:publisher>\n", self.escape_xml(publisher)));
        }
        
        if let Some(description) = &book.metadata.description {
            opf.push_str(&format!("    <dc:description>{}</dc:description>\n", self.escape_xml(description)));
        }
        
        opf.push_str(&format!("    <dc:language>{}</dc:language>\n", 
            book.metadata.language.as_deref().unwrap_or("en")));
        
        opf.push_str("    <meta name=\"generator\" content=\"Calibre Rust Converter\"/>\n");
        opf.push_str("  </metadata>\n");
        
        // Manifest section
        opf.push_str("  <manifest>\n");
        
        // Add navigation files
        opf.push_str("    <item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\"/>\n");
        
        // Add stylesheet
        opf.push_str("    <item id=\"css\" href=\"stylesheet.css\" media-type=\"text/css\"/>\n");
        
        // Add content items
        for (id, item) in &book.manifest {
            opf.push_str(&format!("    <item id=\"{}\" href=\"{}\" media-type=\"{}\"/>\n",
                self.escape_xml(id),
                self.escape_xml(&item.href),
                self.escape_xml(&item.media_type)
            ));
        }
        
        opf.push_str("  </manifest>\n");
        
        // Spine section
        opf.push_str("  <spine toc=\"ncx\">\n");
        for spine_item in &book.spine {
            opf.push_str(&format!("    <itemref idref=\"{}\"/>\n", self.escape_xml(&spine_item.id)));
        }
        opf.push_str("  </spine>\n");
        
        opf.push_str("</package>\n");
        opf
    }
    
    fn create_toc_ncx(&self, book: &Book) -> String {
        use uuid::Uuid;
        let book_id = Uuid::new_v4().to_string();
        
        let mut ncx = String::new();
        ncx.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        ncx.push_str("<!DOCTYPE ncx PUBLIC \"-//NISO//DTD ncx 2005-1//EN\" \"http://www.daisy.org/z3986/2005/ncx-2005-1.dtd\">\n");
        ncx.push_str("<ncx version=\"2005-1\" xmlns=\"http://www.daisy.org/z3986/2005/ncx/\">\n");
        
        // Head section
        ncx.push_str("  <head>\n");
        ncx.push_str(&format!("    <meta name=\"dtb:uid\" content=\"{}\"/>\n", book_id));
        ncx.push_str("    <meta name=\"dtb:depth\" content=\"1\"/>\n");
        ncx.push_str("    <meta name=\"dtb:totalPageCount\" content=\"0\"/>\n");
        ncx.push_str("    <meta name=\"dtb:maxPageNumber\" content=\"0\"/>\n");
        ncx.push_str("  </head>\n");
        
        // Doc title
        ncx.push_str("  <docTitle>\n");
        ncx.push_str(&format!("    <text>{}</text>\n", 
            self.escape_xml(book.metadata.title.as_deref().unwrap_or("Untitled"))));
        ncx.push_str("  </docTitle>\n");
        
        // Navigation map
        ncx.push_str("  <navMap>\n");
        
        // Use TOC if available, otherwise fall back to spine items
        if let Some(toc) = &book.toc {
            for (index, toc_item) in toc.items.iter().enumerate() {
                ncx.push_str(&format!("    <navPoint id=\"navpoint-{}\" playOrder=\"{}\">\n", index + 1, index + 1));
                ncx.push_str(&format!("      <navLabel><text>{}</text></navLabel>\n", self.escape_xml(&toc_item.title)));
                if let Some(href) = &toc_item.href {
                    ncx.push_str(&format!("      <content src=\"{}\"/>\n", self.escape_xml(href)));
                }
                ncx.push_str("    </navPoint>\n");
            }
        } else {
            // Fallback to spine items
            for (index, spine_item) in book.spine.iter().enumerate() {
                let title = if index == 0 {
                    book.metadata.title.as_deref().unwrap_or("Start").to_string()
                } else {
                    format!("Section {}", index + 1)
                };
                
                ncx.push_str(&format!("    <navPoint id=\"navpoint-{}\" playOrder=\"{}\">\n", index + 1, index + 1));
                ncx.push_str(&format!("      <navLabel><text>{}</text></navLabel>\n", self.escape_xml(&title)));
                ncx.push_str(&format!("      <content src=\"{}\"/>\n", self.escape_xml(&spine_item.href)));
                ncx.push_str("    </navPoint>\n");
            }
        }
        
        ncx.push_str("  </navMap>\n");
        ncx.push_str("</ncx>\n");
        ncx
    }
    
    fn escape_xml(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
    
    pub async fn write_mobi(&self, book: &Book, path: &Path) -> Result<()> {
        // Removed unused imports
        debug!("Writing MOBI file: {:?}", path);

        // --- 1. Prepare content records (PalmDoc compression) ---
        let (content_records, text_length, num_records) = self.generate_content_records(book)?;
        let total_records = 1 + num_records as usize; // 1 header + content

        // --- 2. Build PalmDB header (78 bytes) ---
        let title = book.metadata.title.as_deref().unwrap_or("Untitled");
        let mut pdb_header = vec![0u8; 78];
        let title_bytes = title.as_bytes();
        let copy_len = title_bytes.len().min(32);
        pdb_header[..copy_len].copy_from_slice(&title_bytes[..copy_len]);
        // Set creation/modification times to now (dummy)
        pdb_header[32..36].copy_from_slice(&0u32.to_be_bytes());
        pdb_header[36..40].copy_from_slice(&0u32.to_be_bytes());
        pdb_header[40..44].copy_from_slice(&0u32.to_be_bytes());
        pdb_header[44..48].copy_from_slice(&0u32.to_be_bytes());
        // Set type/creator
        pdb_header[60..64].copy_from_slice(b"BOOK");
        pdb_header[64..68].copy_from_slice(b"MOBI");
        // Section count at offset 76
        pdb_header[76..78].copy_from_slice(&(total_records as u16).to_be_bytes());

        // --- 3. Build header record (PalmDoc+MOBI+EXTH+title) ---
        let mut header_record = Vec::new();
        // PalmDoc header (16 bytes)
        header_record.extend_from_slice(&[0x00, 0x01]); // compression: none (1 = no compression in Calibre format)
        header_record.extend_from_slice(&[0x00, 0x00]); // unused
        header_record.extend_from_slice(&text_length.to_be_bytes()); // text length
        header_record.extend_from_slice(&(4096u16).to_be_bytes()); // record size
        header_record.extend_from_slice(&[0x00, 0x00]); // current position
        header_record.extend_from_slice(&[0x00, 0x00]); // encryption type
        header_record.extend_from_slice(&[0x00, 0x00]); // unknown
        // MOBI header (232 bytes)
        header_record.extend_from_slice(b"MOBI"); // identifier
        header_record.extend_from_slice(&232u32.to_be_bytes()); // header length
        header_record.extend_from_slice(&[0x00, 0x00, 0x00, 0x02]); // mobi type
        header_record.extend_from_slice(&[0x00, 0x00, 0xfd, 0xe9]); // encoding: utf-8
        header_record.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // UID
        header_record.extend_from_slice(&[0x00, 0x00, 0x00, 0x06]); // version
        header_record.extend_from_slice(&[0u8; 32]); // orthographic info
        header_record.extend_from_slice(&[0u8; 4]); // inflection info
        header_record.extend_from_slice(&[0u8; 8]); // index names
        header_record.extend_from_slice(&[0u8; 4]); // index keys
        header_record.extend_from_slice(&[0u8; 4]); // extra index keys
        header_record.extend_from_slice(&[0u8; 4]); // unknown
        header_record.extend_from_slice(&[0u8; 4]); // unknown
        // Title offset/length at 0x54/0x58 (offset 16+0x54=100, 16+0x58=104)
        let title_offset: u32 = 232 + 16; // after MOBI header
        let title_length = title_bytes.len() as u32;
        header_record.extend_from_slice(&title_offset.to_be_bytes());
        header_record.extend_from_slice(&title_length.to_be_bytes());
        // Language code (4 bytes)
        header_record.extend_from_slice(&[0x00, 0x00, 0x00, 0x09]); // English
        header_record.extend_from_slice(&[0x00, 0x00, 0x00, 0x09]); // input lang
        header_record.extend_from_slice(&[0x00, 0x00, 0x00, 0x09]); // output lang
        header_record.extend_from_slice(&[0x00, 0x00, 0x00, 0x06]); // min version
        header_record.extend_from_slice(&[0xff, 0xff, 0xff, 0xff]); // first image index
        header_record.extend_from_slice(&[0u8; 32]); // huff/cdic
        header_record.extend_from_slice(&[0x00, 0x00, 0x00, 0x40]); // EXTH flags
        header_record.extend_from_slice(&[0u8; 36]); // filler to 232 bytes
        // EXTH header
        let mut exth_records = Vec::new();
        exth_records.push((100u32, title_bytes.to_vec()));
        if let Some(author) = &book.metadata.author {
            exth_records.push((101u32, author.as_bytes().to_vec()));
        }
        exth_records.push((524u32, b"en".to_vec()));
        let mut exth_data = Vec::new();
        for (code, data) in &exth_records {
            exth_data.extend_from_slice(&code.to_be_bytes());
            exth_data.extend_from_slice(&(data.len() as u32 + 8).to_be_bytes());
            exth_data.extend_from_slice(data);
        }
        let exth_length = exth_data.len() + 12;
        let padding = (4 - (exth_length % 4)) % 4;
        header_record.extend_from_slice(b"EXTH");
        header_record.extend_from_slice(&(exth_length as u32).to_be_bytes());
        header_record.extend_from_slice(&(exth_records.len() as u32).to_be_bytes());
        header_record.extend_from_slice(&exth_data);
        header_record.extend_from_slice(&vec![0u8; padding]);
        // Title at correct offset
        let current_len = header_record.len();
        if current_len < (title_offset as usize) {
            header_record.extend_from_slice(&vec![0u8; (title_offset as usize) - current_len]);
        }
        header_record.extend_from_slice(title_bytes);
        // Pad header record to 4096 bytes
        while header_record.len() < 4096 {
            header_record.push(0);
        }
        // --- 4. Calculate record offsets ---
        let mut record_offsets = Vec::with_capacity(total_records);
        let mut offset = 78 + total_records * 8; // PalmDB header + record info list
        record_offsets.push(offset as u32);
        offset += 4096;
        for rec in &content_records {
            record_offsets.push(offset as u32);
            offset += rec.len();
        }
        // --- 5. Write file ---
        let mut file = std::fs::File::create(path)?;
        file.write_all(&pdb_header)?;
        for &rec_offset in &record_offsets {
            file.write_all(&rec_offset.to_be_bytes())?;
            file.write_all(&[0u8; 4])?;
        }
        file.write_all(&header_record)?;
        for rec in &content_records {
            file.write_all(rec)?;
        }
        Ok(())
    }
    
    fn generate_content_records(&self, book: &Book) -> Result<(Vec<Vec<u8>>, u32, u16)> {
        let html_content = self.create_html_content(book);
        let html_bytes = html_content.as_bytes();
        
        // For now, don't compress to avoid artifacts
        // TODO: Implement proper PalmDoc compression
        let content = html_bytes.to_vec();
        
        // Split into 4096-byte records
        let mut records = Vec::new();
        let mut offset = 0;
        let mut record_count = 0;
        
        while offset < content.len() {
            let end = std::cmp::min(offset + 4096, content.len());
            let mut record = content[offset..end].to_vec();
            
            // Pad record to 4096 bytes
            while record.len() < 4096 {
                record.push(0);
            }
            
            records.push(record);
            offset = end;
            record_count += 1;
        }
        
        Ok((records, html_bytes.len() as u32, record_count))
    }
    
    fn create_html_content(&self, book: &Book) -> String {
        let mut html = String::new();
        
        // Create HTML header
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<meta charset=\"utf-8\"/>\n");
        html.push_str(&format!("<title>{}</title>\n", 
            book.metadata.title.as_deref().unwrap_or("Untitled")));
        html.push_str("</head>\n<body>\n");
        
        // Add content from spine items
        for spine_item in &book.spine {
            if let Some(manifest_item) = book.manifest.get(&spine_item.id) {
                if let Some(ref content_bytes) = manifest_item.content {
                    if let Ok(text) = String::from_utf8(content_bytes.clone()) {
                        // Extract only the body content, not the full HTML document
                        if let Some(body_start) = text.find("<body>") {
                            if let Some(body_end) = text.find("</body>") {
                                let body_content = &text[body_start + 6..body_end];
                                html.push_str(body_content);
                                html.push_str("\n");
                            }
                        } else {
                            // If no body tags, just add the content as-is
                            html.push_str(&text);
                            html.push_str("\n");
                        }
                    }
                }
            }
        }
        
        html.push_str("</body>\n</html>");
        html
    }
    
    #[allow(dead_code)]
    fn compress_palmdoc(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Simple PalmDoc compression implementation
        // This is a basic implementation - in production you'd want a more robust one
        let mut compressed = Vec::new();
        let mut i = 0;
        
        while i < data.len() {
            if i + 2 < data.len() && data[i] == data[i + 1] && data[i] == data[i + 2] {
                // Run-length encoding for 3+ repeated bytes
                let byte = data[i];
                let mut count = 3;
                i += 3;
                
                while i < data.len() && data[i] == byte && count < 255 {
                    count += 1;
                    i += 1;
                }
                
                compressed.push(0x80 | (count - 3));
                compressed.push(byte);
            } else {
                // Literal byte
                compressed.push(data[i]);
                i += 1;
            }
        }
        
        Ok(compressed)
    }
    
    pub async fn write_azw3(&self, _book: &Book, _path: &Path) -> Result<()> {
        debug!("Writing AZW3 file (placeholder)");
        
        // TODO: Implement AZW3 writing
        // - Similar to MOBI but with KF8 format
        // - Enhanced formatting support
        
        Ok(())
    }
    
    pub async fn write_pdf(&self, _book: &Book, _path: &Path) -> Result<()> {
        debug!("Writing PDF file (placeholder)");
        
        // TODO: Implement PDF writing
        // - Convert HTML to PDF
        // - Handle fonts and images
        // - Create PDF structure
        
        Ok(())
    }
    
    pub async fn write_htmlz(&self, _book: &Book, _path: &Path) -> Result<()> {
        debug!("Writing HTMLZ file (placeholder)");
        
        // TODO: Implement HTMLZ writing
        // - Package HTML files in ZIP
        // - Include CSS and images
        // - Create index.html
        
        Ok(())
    }
    
    fn create_stylesheet(&self) -> String {
        // Create a basic stylesheet based on the known good EPUB
        r#"@namespace h "http://www.w3.org/1999/xhtml";

body {
    font-family: serif;
    margin: 1em;
    line-height: 1.4;
}

p {
    margin: 0 0 1em 0;
    text-align: justify;
    text-indent: 1.5em;
}

.page-break {
    page-break-before: always;
}

font[size="7"] {
    font-size: 2em;
    font-weight: bold;
}

font[size="5"] {
    font-size: 1.5em;
    font-weight: bold;
}

b {
    font-weight: bold;
}

center {
    text-align: center;
}

div {
    margin: 0;
}

/* Style for height and width attributes on paragraphs */
p[height="1em"] {
    margin-top: 1em;
    margin-bottom: 1em;
}

p[height="0pt"] {
    margin-top: 0;
    margin-bottom: 0;
}
"#.to_string()
    }
} 