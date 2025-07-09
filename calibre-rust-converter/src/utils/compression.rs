use anyhow::Result;
use flate2::read::{DeflateDecoder, GzDecoder, ZlibDecoder};
use flate2::write::{DeflateEncoder, GzEncoder, ZlibEncoder};
use flate2::Compression;
use std::io::{Read, Write};

pub enum CompressionType {
    Deflate,
    Gzip,
    Zlib,
}

pub fn compress_data(data: &[u8], compression_type: CompressionType) -> Result<Vec<u8>> {
    let mut compressed = Vec::new();
    
    match compression_type {
        CompressionType::Deflate => {
            let mut encoder = DeflateEncoder::new(&mut compressed, Compression::default());
            encoder.write_all(data)?;
            encoder.finish()?;
        }
        CompressionType::Gzip => {
            let mut encoder = GzEncoder::new(&mut compressed, Compression::default());
            encoder.write_all(data)?;
            encoder.finish()?;
        }
        CompressionType::Zlib => {
            let mut encoder = ZlibEncoder::new(&mut compressed, Compression::default());
            encoder.write_all(data)?;
            encoder.finish()?;
        }
    }
    
    Ok(compressed)
}

pub fn decompress_data(data: &[u8], compression_type: CompressionType) -> Result<Vec<u8>> {
    let mut decompressed = Vec::new();
    
    match compression_type {
        CompressionType::Deflate => {
            let mut decoder = DeflateDecoder::new(data);
            decoder.read_to_end(&mut decompressed)?;
        }
        CompressionType::Gzip => {
            let mut decoder = GzDecoder::new(data);
            decoder.read_to_end(&mut decompressed)?;
        }
        CompressionType::Zlib => {
            let mut decoder = ZlibDecoder::new(data);
            decoder.read_to_end(&mut decompressed)?;
        }
    }
    
    Ok(decompressed)
}

// PalmDOC compression (used in MOBI files)
pub fn compress_palmdoc(data: &[u8]) -> Result<Vec<u8>> {
    // PalmDOC uses a simple LZ77 compression
    // This is a simplified implementation
    let mut compressed = Vec::new();
    let mut i = 0;
    
    while i < data.len() {
        if i + 2 < data.len() {
            // Look for repeated sequences
            let mut best_match = (0, 0);
            let search_start = if i > 2047 { i - 2048 } else { 0 };
            
            for j in search_start..i {
                let mut match_len = 0;
                while i + match_len < data.len() 
                    && j + match_len < i 
                    && match_len < 10 
                    && data[i + match_len] == data[j + match_len] {
                    match_len += 1;
                }
                
                if match_len >= 3 && match_len > best_match.1 {
                    best_match = (i - j, match_len);
                }
            }
            
            if best_match.1 >= 3 {
                // Write match
                let offset = best_match.0;
                let length = best_match.1;
                compressed.push(0x80 | ((offset >> 8) & 0x7F) as u8);
                compressed.push(offset as u8);
                compressed.push(length as u8);
                i += length;
            } else {
                // Write literal
                compressed.push(data[i]);
                i += 1;
            }
        } else {
            // Write remaining bytes as literals
            compressed.push(data[i]);
            i += 1;
        }
    }
    
    Ok(compressed)
}

pub fn decompress_palmdoc(data: &[u8]) -> Result<Vec<u8>> {
    let mut decompressed = Vec::new();
    let mut i = 0;
    
    while i < data.len() {
        let byte = data[i];
        
        if (byte & 0x80) != 0 {
            // This is a match
            if i + 2 >= data.len() {
                return Err(anyhow::anyhow!("Invalid PalmDOC data"));
            }
            
            let offset = (((byte & 0x7F) as usize) << 8) | (data[i + 1] as usize);
            let length = data[i + 2] as usize;
            
            if offset > decompressed.len() || length == 0 {
                return Err(anyhow::anyhow!("Invalid PalmDOC match"));
            }
            
            for _ in 0..length {
                let pos = decompressed.len() - offset;
                if pos < decompressed.len() {
                    decompressed.push(decompressed[pos]);
                } else {
                    decompressed.push(0);
                }
            }
            
            i += 3;
        } else {
            // This is a literal
            decompressed.push(byte);
            i += 1;
        }
    }
    
    Ok(decompressed)
} 