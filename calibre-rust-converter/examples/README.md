# Calibre Rust Converter Examples

This directory contains example files and sample data for testing the Calibre Rust Converter.

## Sample Files

### Text Files
- `sample.txt` - A simple text file with basic content for testing TXT input/output

### HTML Files  
- `sample.html` - A basic HTML file with structured content for testing HTML input/output

### EPUB Files
- `sample.epub` - A complete EPUB book about Rust programming
- `sample_epub/` - Directory containing the source files used to create the EPUB

## EPUB Sample Details

The `sample.epub` file is a complete, valid EPUB 2.0 book that demonstrates:

### Content
- **Title**: "Sample EPUB Book"
- **Author**: "Rust Converter Team" 
- **Subject**: Programming, Rust, E-books
- **Language**: English
- **Publisher**: Calibre Rust Converter

### Structure
The EPUB contains:
- **Title page** with book information
- **Table of contents** with navigation links
- **3 chapters** covering Rust programming basics:
  - Chapter 1: Introduction to Rust
  - Chapter 2: Getting Started  
  - Chapter 3: Basic Concepts

### Technical Features
- **Valid EPUB 2.0 format** with proper container.xml and content.opf
- **Navigation**: NCX file for table of contents
- **Styling**: CSS stylesheet with typography and layout rules
- **Cover image**: JPEG cover image (400x600 pixels)
- **Metadata**: Complete Dublin Core metadata
- **XHTML content**: Properly structured XHTML 1.1 files

### File Structure
```
sample_epub/
├── META-INF/
│   └── container.xml          # Points to content.opf
└── OEBPS/
    ├── content.opf            # Package document with metadata and manifest
    ├── toc.ncx                # Navigation control file
    ├── titlepage.xhtml        # Title page
    ├── chapter1.xhtml         # Introduction to Rust
    ├── chapter2.xhtml         # Getting Started
    ├── chapter3.xhtml         # Basic Concepts
    ├── style.css              # CSS stylesheet
    └── cover.jpg              # Cover image
```

## Usage Examples

See `basic_usage.rs` for examples of how to use the converter with these sample files:

1. **TXT to HTML conversion**
2. **HTML to TXT conversion** 
3. **EPUB to MOBI conversion** (Kindle format)
4. **EPUB to AZW3 conversion** (Kindle format)
5. **EPUB to PDF conversion**

## Testing

The EPUB sample can be used to test:
- EPUB parsing and validation
- Format conversion to other ebook formats
- Metadata extraction and modification
- Content transformation and styling
- Navigation and table of contents handling

## Validation

The EPUB file has been verified to:
- Have valid ZIP structure (EPUB is a ZIP archive)
- Contain all required EPUB files
- Follow EPUB 2.0 specification
- Be readable by standard EPUB readers

You can validate the EPUB using tools like:
- `epubcheck` (Java-based EPUB validator)
- Calibre's built-in validation
- Online EPUB validators 