# Calibre Rust Converter

A Rust implementation of Calibre's `ebook-convert` functionality, providing fast and efficient ebook format conversion.

## Features

- **Multiple Input Formats**: EPUB, MOBI, PDF, TXT, HTML, HTMLZ, DOCX, FB2, and more
- **Multiple Output Formats**: EPUB, MOBI, AZW3, PDF, TXT, HTML, HTMLZ, and more
- **High Performance**: Built in Rust for speed and memory safety
- **Progress Reporting**: Real-time conversion progress with detailed logging
- **Debug Pipeline**: Save intermediate conversion stages for debugging
- **Extensible**: Plugin-based architecture for easy format support extension

## Installation

### Prerequisites

- Rust 1.70+ and Cargo
- For PDF support: Additional system dependencies may be required

### Building from Source

```bash
git clone <repository-url>
cd calibre-rust-converter
cargo build --release
```

### Installation

```bash
cargo install --path .
```

## Usage

### Basic Conversion

```bash
# Convert EPUB to MOBI
calibre-converter convert input.epub output.mobi

# Convert TXT to EPUB
calibre-converter convert input.txt output.epub

# Convert HTML to PDF
calibre-converter convert input.html output.pdf
```

### Advanced Options

```bash
# Verbose output
calibre-converter convert input.epub output.mobi --verbose

# Debug pipeline (save intermediate files)
calibre-converter convert input.epub output.mobi --debug-pipeline ./debug-output
```

### Help

```bash
# Show general help
calibre-converter --help

# Show convert command help
calibre-converter convert --help
```

## Supported Formats

### Input Formats
- **EPUB** (.epub) - Open eBook format
- **MOBI** (.mobi) - Mobipocket format
- **PDF** (.pdf) - Portable Document Format
- **TXT** (.txt) - Plain text files
- **HTML** (.html, .htm) - HyperText Markup Language
- **HTMLZ** (.htmlz) - Compressed HTML
- **DOCX** (.docx) - Microsoft Word documents
- **FB2** (.fb2) - FictionBook format
- **RTF** (.rtf) - Rich Text Format
- **LIT** (.lit) - Microsoft Reader format
- **LRF** (.lrf) - Sony Reader format
- **PDB** (.pdb) - Palm Database format

### Output Formats
- **EPUB** (.epub) - Open eBook format
- **MOBI** (.mobi) - Mobipocket format (legacy)
- **AZW3** (.azw3) - Amazon Kindle format (KF8)
- **PDF** (.pdf) - Portable Document Format
- **TXT** (.txt) - Plain text files
- **HTML** (.html) - HyperText Markup Language
- **HTMLZ** (.htmlz) - Compressed HTML

## Architecture

The converter follows a pipeline-based architecture similar to Calibre:

```
Input File → Input Plugin → Transform Pipeline → Output Plugin → Output File
```

### Core Components

- **Conversion Pipeline**: Orchestrates the entire conversion process
- **Input Plugins**: Read and parse different input formats
- **Output Plugins**: Generate different output formats
- **Transform Pipeline**: Apply various transformations (metadata, structure, CSS, HTML)
- **Book Model**: Internal representation of ebook content and structure

### Key Features

- **Format Detection**: Automatic format detection by file extension and content
- **Progress Reporting**: Real-time progress updates with detailed logging
- **Error Handling**: Comprehensive error handling with helpful messages
- **Debug Support**: Save intermediate conversion stages for troubleshooting
- **Memory Efficient**: Streaming processing for large files

## Development

### Project Structure

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Library root
├── cli/                 # Command-line interface
│   └── convert.rs       # Convert command implementation
├── conversion/          # Core conversion logic
│   ├── pipeline.rs      # Main conversion pipeline
│   ├── book.rs          # Book data model
│   ├── input.rs         # Input format plugins
│   ├── output.rs        # Output format plugins
│   └── transforms.rs    # Content transformation pipeline
├── formats/             # Format detection and support
│   └── mod.rs           # Format definitions and detection
└── utils/               # Utility functions
    ├── encoding.rs      # Text encoding utilities
    ├── compression.rs   # Compression utilities
    └── xml.rs           # XML parsing utilities
```

### Adding New Formats

To add support for a new format:

1. **Add format definition** in `src/formats/mod.rs`
2. **Implement input plugin** in `src/conversion/input.rs`
3. **Implement output plugin** in `src/conversion/output.rs`
4. **Add format detection** in `detect_format_by_content()`
5. **Update CLI help** in `src/cli/convert.rs`

### Building and Testing

```bash
# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run with specific features
cargo build --features epub-support,mobi-support
```

## Performance

The Rust implementation provides significant performance improvements over the Python-based Calibre:

- **Faster startup**: No Python interpreter overhead
- **Lower memory usage**: More efficient data structures
- **Better concurrency**: Async/await for I/O operations
- **Native compilation**: Optimized machine code

## Limitations

### Current Implementation Status

- **Fully Implemented**: TXT, HTML input/output
- **Partially Implemented**: Basic structure for EPUB, MOBI, PDF
- **Not Yet Implemented**: Advanced features like CSS processing, font embedding

### Known Issues

- EPUB parsing and writing need completion
- MOBI/AZW3 format support is basic
- PDF generation requires additional dependencies
- Advanced CSS processing not yet implemented

## Contributing

Contributions are welcome! Please see the contributing guidelines for:

- Code style and formatting
- Testing requirements
- Pull request process
- Issue reporting

## License

This project is licensed under the GPL-3.0 License - see the LICENSE file for details.

## Acknowledgments

- **Calibre**: Original Python implementation that this project is based on
- **Rust Community**: Excellent ecosystem and tooling
- **Open Source Contributors**: Various libraries and tools used in this project

## Roadmap

### Short Term (v0.2)
- Complete EPUB input/output support
- Basic MOBI/AZW3 support
- Improved error handling and reporting

### Medium Term (v0.3)
- PDF input/output support
- CSS processing and optimization
- Font embedding support

### Long Term (v0.4)
- GUI interface
- Batch processing
- Plugin system for custom formats
- Performance optimizations

## Support

For issues, questions, or contributions:

- **Issues**: Use the GitHub issue tracker
- **Discussions**: Use GitHub Discussions
- **Documentation**: Check the inline code documentation

---

*This project is a work in progress and may not support all features of the original Calibre converter yet.* 