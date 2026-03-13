use clap::Args;

#[derive(Args)]
pub struct DiscoverCommand {
    pub domain: String,
    #[arg(long)]
    pub cache: Option<String>,
    #[arg(long)]
    pub direct: bool,
    #[arg(long)]
    pub json: bool,
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
        let record = if let Some(ref cache_url) = self.cache {
            client.discover_with_cache(&self.domain, cache_url).await?
        } else {
            client.discover(&self.domain).await?
        };
        if self.json {
            println!("{}", serde_json::to_string_pretty(&record)?);
        } else {
            println!(
                "Agent: {} - {} (provider: {})",
                record.agent.name,
                record.agent.description,
                record.agent.provider.as_deref().unwrap_or("unspecified")
            );
            for ep in &record.endpoints {
                println!("  {:?} {}", ep.protocol, ep.url);
            }
        }
        Ok(())
    }
}
