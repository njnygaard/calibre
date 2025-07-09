use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum Format {
    Epub,
    Mobi,
    Azw3,
    Pdf,
    Txt,
    Html,
    Htmlz,
    Lit,
    Lrf,
    Rtf,
    Docx,
    Fb2,
    Pdb,
    Snb,
    Tcr,
    Rb,
}

impl Format {
    pub fn extension(&self) -> &'static str {
        match self {
            Format::Epub => "epub",
            Format::Mobi => "mobi",
            Format::Azw3 => "azw3",
            Format::Pdf => "pdf",
            Format::Txt => "txt",
            Format::Html => "html",
            Format::Htmlz => "htmlz",
            Format::Lit => "lit",
            Format::Lrf => "lrf",
            Format::Rtf => "rtf",
            Format::Docx => "docx",
            Format::Fb2 => "fb2",
            Format::Pdb => "pdb",
            Format::Snb => "snb",
            Format::Tcr => "tcr",
            Format::Rb => "rb",
        }
    }
    
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "epub" => Format::Epub,
            "mobi" => Format::Mobi,
            "azw3" => Format::Azw3,
            "pdf" => Format::Pdf,
            "txt" => Format::Txt,
            "html" => Format::Html,
            "htmlz" => Format::Htmlz,
            "lit" => Format::Lit,
            "lrf" => Format::Lrf,
            "rtf" => Format::Rtf,
            "docx" => Format::Docx,
            "fb2" => Format::Fb2,
            "pdb" => Format::Pdb,
            "snb" => Format::Snb,
            "tcr" => Format::Tcr,
            "rb" => Format::Rb,
            _ => Format::Txt, // Default to text
        }
    }
    
    pub fn mime_type(&self) -> &'static str {
        match self {
            Format::Epub => "application/epub+zip",
            Format::Mobi => "application/x-mobipocket-ebook",
            Format::Azw3 => "application/vnd.amazon.ebook",
            Format::Pdf => "application/pdf",
            Format::Txt => "text/plain",
            Format::Html => "text/html",
            Format::Htmlz => "application/html+zip",
            Format::Lit => "application/x-ms-reader",
            Format::Lrf => "application/x-sony-bbeb",
            Format::Rtf => "application/rtf",
            Format::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Format::Fb2 => "application/x-fictionbook+xml",
            Format::Pdb => "application/vnd.palm",
            Format::Snb => "application/x-sony-bbeb",
            Format::Tcr => "application/x-tcr",
            Format::Rb => "application/x-rocket-ebook",
        }
    }
}

pub fn detect_format(path: &Path) -> Result<Format> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("txt");
    
    Ok(Format::from_string(extension))
} 