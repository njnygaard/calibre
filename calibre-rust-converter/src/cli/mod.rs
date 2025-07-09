pub mod convert;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "calibre-converter")]
#[command(about = "A Rust implementation of Calibre's ebook-convert functionality")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert an ebook from one format to another
    Convert {
        /// Input file path
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
        
        /// Input format (auto-detected if not specified)
        #[arg(long)]
        input_format: Option<String>,
        
        /// Output format (auto-detected if not specified)
        #[arg(long)]
        output_format: Option<String>,
        
        /// Book title
        #[arg(long)]
        title: Option<String>,
        
        /// Book author
        #[arg(long)]
        author: Option<String>,
        
        /// Book language (e.g., en, fr, de)
        #[arg(long)]
        language: Option<String>,
        
        /// Additional metadata as key=value pairs
        #[arg(long, value_delimiter = ',')]
        metadata: Option<Vec<String>>,
        
        /// Conversion options
        #[arg(long, value_delimiter = ',')]
        options: Option<Vec<String>>,
    },
} 