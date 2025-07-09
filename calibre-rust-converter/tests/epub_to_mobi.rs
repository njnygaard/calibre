use calibre_converter::ConvertCommand;
use std::path::Path;

#[tokio::test]
async fn test_epub_to_mobi_conversion() {
    let input = "examples/sample.epub";
    let output = "test_output.mobi";
    
    // Ensure input file exists
    assert!(Path::new(input).exists(), "Input EPUB file should exist");
    
    // Create conversion command
    let cmd = ConvertCommand::new(input.into(), output.into())
        .with_input_format("epub".to_string())
        .with_output_format("mobi".to_string())
        .with_metadata(vec![
            ("title".to_string(), "Test EPUB to MOBI Conversion".to_string()),
            ("author".to_string(), "Rust Converter Test".to_string()),
        ]);
    
    // Execute conversion
    let result = cmd.execute().await;
    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());
    
    // Verify output file was created
    assert!(Path::new(output).exists(), "Output MOBI file should be created");
    
    // Check file size (should be non-zero)
    let metadata = std::fs::metadata(output).expect("Should be able to read output file metadata");
    assert!(metadata.len() > 0, "Output file should have non-zero size");
    
    // Clean up
    let _ = std::fs::remove_file(output);
}

#[tokio::test]
async fn test_epub_reading() {
    use calibre_converter::conversion::input::InputReader;
    
    let input = "examples/sample.epub";
    let reader = InputReader::new();
    
    let result = reader.read_epub(Path::new(input)).await;
    assert!(result.is_ok(), "EPUB reading should succeed: {:?}", result.err());
    
    let book = result.unwrap();
    
    // Verify metadata was extracted
    assert!(book.metadata.title.is_some(), "Book should have a title");
    assert!(book.metadata.author.is_some(), "Book should have an author");
    
    // Verify content was extracted
    assert!(!book.manifest.is_empty(), "Book should have manifest items");
    assert!(!book.spine.is_empty(), "Book should have spine items");
    
    // Verify content is accessible
    for spine_item in &book.spine {
        if let Some(manifest_item) = book.manifest.get(&spine_item.id) {
            assert!(manifest_item.content.is_some(), "Manifest item should have content");
        }
    }
}

#[tokio::test]
async fn test_mobi_writing() {
    use calibre_converter::conversion::output::OutputWriter;
    use calibre_converter::conversion::book::Book;
    
    let output = "test_mobi_writing.mobi";
    let writer = OutputWriter::new();
    
    // Create a simple book
    let mut book = Book::new();
    book.metadata.title = Some("Test Book".to_string());
    book.metadata.author = Some("Test Author".to_string());
    
    let result = writer.write_mobi(&book, Path::new(output)).await;
    assert!(result.is_ok(), "MOBI writing should succeed: {:?}", result.err());
    
    // Verify output file was created
    assert!(Path::new(output).exists(), "Output MOBI file should be created");
    
    // Check file size (should be non-zero)
    let metadata = std::fs::metadata(output).expect("Should be able to read output file metadata");
    assert!(metadata.len() > 0, "Output file should have non-zero size");
    
    // Clean up
    let _ = std::fs::remove_file(output);
} 