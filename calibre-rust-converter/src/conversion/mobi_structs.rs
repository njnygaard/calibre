use anyhow::{Result, anyhow};
use std::collections::HashMap;

/// PalmDB header (78 bytes)
#[derive(Debug)]
pub struct PalmDbHeader {
    pub name: [u8; 32],           // Database name
    pub attributes: u16,          // Database attributes
    pub version: u16,             // Version number
    pub creation_date: u32,       // Creation date
    pub modification_date: u32,   // Modification date
    pub last_backup_date: u32,    // Last backup date
    pub modification_number: u32, // Modification number
    pub app_info_id: u32,         // App info ID
    pub sort_info_id: u32,        // Sort info ID
    pub type_code: [u8; 4],       // Type code "BOOK"
    pub creator_code: [u8; 4],    // Creator code "MOBI"
    pub unique_id_seed: u32,      // Unique ID seed
    pub next_record_list_id: u32, // Next record list ID
    pub num_records: u16,         // Number of records
}

impl PalmDbHeader {
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 78 {
            return Err(anyhow!("PalmDB header too short: {} bytes", data.len()));
        }

        let mut name = [0u8; 32];
        name.copy_from_slice(&data[0..32]);

        Ok(PalmDbHeader {
            name,
            attributes: u16::from_be_bytes([data[32], data[33]]),
            version: u16::from_be_bytes([data[34], data[35]]),
            creation_date: u32::from_be_bytes([data[36], data[37], data[38], data[39]]),
            modification_date: u32::from_be_bytes([data[40], data[41], data[42], data[43]]),
            last_backup_date: u32::from_be_bytes([data[44], data[45], data[46], data[47]]),
            modification_number: u32::from_be_bytes([data[48], data[49], data[50], data[51]]),
            app_info_id: u32::from_be_bytes([data[52], data[53], data[54], data[55]]),
            sort_info_id: u32::from_be_bytes([data[56], data[57], data[58], data[59]]),
            type_code: [data[60], data[61], data[62], data[63]],
            creator_code: [data[64], data[65], data[66], data[67]],
            unique_id_seed: u32::from_be_bytes([data[68], data[69], data[70], data[71]]),
            next_record_list_id: u32::from_be_bytes([data[72], data[73], data[74], data[75]]),
            num_records: u16::from_be_bytes([data[76], data[77]]),
        })
    }

    pub fn database_name(&self) -> String {
        String::from_utf8_lossy(&self.name)
            .trim_end_matches('\0')
            .to_string()
    }
}

/// MOBI header structure
#[derive(Debug)]
pub struct MobiHeader {
    pub compression: u16,       // 1=None, 2=PalmDoc, 17480=HUFF/CDIC
    pub text_length: u32,       // Uncompressed text length
    pub record_count: u16,      // Number of text records
    pub record_size: u16,       // Max size of each record (usually 4096)
    pub encryption_type: u16,   // 0=None, 1=Old, 2=Mobipocket
    pub identifier: [u8; 4],    // "MOBI"
    pub header_length: u32,     // Header length (232+ bytes)
    pub mobi_type: u32,         // 2=Book, 3=PalmDoc, etc.
    pub text_encoding: u32,     // 1252=cp1252, 65001=UTF-8
    pub unique_id: u32,         // Unique ID
    pub file_version: u32,      // File version
    pub title_offset: u32,      // Title offset in header
    pub title_length: u32,      // Title length
    pub language: u32,          // Language code
    pub exth_flags: u32,        // EXTH header flags
    pub extra_flags: u16,       // Extra data flags for trailing data
    pub full_title: Option<String>, // Full title from header
}

impl MobiHeader {
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 24 {
            return Err(anyhow!("MOBI header too short"));
        }

        // Parse PalmDoc header (first 16 bytes)
        let compression = u16::from_be_bytes([data[0], data[1]]);
        let text_length = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let record_count = u16::from_be_bytes([data[8], data[9]]);
        let record_size = u16::from_be_bytes([data[10], data[11]]);
        let encryption_type = u16::from_be_bytes([data[12], data[13]]);

        // Check for MOBI identifier
        if data.len() < 20 {
            return Err(anyhow!("No MOBI header found"));
        }

        let identifier = [data[16], data[17], data[18], data[19]];
        if &identifier != b"MOBI" {
            return Err(anyhow!("Invalid MOBI identifier: {:?}", identifier));
        }

        if data.len() < 116 {
            return Err(anyhow!("MOBI header too short for full parsing"));
        }

        let header_length = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        let mobi_type = u32::from_be_bytes([data[24], data[25], data[26], data[27]]);
        let text_encoding = u32::from_be_bytes([data[28], data[29], data[30], data[31]]);
        let unique_id = u32::from_be_bytes([data[32], data[33], data[34], data[35]]);
        let file_version = u32::from_be_bytes([data[36], data[37], data[38], data[39]]);
        
        // Skip to title offset/length (at offset 84/88 from MOBI start)
        let title_offset = if data.len() > 84 {
            u32::from_be_bytes([data[84], data[85], data[86], data[87]])
        } else { 0 };
        
        let title_length = if data.len() > 88 {
            u32::from_be_bytes([data[88], data[89], data[90], data[91]])
        } else { 0 };

        let language = if data.len() > 92 {
            u32::from_be_bytes([data[92], data[93], data[94], data[95]])
        } else { 0 };

        let exth_flags = if data.len() > 128 {
            u32::from_be_bytes([data[128], data[129], data[130], data[131]])
        } else { 0 };

        // Read extra_flags from offset 0xF2 (242)
        let extra_flags = if data.len() > 244 {
            u16::from_be_bytes([data[242], data[243]])
        } else { 0 };

        // Extract title if present
        let full_title = if title_offset > 0 && title_length > 0 && title_offset < data.len() as u32 {
            let start = title_offset as usize;
            let end = std::cmp::min(start + title_length as usize, data.len());
            if start < end {
                Some(String::from_utf8_lossy(&data[start..end]).trim_end_matches('\0').to_string())
            } else {
                None
            }
        } else {
            None
        };

        Ok(MobiHeader {
            compression,
            text_length,
            record_count,
            record_size,
            encryption_type,
            identifier,
            header_length,
            mobi_type,
            text_encoding,
            unique_id,
            file_version,
            title_offset,
            title_length,
            language,
            exth_flags,
            extra_flags,
            full_title,
        })
    }

    pub fn has_exth(&self) -> bool {
        (self.exth_flags & 0x40) != 0
    }

    pub fn compression_name(&self) -> &'static str {
        match self.compression {
            1 => "None",
            2 => "PalmDoc",
            17480 => "HUFF/CDIC",
            _ => "Unknown",
        }
    }

    pub fn encoding_name(&self) -> &'static str {
        match self.text_encoding {
            1252 => "cp1252",
            65001 => "utf-8",
            _ => "unknown",
        }
    }
}

/// EXTH (Extended Header) record
#[derive(Debug)]
pub struct ExthHeader {
    pub records: HashMap<u32, Vec<u8>>,
}

impl ExthHeader {
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 12 || &data[0..4] != b"EXTH" {
            return Err(anyhow!("Invalid EXTH header"));
        }

        let _header_length = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;
        let record_count = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

        let mut records = HashMap::new();
        let mut offset = 12;

        for _ in 0..record_count {
            if offset + 8 > data.len() {
                break;
            }

            let record_type = u32::from_be_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
            let record_length = u32::from_be_bytes([data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7]]) as usize;

            if record_length < 8 || offset + record_length > data.len() {
                break;
            }

            let record_data = data[offset + 8..offset + record_length].to_vec();
            records.insert(record_type, record_data);

            offset += record_length;
        }

        Ok(ExthHeader { records })
    }

    pub fn get_string(&self, record_type: u32) -> Option<String> {
        self.records.get(&record_type)
            .map(|data| String::from_utf8_lossy(data).trim_end_matches('\0').to_string())
    }

    pub fn get_title(&self) -> Option<String> {
        self.get_string(100) // Title
    }

    pub fn get_author(&self) -> Option<String> {
        self.get_string(101) // Author
    }

    pub fn get_publisher(&self) -> Option<String> {
        self.get_string(102) // Publisher
    }

    pub fn get_description(&self) -> Option<String> {
        self.get_string(103) // Description
    }

    pub fn get_language(&self) -> Option<String> {
        self.get_string(524) // Language
    }
}
