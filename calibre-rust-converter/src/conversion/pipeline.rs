use anyhow::Result;
use std::path::PathBuf;
use tracing::{debug, info};
use crate::conversion::book::Book;
use crate::conversion::input::InputReader;
use crate::conversion::output::OutputWriter;
use crate::conversion::transforms::TransformPipeline;
use crate::formats::Format;

pub struct ConversionPipeline {
    input_path: PathBuf,
    output_path: PathBuf,
    input_format: Format,
    output_format: Format,
    debug_output: Option<PathBuf>,
}

impl ConversionPipeline {
    pub fn new(
        input_path: PathBuf,
        output_path: PathBuf,
        input_format: Format,
        output_format: Format,
    ) -> Self {
        Self {
            input_path,
            output_path,
            input_format,
            output_format,
            debug_output: None,
        }
    }
    
    pub fn set_debug_output(&mut self, debug_path: PathBuf) {
        self.debug_output = Some(debug_path);
    }
    
    pub async fn execute(&self) -> Result<()> {
        info!("Starting conversion pipeline");
        info!("Input: {:?} ({:?})", self.input_path, self.input_format);
        info!("Output: {:?} ({:?})", self.output_path, self.output_format);
        
        // Step 1: Read input file
        debug!("Reading input file");
        let mut book = self.read_input().await?;
        
        // Step 2: Apply transformations
        debug!("Applying transformations");
        let transform_pipeline = TransformPipeline::new();
        transform_pipeline.apply(&mut book).await?;
        
        // Step 3: Write output file
        debug!("Writing output file");
        self.write_output(&book).await?;
        
        info!("Conversion pipeline completed successfully");
        Ok(())
    }
    
    async fn read_input(&self) -> Result<Book> {
        let reader = InputReader::new();
        
        match self.input_format {
            Format::Txt => reader.read_txt(&self.input_path).await,
            Format::Html => reader.read_html(&self.input_path).await,
            Format::Epub => reader.read_epub(&self.input_path).await,
            Format::Mobi => reader.read_mobi(&self.input_path).await,
            Format::Pdf => reader.read_pdf(&self.input_path).await,
            Format::Htmlz => reader.read_htmlz(&self.input_path).await,
            Format::Docx => reader.read_docx(&self.input_path).await,
            Format::Fb2 => reader.read_fb2(&self.input_path).await,
            _ => {
                // Default to text for unsupported formats
                reader.read_txt(&self.input_path).await
            }
        }
    }
    
    async fn write_output(&self, book: &Book) -> Result<()> {
        let writer = OutputWriter::new();
        
        match self.output_format {
            Format::Txt => writer.write_txt(book, &self.output_path).await,
            Format::Html => writer.write_html(book, &self.output_path).await,
            Format::Epub => writer.write_epub(book, &self.output_path).await,
            Format::Mobi => writer.write_mobi(book, &self.output_path).await,
            Format::Azw3 => writer.write_azw3(book, &self.output_path).await,
            Format::Pdf => writer.write_pdf(book, &self.output_path).await,
            Format::Htmlz => writer.write_htmlz(book, &self.output_path).await,
            _ => {
                // Default to text for unsupported formats
                writer.write_txt(book, &self.output_path).await
            }
        }
    }
} 