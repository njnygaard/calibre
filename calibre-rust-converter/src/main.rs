use clap::Parser;
use calibre_converter::cli::{Cli, Commands};
use calibre_converter::cli::convert::ConvertCommand;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Parse CLI arguments
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Convert { .. } => {
            let cmd = ConvertCommand::from_cli_args(&cli.command)?;
            println!("Command: {:?}", cmd);
            
            // Execute the conversion
            cmd.execute().await?;
            
            println!("Conversion completed successfully!");
        }
    }
    
    Ok(())
} 