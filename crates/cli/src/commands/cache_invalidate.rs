//! Cache invalidation command implementation

use clap::Args;

#[derive(Args)]
pub struct CacheInvalidateCommand {
    /// Domain to invalidate (omit for --all)
    #[arg(long)]
    pub domain: Option<String>,

    /// Clear entire cache
    #[arg(long)]
    pub all: bool,

    /// Cache server URL (required)
    #[arg(long)]
    pub server: String,
}

impl CacheInvalidateCommand {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = if self.all {
            format!("{}/cache", self.server.trim_end_matches('/'))
        } else if let Some(ref domain) = self.domain {
            format!("{}/cache/{}", self.server.trim_end_matches('/'), domain)
        } else {
            return Err("Either --domain or --all must be specified".into());
        };

        let response = client.delete(&url).send().await?;
        if response.status().is_success() {
            if self.all {
                println!("Cache cleared successfully");
            } else {
                println!(
                    "Cache entry for '{}' invalidated",
                    self.domain.as_deref().unwrap_or("")
                );
            }
        } else {
            eprintln!("Cache invalidation failed: HTTP {}", response.status());
            std::process::exit(1);
        }
        Ok(())
    }
}
