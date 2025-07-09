use std::path::Path;
use std::fs;
use std::io::Read;

#[test]
fn test_epub_sample_exists() {
    let epub_path = Path::new("examples/sample.epub");
    assert!(epub_path.exists(), "Sample EPUB file should exist");
    
    let metadata = fs::metadata(epub_path).unwrap();
    assert!(metadata.len() > 0, "EPUB file should not be empty");
    println!("Sample EPUB file size: {} bytes", metadata.len());
}

#[test]
fn test_epub_sample_structure() {
    // This test would verify the EPUB structure when we implement EPUB reading
    // For now, just check that the file exists and is a valid ZIP
    let epub_path = Path::new("examples/sample.epub");
    
    // Try to read the first few bytes to check if it's a ZIP file
    let mut file = fs::File::open(epub_path).unwrap();
    let mut buffer = [0; 4];
    file.read_exact(&mut buffer).unwrap();
    
    // ZIP files start with PK\x03\x04
    assert_eq!(buffer, [0x50, 0x4B, 0x03, 0x04], "EPUB should be a valid ZIP file");
    println!("EPUB file has valid ZIP header");
}

#[test]
fn test_epub_sample_contents() {
    // This test would verify the EPUB contents when we implement EPUB parsing
    // For now, just check that we can list the ZIP contents
    let epub_path = Path::new("examples/sample.epub");
    
    // Use a simple approach to check ZIP contents
    let output = std::process::Command::new("unzip")
        .arg("-l")
        .arg(epub_path)
        .output()
        .expect("Failed to execute unzip command");
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    // Check for essential EPUB files
    assert!(output_str.contains("META-INF/container.xml"), "Should contain container.xml");
    assert!(output_str.contains("OEBPS/content.opf"), "Should contain content.opf");
    assert!(output_str.contains("OEBPS/toc.ncx"), "Should contain toc.ncx");
    assert!(output_str.contains("OEBPS/chapter1.xhtml"), "Should contain chapter1.xhtml");
    assert!(output_str.contains("OEBPS/chapter2.xhtml"), "Should contain chapter2.xhtml");
    assert!(output_str.contains("OEBPS/chapter3.xhtml"), "Should contain chapter3.xhtml");
    assert!(output_str.contains("OEBPS/style.css"), "Should contain style.css");
    assert!(output_str.contains("OEBPS/cover.jpg"), "Should contain cover.jpg");
    
    println!("EPUB contains all expected files");
    println!("EPUB structure:\n{}", output_str);
} 