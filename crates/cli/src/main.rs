//! Open-Tethyr CLI
//!
//! Command-line interface for the Open-Tethyr AX protocol toolkit.

use clap::{Parser, Subcommand};

mod commands;

use commands::{
    cache_invalidate::CacheInvalidateCommand, discover::DiscoverCommand,
    generate::GenerateCommand, serve::ServeCommand, validate::ValidateCommand,
};

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
    /// Invalidate cache entries
    CacheInvalidate(CacheInvalidateCommand),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with OPEN_TETHYR_LOG env var support
    // Precedence: CLI > env > config file > defaults (constitution mandate)
    let log_filter = std::env::var("OPEN_TETHYR_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_new(&log_filter)
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Generate(cmd) => cmd.execute().await?,
        Commands::Validate(cmd) => cmd.execute().await?,
        Commands::Discover(cmd) => cmd.execute().await?,
        Commands::Serve(cmd) => cmd.execute().await?,
        Commands::CacheInvalidate(cmd) => cmd.execute().await?,
    }

    Ok(())
}
