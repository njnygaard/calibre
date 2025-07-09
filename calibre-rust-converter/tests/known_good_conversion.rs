use calibre_converter::ConvertCommand;
use std::path::Path;

#[tokio::test]
async fn test_known_good_epub_to_mobi() {
    let input = "examples/known_good/epub/A People's History of the United States - Howard Zinn.epub";
    let output = "test_known_good_output.mobi";
    let expected = "examples/known_good/mobi/A People's History of the United States - Howard Zinn.mobi";
    
    // Ensure input file exists
    assert!(Path::new(input).exists(), "Input EPUB file should exist");
    assert!(Path::new(expected).exists(), "Expected MOBI file should exist");
    
    // Create conversion command
    let cmd = ConvertCommand::new(input.into(), output.into())
        .with_input_format("epub".to_string())
        .with_output_format("mobi".to_string())
        .with_metadata(vec![
            ("title".to_string(), "A People's History of the United States".to_string()),
            ("author".to_string(), "Howard Zinn".to_string()),
        ]);
    
    // Execute conversion
    let result = cmd.execute().await;
    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());
    
    // Verify output file was created
    assert!(Path::new(output).exists(), "Output MOBI file should be created");
    
    // Check file size (should be substantial)
    let output_metadata = std::fs::metadata(output).expect("Should be able to read output file metadata");
    let expected_metadata = std::fs::metadata(expected).expect("Should be able to read expected file metadata");
    
    println!("Output file size: {} bytes", output_metadata.len());
    println!("Expected file size: {} bytes", expected_metadata.len());
    
    // The output should be a reasonable size (not empty, but may not match exactly due to compression differences)
    assert!(output_metadata.len() > 100_000, "Output file should be substantial (>100KB)");
    
    // Clean up
    let _ = std::fs::remove_file(output);
}

#[tokio::test]
async fn test_known_good_epub_reading() {
    use calibre_converter::conversion::input::InputReader;
    
    let input = "examples/known_good/epub/A People's History of the United States - Howard Zinn.epub";
    let reader = InputReader::new();
    
    // Enable debug logging
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    let result = reader.read_epub(Path::new(input)).await;
    assert!(result.is_ok(), "EPUB reading should succeed: {:?}", result.err());
    
    let book = result.unwrap();
    
    // Verify metadata was extracted correctly
    assert!(book.metadata.title.is_some(), "Book should have a title");
    let extracted_title = book.metadata.title.as_ref().unwrap();
    println!("Extracted title: '{}'", extracted_title);
    // Handle both straight and curly apostrophes
    assert!(
        extracted_title.contains("People") && extracted_title.contains("History") && extracted_title.contains("United States"),
        "Title should contain key words"
    );
    
    assert!(book.metadata.author.is_some(), "Book should have an author");
    let extracted_author = book.metadata.author.as_ref().unwrap();
    println!("Extracted author: '{}'", extracted_author);
    assert!(extracted_author.contains("Zinn"), "Author should contain 'Zinn'");
    
    // Verify content was extracted
    assert!(!book.manifest.is_empty(), "Book should have manifest items");
    println!("Manifest items: {}", book.manifest.len());
    
    assert!(!book.spine.is_empty(), "Book should have spine items");
    println!("Spine items: {}", book.spine.len());
    
    // Verify content is accessible
    for spine_item in &book.spine {
        if let Some(manifest_item) = book.manifest.get(&spine_item.id) {
            assert!(manifest_item.content.is_some(), "Manifest item should have content");
            let content_size = manifest_item.content.as_ref().unwrap().len();
            println!("Spine item {}: {} bytes", spine_item.id, content_size);
        }
    }
    
    // Check for specific expected items
    assert!(book.manifest.contains_key("titlepage"), "Should have titlepage");
    assert!(book.manifest.contains_key("title"), "Should have title");
    assert!(book.manifest.contains_key("cover"), "Should have cover");
    assert!(book.manifest.contains_key("part1"), "Should have part1");
} 