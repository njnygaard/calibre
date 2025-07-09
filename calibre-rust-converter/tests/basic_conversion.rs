use calibre_converter::ConvertCommand;
use std::fs;
use std::path::Path;

#[tokio::test]
async fn test_txt_to_html_conversion() {
    // Create test files in current directory
    let input_path = "test_input.txt";
    let output_path = "test_output.html";
    
    // Create a simple test text file
    let test_content = "This is a test document.\n\nIt has multiple paragraphs.\n\nThis should be converted to HTML.";
    fs::write(input_path, test_content).unwrap();
    
    // Create and execute the conversion command
    let convert_cmd = ConvertCommand::new(input_path.into(), output_path.into())
        .with_input_format("txt".to_string())
        .with_output_format("html".to_string());
    
    // Execute the conversion
    let result = convert_cmd.execute().await;
    assert!(result.is_ok(), "Conversion failed: {:?}", result.err());
    
    // Verify the output file exists
    assert!(Path::new(output_path).exists(), "Output file was not created");
    
    // Read and verify the output content
    let output_content = fs::read_to_string(&output_path).unwrap();
    assert!(output_content.contains("This is a test document"));
    assert!(output_content.contains("<html>"));
    assert!(output_content.contains("</html>"));

    // Clean up
    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);
}

#[tokio::test]
async fn test_html_to_txt_conversion() {
    let input_path = "test_input.html";
    let output_path = "test_output.txt";
    
    // Create a simple test HTML file
    let test_html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Document</title>
</head>
<body>
    <h1>Test Document</h1>
    <p>This is a test paragraph.</p>
    <p>This is another paragraph.</p>
</body>
</html>"#;
    fs::write(input_path, test_html).unwrap();
    
    // Create and execute the conversion command
    let convert_cmd = ConvertCommand::new(input_path.into(), output_path.into())
        .with_input_format("html".to_string())
        .with_output_format("txt".to_string());
    
    // Execute the conversion
    let result = convert_cmd.execute().await;
    assert!(result.is_ok(), "Conversion failed: {:?}", result.err());
    
    // Verify the output file exists
    assert!(Path::new(output_path).exists(), "Output file was not created");
    
    // Read and verify the output content
    let output_content = fs::read_to_string(output_path).unwrap();
    assert!(output_content.contains("Test Document"));
    assert!(output_content.contains("This is a test paragraph"));
    assert!(output_content.contains("This is another paragraph"));
    
    // Clean up
    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn test_format_detection() {
    use calibre_converter::formats::{detect_format, Format};
    use std::path::Path;
    
    // Test TXT format detection
    let txt_path = Path::new("test.txt");
    let format = detect_format(txt_path).unwrap();
    assert_eq!(format, Format::Txt);
    
    // Test HTML format detection
    let html_path = Path::new("test.html");
    let format = detect_format(html_path).unwrap();
    assert_eq!(format, Format::Html);
    
    // Test EPUB format detection
    let epub_path = Path::new("test.epub");
    let format = detect_format(epub_path).unwrap();
    assert_eq!(format, Format::Epub);
} 