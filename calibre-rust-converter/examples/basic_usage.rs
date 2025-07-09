use calibre_converter::ConvertCommand;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Calibre Rust Converter - Basic Usage Example");
    println!("============================================\n");

    // Example 1: Convert TXT to HTML
    println!("Example 1: Converting TXT to HTML");
    println!("----------------------------------");
    
    let txt_to_html = ConvertCommand::new(
        Path::new("examples/sample.txt").to_path_buf(),
        Path::new("examples/output_sample.html").to_path_buf(),
    )
    .with_input_format("txt".to_string())
    .with_output_format("html".to_string());
    
    println!("Command: {:?}", txt_to_html);
    txt_to_html.execute().await?;
    println!("Output would be saved to: examples/output_sample.html\n");

    // Example 2: Convert HTML to TXT
    println!("Example 2: Converting HTML to TXT");
    println!("----------------------------------");
    
    let html_to_txt = ConvertCommand::new(
        Path::new("examples/sample.html").to_path_buf(),
        Path::new("examples/output_sample.txt").to_path_buf(),
    )
    .with_input_format("html".to_string())
    .with_output_format("txt".to_string());
    
    println!("Command: {:?}", html_to_txt);
    html_to_txt.execute().await?;
    println!("Output would be saved to: examples/output_sample.txt\n");

    // Example 3: Convert EPUB to MOBI (Kindle format)
    println!("Example 3: Converting EPUB to MOBI (Kindle format)");
    println!("--------------------------------------------------");
    
    let epub_to_mobi = ConvertCommand::new(
        Path::new("examples/sample.epub").to_path_buf(),
        Path::new("examples/sample.mobi").to_path_buf(),
    )
    .with_input_format("epub".to_string())
    .with_output_format("mobi".to_string())
    .with_metadata(vec![
        ("title".to_string(), "Sample EPUB Book".to_string()),
        ("author".to_string(), "Rust Converter Team".to_string()),
        ("publisher".to_string(), "Calibre Rust Converter".to_string()),
    ])
    .with_options(vec![
        "--kindle-format".to_string(),
        "--optimize-images".to_string(),
    ]);
    
    println!("Command: {:?}", epub_to_mobi);
    epub_to_mobi.execute().await?;
    println!("Output would be saved to: examples/sample.mobi\n");

    // Example 4: Convert EPUB to AZW3 (Kindle format)
    println!("Example 4: Converting EPUB to AZW3 (Kindle format)");
    println!("---------------------------------------------------");
    
    let epub_to_azw3 = ConvertCommand::new(
        Path::new("examples/sample.epub").to_path_buf(),
        Path::new("examples/sample.azw3").to_path_buf(),
    )
    .with_input_format("epub".to_string())
    .with_output_format("azw3".to_string())
    .with_options(vec![
        "--no-inline-toc".to_string(),
        "--preserve-cover-aspect-ratio".to_string(),
    ]);
    
    println!("Command: {:?}", epub_to_azw3);
    epub_to_azw3.execute().await?;
    println!("Output would be saved to: examples/sample.azw3\n");

    // Example 5: Convert EPUB to PDF
    println!("Example 5: Converting EPUB to PDF");
    println!("----------------------------------");
    
    let epub_to_pdf = ConvertCommand::new(
        Path::new("examples/sample.epub").to_path_buf(),
        Path::new("examples/sample.pdf").to_path_buf(),
    )
    .with_input_format("epub".to_string())
    .with_output_format("pdf".to_string())
    .with_options(vec![
        "--paper-size".to_string(), "a4".to_string(),
        "--margin-top".to_string(), "1in".to_string(),
        "--margin-bottom".to_string(), "1in".to_string(),
        "--margin-left".to_string(), "1in".to_string(),
        "--margin-right".to_string(), "1in".to_string(),
    ]);
    
    println!("Command: {:?}", epub_to_pdf);
    epub_to_pdf.execute().await?;
    println!("Output would be saved to: examples/sample.pdf\n");

    println!("Note: Uncomment the execute() calls to actually run the conversions.");
    println!("The sample.epub file contains a complete book about Rust programming");
    println!("with proper EPUB structure including metadata, navigation, and styling.");

    Ok(())
} 