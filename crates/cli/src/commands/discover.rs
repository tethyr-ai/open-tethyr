//! Discover command implementation

use clap::Args;
use open_tethyr::cache::{CacheCoordinator, CacheCoordinatorConfig};
use open_tethyr::http::AxHttpClient;
use std::collections::HashMap;

#[derive(Args)]
pub struct DiscoverCommand {
    /// Target domain to discover agents from
    pub domain: String,

    /// Cache URL to use for discovery
    #[arg(long)]
    pub cache: Option<String>,
}

impl DiscoverCommand {
    /// Execute the discover command
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Discovering agents for domain: {}", self.domain);

        // If cache URL is provided, use it directly
        if let Some(cache_url) = &self.cache {
            println!("Using cache: {}", cache_url);
            self.discover_with_cache(cache_url).await?;
        } else {
            println!("Using direct discovery (no cache)");
            self.discover_direct().await?;
        }

        Ok(())
    }

    /// Discover agents using a specific cache URL
    async fn discover_with_cache(&self, cache_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Create a cache coordinator with manual cache endpoint
        let mut manual_endpoints = HashMap::new();
        manual_endpoints.insert(self.domain.clone(), cache_url.to_string());

        let config = CacheCoordinatorConfig {
            enable_dns_discovery: false, // Disable DNS discovery when using explicit cache
            manual_cache_endpoints: manual_endpoints,
            ..Default::default()
        };

        let coordinator = CacheCoordinator::with_config(config)?;

        // Discover agents
        let document = coordinator.discover(&self.domain).await?;

        // Display results
        self.display_results(&document)?;

        Ok(())
    }

    /// Discover agents directly from the domain (no cache)
    async fn discover_direct(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create HTTP client and fetch directly
        let client = AxHttpClient::new()?;
        let document = client.fetch_ax_record(&self.domain).await?;

        // Display results
        self.display_results(&document)?;

        Ok(())
    }

    /// Display discovered agents in a user-friendly format
    pub fn display_results(
        &self,
        document: &open_tethyr::ax::AgentExchangeDocument,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n✓ Discovery successful!");
        println!("Found {} agent(s):\n", document.records.len());

        for (idx, record) in document.records.iter().enumerate() {
            println!("Agent {}:", idx + 1);
            println!("  Name:        {}", record.agent.name);
            println!("  Description: {}", record.agent.description);
            println!("  Provider:    {}", record.agent.provider);
            println!("  Version:     {}", record.version);

            if !record.endpoints.is_empty() {
                println!("  Endpoints:   {} endpoint(s)", record.endpoints.len());
                for (ep_idx, endpoint) in record.endpoints.iter().enumerate() {
                    println!(
                        "    {}. {:?} - {}",
                        ep_idx + 1,
                        endpoint.protocol,
                        endpoint.url
                    );
                    if !endpoint.auth.is_empty() {
                        println!("       Auth: {}", endpoint.auth.join(", "));
                    }
                }
            }

            if record.capabilities.is_some() {
                println!("  Capabilities: Present");
            }

            println!();
        }

        Ok(())
    }
}
