use encoding_rs::{Encoding, UTF_8};
use std::io::Read;

/// Detect the encoding of a byte slice
pub fn detect_encoding(bytes: &[u8]) -> &'static Encoding {
    // Simple encoding detection based on BOM
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        UTF_8
    } else if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        encoding_rs::UTF_16BE
    } else if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        encoding_rs::UTF_16LE
    } else {
        // Default to UTF-8
        UTF_8
    }
}

/// Convert bytes to string using detected encoding
pub fn bytes_to_string(bytes: &[u8]) -> String {
    let encoding = detect_encoding(bytes);
    let (decoded, _, _) = encoding.decode(bytes);
    decoded.into_owned()
}

/// Convert string to bytes using specified encoding
pub fn string_to_bytes(s: &str, encoding: &'static Encoding) -> Vec<u8> {
    let (encoded, _, _) = encoding.encode(s);
    encoded.into_owned()
}

/// Read file with encoding detection
pub fn read_file_with_encoding<R: Read>(mut reader: R) -> std::io::Result<String> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    Ok(bytes_to_string(&bytes))
} 