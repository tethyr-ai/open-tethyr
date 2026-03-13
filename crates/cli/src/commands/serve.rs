//! Serve command implementation

use clap::Args;

#[derive(Args)]
pub struct ServeCommand {
    /// Domain to serve
    #[arg(long)]
    pub domain: Option<String>,

    /// Port to bind to
    #[arg(long, default_value = "8080")]
    pub port: u16,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<std::path::PathBuf>,

    /// Maximum cache entries
    #[arg(long, default_value = "10000")]
    pub max_entries: usize,

    /// Default TTL in seconds
    #[arg(long, default_value = "3600")]
    pub ttl: u64,
}

impl ServeCommand {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let server_config = if let Some(ref config_path) = self.config {
            let config = open_tethyr::config::load_config(config_path)?;
            config.server.unwrap_or_else(|| self.default_server_config())
        } else {
            self.default_server_config()
        };

        let server = open_tethyr::server::CacheServer::new(server_config).await?;
        server.start().await?;
        Ok(())
    }

    fn default_server_config(&self) -> open_tethyr::config::ServerConfig {
        open_tethyr::config::ServerConfig {
            domain: self.domain.clone().unwrap_or_else(|| "localhost".to_string()),
            port: self.port,
            cache: open_tethyr::config::CacheConfig {
                max_entries: self.max_entries,
                default_ttl: self.ttl,
                ..Default::default()
            },
            policy: Default::default(),
            rate_limit: Default::default(),
            log_level: std::env::var("OPEN_TETHYR_LOG").unwrap_or_else(|_| "info".to_string()),
        }
    }
}
