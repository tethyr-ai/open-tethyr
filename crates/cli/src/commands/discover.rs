//! Discover command implementation

use clap::Args;

#[derive(Args)]
pub struct DiscoverCommand {
    /// Target domain to discover agents from
    pub domain: String,

    /// Cache server URL (optional, overrides DNS discovery)
    #[arg(long)]
    pub cache: Option<String>,

    /// Skip cache, fetch directly
    #[arg(long)]
    pub direct: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Request timeout in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,
}

impl DiscoverCommand {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = if self.direct {
            open_tethyr::OpenTethyr::new(&self.domain)?
        } else {
            open_tethyr::OpenTethyr::with_dns_discovery(&self.domain).await?
        };

        let doc = if let Some(ref cache_url) = self.cache {
            client.discover_with_cache(&self.domain, cache_url).await?
        } else {
            client.discover(&self.domain).await?
        };

        if self.json {
            println!("{}", serde_json::to_string_pretty(&doc)?);
        } else {
            if doc.records.is_empty() {
                println!("No agents found for domain: {}", self.domain);
            } else {
                println!(
                    "Discovered {} agent(s) for {}:",
                    doc.records.len(),
                    self.domain
                );
                for record in &doc.records {
                    println!(
                        "  {} - {} (provider: {})",
                        record.agent.name, record.agent.description, record.agent.provider
                    );
                    for ep in &record.endpoints {
                        println!("    {:?} {}", ep.protocol, ep.url);
                    }
                }
            }
        }

        Ok(())
    }
}
