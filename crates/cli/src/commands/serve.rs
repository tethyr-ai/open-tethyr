//! Serve command implementation

use clap::Args;
use open_tethyr::cache::{CacheCoordinatorConfig, RateLimitConfig};
use open_tethyr::policy::enforcement::DomainPolicy;
use open_tethyr::server::{CacheServer, ServerConfig};
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

#[derive(Args)]
pub struct ServeCommand {
    /// Domain to serve cache for
    #[arg(short, long)]
    pub domain: String,

    /// Port to bind to
    #[arg(short, long, default_value = "8080")]
    pub port: u16,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}

impl ServeCommand {
    /// Execute the serve command
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting cache server for domain: {}", self.domain);

        // Build server configuration
        let server_config = if let Some(config_path) = &self.config {
            // Load configuration from file
            info!("Loading configuration from: {}", config_path.display());
            self.load_config_from_file(config_path).await?
        } else {
            // Use default configuration with CLI arguments
            info!("Using default configuration");
            self.build_default_config()
        };

        // Create and start the cache server
        let server = CacheServer::new(server_config).await?;

        info!("Cache server starting on {}:{}", self.domain, self.port);

        // Start the server (this will block until shutdown)
        server.start().await?;

        Ok(())
    }

    /// Build default server configuration from CLI arguments
    fn build_default_config(&self) -> ServerConfig {
        ServerConfig {
            bind_address: format!("0.0.0.0:{}", self.port),
            domain: self.domain.clone(),
            cache_config: CacheCoordinatorConfig::default(),
            domain_policy: DomainPolicy::default(),
            rate_limit_config: RateLimitConfig::default(),
            request_timeout: Duration::from_secs(30),
        }
    }

    /// Load server configuration from file
    async fn load_config_from_file(
        &self,
        _config_path: &PathBuf,
    ) -> Result<ServerConfig, Box<dyn std::error::Error>> {
        // For MVP, we'll use default configuration with CLI overrides
        // Future enhancement: parse YAML/TOML config file for advanced settings
        info!("Configuration file support is limited in MVP - using defaults with CLI overrides");
        Ok(self.build_default_config())
    }
}
