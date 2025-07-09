use std::path::PathBuf;
use tracing::info;
use crate::formats::{detect_format, Format};
use crate::cli::Commands;

#[derive(Debug)]
pub struct ConvertCommand {
    pub input: PathBuf,
    pub output: PathBuf,
    pub input_format: Option<String>,
    pub output_format: Option<String>,
    pub metadata: Option<Vec<(String, String)>>,
    pub options: Vec<String>,
}

impl ConvertCommand {
    pub fn new(input: PathBuf, output: PathBuf) -> Self {
        Self {
            input,
            output,
            input_format: None,
            output_format: None,
            metadata: None,
            options: Vec::new(),
        }
    }
    
    pub fn from_cli_args(args: &Commands) -> anyhow::Result<Self> {
        match args {
            Commands::Convert {
                input,
                output,
                input_format,
                output_format,
                title,
                author,
                language,
                metadata,
                options,
            } => {
                let mut cmd = Self::new(input.clone(), output.clone());
                
                if let Some(format) = input_format {
                    cmd = cmd.with_input_format(format.clone());
                }
                
                if let Some(format) = output_format {
                    cmd = cmd.with_output_format(format.clone());
                }
                
                // Build metadata from individual fields and additional metadata
                let mut metadata_vec = Vec::new();
                
                if let Some(title) = title {
                    metadata_vec.push(("title".to_string(), title.clone()));
                }
                
                if let Some(author) = author {
                    metadata_vec.push(("author".to_string(), author.clone()));
                }
                
                if let Some(language) = language {
                    metadata_vec.push(("language".to_string(), language.clone()));
                }
                
                // Parse additional metadata from key=value pairs
                if let Some(metadata_list) = metadata {
                    for item in metadata_list {
                        if let Some((key, value)) = item.split_once('=') {
                            metadata_vec.push((key.trim().to_string(), value.trim().to_string()));
                        } else {
                            return Err(anyhow::anyhow!("Invalid metadata format: {}. Expected key=value", item));
                        }
                    }
                }
                
                if !metadata_vec.is_empty() {
                    cmd = cmd.with_metadata(metadata_vec);
                }
                
                if let Some(options_list) = options {
                    cmd = cmd.with_options(options_list.clone());
                }
                
                Ok(cmd)
            }
        }
    }
    
    pub fn with_input_format(mut self, format: String) -> Self {
        self.input_format = Some(format);
        self
    }
    
    pub fn with_output_format(mut self, format: String) -> Self {
        self.output_format = Some(format);
        self
    }
    
    pub fn with_metadata(mut self, metadata: Vec<(String, String)>) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    pub fn with_options(mut self, options: Vec<String>) -> Self {
        self.options = options;
        self
    }
    
    pub async fn execute(&self) -> anyhow::Result<()> {
        info!("Starting conversion from {:?} to {:?}", self.input, self.output);
        
        // Detect input format if not specified
        let input_format = if let Some(ref format) = self.input_format {
            Format::from_string(format)
        } else {
            detect_format(&self.input)?
        };
        
        // Detect output format if not specified
        let output_format = if let Some(ref format) = self.output_format {
            Format::from_string(format)
        } else {
            detect_format(&self.output)?
        };
        
        info!("Converting from {:?} to {:?}", input_format, output_format);
        
        // Create and execute conversion pipeline
        let pipeline = crate::conversion::pipeline::ConversionPipeline::new(
            self.input.clone(),
            self.output.clone(),
            input_format,
            output_format,
        );
        
        // Apply any metadata if provided
        if let Some(ref metadata) = self.metadata {
            // TODO: Apply metadata to the book during conversion
            info!("Metadata provided: {:?}", metadata);
        }
        
        // Execute the conversion
        pipeline.execute().await?;
        
        info!("Conversion completed successfully");
        Ok(())
    }
} 