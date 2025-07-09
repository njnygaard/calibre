use anyhow::Result;
use std::path::Path;
use tracing::debug;
use crate::conversion::book::Book;
use std::fs::File;
use std::io::{Write, BufWriter};


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
    
    pub async fn write_epub(&self, _book: &Book, _path: &Path) -> Result<()> {
        debug!("Writing EPUB file (placeholder)");
        
        // TODO: Implement EPUB writing
        // - Create container.xml
        // - Create content.opf
        // - Create navigation files
        // - Package as ZIP
        
        Ok(())
    }
    
    pub async fn write_mobi(&self, book: &Book, path: &Path) -> Result<()> {
        use std::io::{Seek, SeekFrom};
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
} 