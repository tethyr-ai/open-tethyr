//! Open-Tethyr CLI
//!
//! Command-line interface for the Open-Tethyr AX protocol toolkit.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;

use commands::{generate::GenerateCommand, validate::ValidateCommand, discover::DiscoverCommand, serve::ServeCommand};

#[derive(Parser)]
#[command(name = "open-tethyr")]
#[command(about = "Open-Tethyr AX protocol toolkit")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate AX records from configuration
    Generate(GenerateCommand),
    /// Validate AX records
    Validate(ValidateCommand),
    /// Discover agents from domain
    Discover(DiscoverCommand),
    /// Start cache server
    Serve(ServeCommand),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Generate(cmd) => cmd.execute().await?,
        Commands::Validate(cmd) => cmd.execute().await?,
        Commands::Discover(cmd) => cmd.execute().await?,
        Commands::Serve(cmd) => cmd.execute().await?,
    }
    
    Ok(())
}