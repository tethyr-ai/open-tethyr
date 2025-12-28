# Design Document: Open-Tethyr Rust Toolkit Architecture

## Overview

The open-tethyr Rust toolkit is designed as a modular, high-performance distributed caching system for agent discovery implementing the AX protocol. The architecture follows Rust best practices with a Cargo workspace containing three main crates: a core library providing shared functionality, a CLI tool for generation and testing, and a cache server for distributed caching.

The design emphasizes simplicity, performance, and strict AX protocol compliance while providing the infrastructure control and policy enforcement capabilities that organizations need for agent discovery governance.

## Architecture

### Cargo Workspace Structure

```
open-tethyr/
├── Cargo.toml                 # Workspace root
├── Cargo.lock
├── README.md
├── LICENSE
├── .github/
│   └── workflows/
│       ├── ci.yml
│       ├── release.yml
│       └── docker.yml
├── crates/
│   ├── open-tethyr/           # Main library (renamed from open-tethyr-core)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── ax/            # AX protocol implementation
│   │   │   ├── cache/         # Cache logic
│   │   │   ├── config/        # Configuration handling
│   │   │   ├── dns/           # DNS discovery
│   │   │   ├── http/          # HTTP client
│   │   │   ├── oauth/         # OAuth provider templates
│   │   │   ├── client.rs      # SDK - #[cfg(feature = "client")]
│   │   │   └── server/        # Server - #[cfg(feature = "server")]
│   │   │       ├── mod.rs
│   │   │       ├── cache_server.rs
│   │   │       ├── handlers.rs
│   │   │       ├── middleware.rs
│   │   │       └── policy.rs
│   │   └── tests/
│   └── cli/                   # Single binary for all commands
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs        # Handles all subcommands
│       │   └── commands/
│       │       ├── generate.rs
│       │       ├── validate.rs
│       │       ├── discover.rs
│       │       └── serve.rs   # Uses server feature
│       └── tests/
├── tests/                     # Integration tests
├── examples/                  # Usage examples
└── docker/
    └── Dockerfile
```

### Library Feature Configuration

The main library uses feature flags to provide lightweight SDK by default with optional server functionality:

```toml
# crates/open-tethyr/Cargo.toml
[package]
name = "open-tethyr"              # Matches README usage: use open_tethyr::OpenTethyr;
version = "0.1.0"

[features]
default = ["client"]              # SDK by default (lightweight)
client = []                       # Client SDK functionality
server = ["axum", "tower", "tower-http"]  # Server (opt-in, heavier dependencies)
full = ["client", "server"]       # Everything

[dependencies]
# Core dependencies (always included)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
tokio = { version = "1.0", features = ["full"] }
trust-dns-resolver = "0.23"
lru = "0.12"

# Server-only dependencies (optional)
axum = { version = "0.7", optional = true }
tower = { version = "0.4", optional = true }
tower-http = { version = "0.5", optional = true, features = ["trace"] }
```

### CLI Binary Configuration

```toml
# crates/cli/Cargo.toml
[package]
name = "open-tethyr-cli"
version = "0.1.0"

[[bin]]
name = "open-tethyr"              # Binary name for installation
path = "src/main.rs"

[dependencies]
open-tethyr = { path = "../open-tethyr", features = ["full"] }  # Needs both client and server
clap = { version = "4.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
```

### Dependency Management

The workspace uses shared dependencies defined in the root `Cargo.toml`:

```toml
[workspace]
members = [
    "crates/open-tethyr",
    "crates/cli"
]

[workspace.dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json"] }
clap = { version = "4.0", features = ["derive"] }
anyhow = "1.0"
thiserror = "1.0"
uuid = { version = "1.0", features = ["v4"] }
axum = { version = "0.7", features = ["tokio"] }
tower = { version = "0.4", features = ["full"] }
tower-http = { version = "0.5", features = ["trace", "cors"] }
trust-dns-resolver = "0.23"
lru = "0.12"
proptest = "1.0"
criterion = "0.5"
mockall = "0.12"
wiremock = "0.6"
```

## Components and Interfaces

### Main Library (open-tethyr)

The main library provides all functionality with feature flags for lightweight SDK usage:

```rust
// crates/open-tethyr/src/lib.rs

// Core modules (always available)
pub mod ax;
pub mod cache;
pub mod config;
pub mod dns;
pub mod http;
pub mod oauth;

// Client SDK (default feature)
#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "client")]
pub use client::OpenTethyr;

// Server functionality (opt-in)
#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "server")]
pub use server::CacheServer;
```

### Client SDK Module (`client.rs`)

```rust
// Client SDK for agent discovery (feature = "client")
pub struct OpenTethyr {
    domain: String,
    cache_url: Option<String>,
    http_client: AxHttpClient,
    dns_discovery: DnsDiscovery,
}

impl OpenTethyr {
    pub fn new(domain: &str) -> Result<Self, ClientError> {
        // Initialize with domain and auto-discover cache via DNS
    }
    
    pub async fn discover(&self, target_domain: &str) -> Result<Vec<Agent>, ClientError> {
        // Discover agents from target domain using cache or direct fetch
    }
    
    pub async fn discover_with_cache(&self, target_domain: &str, cache_url: &str) -> Result<Vec<Agent>, ClientError> {
        // Discover using specific cache URL
    }
}
```

#### AX Protocol Module (`ax/`)

```rust
// AX record data structures - aligned with AX RFC specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExchangeRecord {
    #[serde(default = "default_record_type")]
    pub record_type: String,  // Must be "AX"
    
    #[serde(default = "default_version")]
    pub version: String,      // Must be "1.0"
    
    pub agent: Agent,         // SINGULAR - one agent per AX record
    pub endpoints: Vec<Endpoint>,  // Endpoints at record level per RFC
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Capabilities>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<Limits>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Security>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

fn default_record_type() -> String {
    "AX".to_string()
}

fn default_version() -> String {
    "1.0".to_string()
}

// Agent definition (no endpoints - they're at record level)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub description: String,
    pub provider: String,
}

// Container for multiple AX records (open-tethyr extension for multi-agent files)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExchangeDocument {
    pub records: Vec<AgentExchangeRecord>,
}

// Endpoint configuration (auth methods as strings per RFC)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub protocol: Protocol,
    pub url: String,
    pub auth: Vec<String>,  // e.g., ["OIDC", "OAuth2"] per AX RFC
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

// Security section for OAuth configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Security {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth: Option<OAuthEndpoints>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_signature: Option<String>,
}

// AX protocol validation
pub struct AxValidator;
impl AxValidator {
    pub fn validate_record(record: &AgentExchangeRecord) -> Result<(), ValidationError> {
        // Validate record_type is "AX"
        if record.record_type != "AX" {
            return Err(ValidationError::InvalidRecordType(record.record_type.clone()));
        }
        
        // Validate version is supported
        Self::validate_version(&record.version)?;
        
        // Validate agent and endpoints
        Self::validate_agent(&record.agent)?;
        Self::validate_endpoints(&record.endpoints)?;
        
        Ok(())
    }
    
    pub fn validate_agent(agent: &Agent) -> Result<(), ValidationError>;
    pub fn validate_version(version: &str) -> Result<(), ValidationError> {
        match version {
            "1.0" => Ok(()),
            _ => Err(ValidationError::UnsupportedVersion(version.to_string())),
        }
    }
    pub fn validate_endpoints(endpoints: &[Endpoint]) -> Result<(), ValidationError>;
    pub fn validate_auth_methods(auth: &[String]) -> Result<(), ValidationError>;
}

// AX record generation
pub struct AxGenerator;
impl AxGenerator {
    pub fn generate_record(config: &AgentConfig) -> Result<AgentExchangeDocument, GenerationError>;
    pub fn generate_well_known_structure(document: &AgentExchangeDocument) -> Result<WellKnownFiles, GenerationError>;
}
```

#### Cache Module (`cache/`)

```rust
// In-memory cache implementation
pub struct MemoryCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    config: CacheConfig,
    lru: Arc<Mutex<LruCache<String, ()>>>,
    stats: Arc<CacheStats>,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub record: AgentExchangeDocument,  // Updated to use document container
    pub expires_at: SystemTime,
    pub fetched_at: SystemTime,
    pub domain: String,
}

impl MemoryCache {
    pub async fn get(&self, domain: &str) -> Option<AgentExchangeDocument>;
    pub async fn put(&self, domain: &str, document: AgentExchangeDocument, ttl: Duration);
    pub async fn invalidate(&self, domain: &str) -> bool;
    pub async fn clear(&self);
    pub fn size(&self) -> usize;
    pub fn stats(&self) -> CacheStats;
}

// Cache coordination for hierarchical caching
pub struct CacheCoordinator {
    local_cache: MemoryCache,
    root_cache_url: Option<String>,  // Discovered via DNS
    http_client: reqwest::Client,
    dns_discovery: DnsDiscovery,
}

impl CacheCoordinator {
    pub async fn discover(&self, domain: &str) -> Result<AgentExchangeDocument, CacheError>;
    pub async fn fetch_from_upstream(&self, domain: &str) -> Result<AgentExchangeDocument, CacheError>;
    pub async fn fetch_direct(&self, domain: &str) -> Result<AgentExchangeDocument, CacheError>;
}

// Rate limiting module
pub struct RateLimiter {
    limits: Arc<RwLock<HashMap<IpAddr, TokenBucket>>>,
    config: RateLimitConfig,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
}

// Token bucket for rate limiting
pub struct TokenBucket {
    tokens: f64,
    capacity: f64,
    refill_rate: f64,  // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }
    
    pub fn consume(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }
    
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let tokens_to_add = elapsed * self.refill_rate;
        self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
        self.last_refill = now;
    }
}

impl RateLimiter {
    pub fn check_rate_limit(&self, client_ip: IpAddr) -> Result<(), RateLimitError>;
    pub fn reset_limits(&self);
}

// Cache statistics for observability
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: AtomicUsize,
    pub hit_count: AtomicU64,
    pub miss_count: AtomicU64,
    pub eviction_count: AtomicU64,
    pub memory_usage_bytes: AtomicUsize,
}

impl CacheStats {
    pub fn record_hit(&self);
    pub fn record_miss(&self);
    pub fn record_eviction(&self);
    pub fn update_memory_usage(&self, bytes: usize);
}
```

#### Configuration Module (`config/`)

```rust
// Configuration with inheritance support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub defaults: Option<AgentDefaults>,
    pub agents: Vec<AgentDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefaults {
    pub provider: Option<String>,
    pub auth: Option<AuthConfig>,
    pub endpoints: Option<EndpointDefaults>,
    pub capabilities: Option<Capabilities>,
    pub limits: Option<Limits>,
    pub extensions: Option<serde_json::Value>,
}

// Configuration merger with type-safe inheritance
pub struct ConfigMerger;
impl ConfigMerger {
    pub fn merge_agent(agent: &AgentDefinition, defaults: &AgentDefaults) -> Agent;
    pub fn merge_auth(agent_auth: &Option<AuthConfig>, default_auth: &Option<AuthConfig>) -> Option<AuthConfig>;
}

// Configuration validation
pub struct ConfigValidator;
impl ConfigValidator {
    pub fn validate_config(config: &AgentConfig) -> Result<(), ValidationError>;
    pub fn validate_domain(domain: &str) -> Result<(), ValidationError>;
    pub fn validate_url(url: &str) -> Result<(), ValidationError>;
    pub fn validate_port(port: u16) -> Result<(), ValidationError>;
}
```

#### OAuth Provider Templates (`oauth/`)

```rust
// OAuth provider template system
pub trait OAuthProvider {
    fn generate_endpoints(&self, config: &OAuthConfig) -> Result<OAuthEndpoints, OAuthError>;
    fn provider_name(&self) -> &'static str;
}

pub struct OktaProvider;
impl OAuthProvider for OktaProvider {
    fn generate_endpoints(&self, config: &OAuthConfig) -> Result<OAuthEndpoints, OAuthError> {
        let domain = config.domain.as_ref().ok_or(OAuthError::MissingDomain)?;
        Ok(OAuthEndpoints {
            issuer: format!("https://{}", domain),
            authorization_endpoint: format!("https://{}/oauth2/authorize", domain),
            token_endpoint: format!("https://{}/oauth2/token", domain),
            jwks_uri: format!("https://{}/oauth2/v1/keys", domain),
            userinfo_endpoint: format!("https://{}/oauth2/v1/userinfo", domain),
            revocation_endpoint: format!("https://{}/oauth2/v1/revoke", domain),
        })
    }
}

// Similar implementations for Auth0, Azure, Cognito, Google, Keycloak
pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn OAuthProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self;
    pub fn register_provider(&mut self, name: String, provider: Box<dyn OAuthProvider>);
    pub fn generate_oauth_config(&self, provider: &str, config: &OAuthConfig) -> Result<OAuthEndpoints, OAuthError>;
}
```

#### DNS Discovery Module (`dns/`)

```rust
// DNS-based cache discovery
pub struct DnsDiscovery {
    resolver: TokioAsyncResolver,
}

impl DnsDiscovery {
    pub async fn discover_cache(&self, domain: &str) -> Result<Option<String>, DnsError>;
    pub async fn discover_root_cache(&self, domain: &str) -> Result<Option<String>, DnsError>;
    
    async fn lookup_txt_record(&self, record_name: &str) -> Result<Vec<String>, DnsError>;
    
    fn parse_cache_endpoint(txt_record: &str) -> Result<String, DnsError> {
        // Validate format: endpoint=<url>
        if let Some(url) = txt_record.strip_prefix("endpoint=") {
            // Validate URL format
            reqwest::Url::parse(url)
                .map_err(|_| DnsError::InvalidEndpointFormat(txt_record.to_string()))?;
            Ok(url.to_string())
        } else {
            Err(DnsError::InvalidEndpointFormat(txt_record.to_string()))
        }
    }
}
```

#### HTTP Client Module (`http/`)

```rust
// HTTP client for AX endpoint fetching
pub struct AxHttpClient {
    client: reqwest::Client,
    timeout: Duration,
}

impl AxHttpClient {
    pub fn new(timeout: Duration) -> Self;
    
    pub async fn fetch_ax_record(&self, domain: &str) -> Result<AgentExchangeDocument, HttpError>;
    pub async fn fetch_from_cache(&self, cache_url: &str, domain: &str) -> Result<AgentExchangeDocument, HttpError>;
    
    fn build_ax_url(domain: &str) -> String {
        format!("https://_agent.{}/.well-known/agent-exchange.json", domain)
    }
    
    fn validate_well_known_path(url: &reqwest::Url) -> Result<(), HttpError> {
        if url.path() != "/.well-known/agent-exchange.json" {
            return Err(HttpError::InvalidWellKnownPath(url.path().to_string()));
        }
        Ok(())
    }
}

// File writer for CLI output
pub struct FileWriter;
impl FileWriter {
    pub fn write_structure(output_dir: &Path, files: &WellKnownFiles) -> Result<(), IoError>;
    pub fn write_ax_record(path: &Path, document: &AgentExchangeDocument) -> Result<(), IoError>;
}
```

### CLI Tool (cli crate)

The CLI provides a single binary with all commands:

```rust
// crates/cli/src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "open-tethyr")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Generate(GenerateCommand),
    Validate(ValidateCommand),
    Discover(DiscoverCommand),
    Serve(ServeCommand),  // All in one binary
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Generate(cmd) => cmd.execute().await?,
        Commands::Validate(cmd) => cmd.execute().await?,
        Commands::Discover(cmd) => cmd.execute().await?,
        Commands::Serve(cmd) => cmd.execute().await?,  // Uses server feature
    }
    
    Ok(())
}

// Generate command implementation
#[derive(Args)]
pub struct GenerateCommand {
    #[arg(short, long)]
    pub config: PathBuf,
    
    #[arg(short, long)]
    pub output: PathBuf,
    
    #[arg(long)]
    pub validate: bool,
}

impl GenerateCommand {
    pub async fn execute(&self) -> Result<(), CliError> {
        // Load configuration with inheritance
        let config = ConfigLoader::load_from_file(&self.config)?;
        
        // Generate AX records
        let generator = AxGenerator::new();
        let document = generator.generate_record(&config)?;
        
        // Validate if requested
        if self.validate {
            for record in &document.records {
                AxValidator::validate_record(record)?;
            }
        }
        
        // Write well-known structure
        let files = generator.generate_well_known_structure(&document)?;
        FileWriter::write_structure(&self.output, &files)?;
        
        Ok(())
    }
}
```

### Cache Server (server module with feature flag)

The cache server is part of the main library under feature flag:

```rust
// crates/open-tethyr/src/server/mod.rs (feature = "server")
pub mod cache_server;
pub mod handlers;
pub mod middleware;
pub mod policy;

pub use cache_server::CacheServer;

```rust
// Main server application
pub struct CacheServer {
    cache: Arc<MemoryCache>,
    coordinator: Arc<CacheCoordinator>,
    policy: Arc<PolicyEngine>,
    rate_limiter: Arc<RateLimiter>,
    metrics: Arc<CacheMetrics>,
    config: ServerConfig,
}

impl CacheServer {
    pub async fn new(config: ServerConfig) -> Result<Self, ServerError>;
    
    pub async fn start(&self) -> Result<(), ServerError> {
        let app = self.build_routes();
        
        let listener = tokio::net::TcpListener::bind(&self.config.bind_address).await?;
        tracing::info!("Cache server listening on {}", self.config.bind_address);
        
        axum::serve(listener, app).await?;
        Ok(())
    }
    
    fn build_routes(&self) -> Router {
        Router::new()
            .route("/discover/:domain", get(Self::handle_discover))
            .route("/health", get(Self::handle_health))
            .route("/metrics", get(Self::handle_metrics))
            .layer(TraceLayer::new_for_http())
            .layer(RateLimitLayer::new(self.rate_limiter.clone()))
            .with_state(Arc::new(self.clone()))
    }
}

// Cache metrics for observability
pub struct CacheMetrics {
    pub hit_count: AtomicU64,
    pub miss_count: AtomicU64,
    pub request_duration: Arc<Mutex<SimpleHistogram>>,  // Bounded histogram
    pub active_connections: AtomicU32,
}

// Simple bounded histogram for request duration tracking
pub struct SimpleHistogram {
    buckets: Vec<(Duration, u64)>,  // (upper_bound, count)
    total_count: u64,
    sum: Duration,
}

impl SimpleHistogram {
    pub fn new() -> Self {
        Self {
            buckets: vec![
                (Duration::from_millis(1), 0),
                (Duration::from_millis(5), 0),
                (Duration::from_millis(10), 0),
                (Duration::from_millis(50), 0),
                (Duration::from_millis(100), 0),
                (Duration::from_millis(500), 0),
                (Duration::from_secs(1), 0),
                (Duration::from_secs(5), 0),
            ],
            total_count: 0,
            sum: Duration::ZERO,
        }
    }
    
    pub fn record(&mut self, duration: Duration) {
        self.total_count += 1;
        self.sum += duration;
        
        for (upper_bound, count) in &mut self.buckets {
            if duration <= *upper_bound {
                *count += 1;
                break;
            }
        }
    }
}

impl CacheMetrics {
    pub fn record_hit(&self);
    pub fn record_miss(&self);
    pub fn record_request_duration(&self, duration: Duration);
    pub fn increment_connections(&self);
    pub fn decrement_connections(&self);
}

// Discovery endpoint handler
impl CacheServer {
    async fn handle_discover(
        State(server): State<Arc<CacheServer>>,
        Path(domain): Path<String>,
        headers: HeaderMap,
    ) -> Result<(HeaderMap, Json<AgentExchangeDocument>), ServerError> {
        let correlation_id = uuid::Uuid::new_v4();
        let span = tracing::info_span!("discover", domain = %domain, correlation_id = %correlation_id);
        let _enter = span.enter();
        
        let start_time = std::time::Instant::now();
        
        // Rate limiting
        let client_ip = extract_client_ip(&headers)?;
        server.rate_limiter.check_rate_limit(client_ip)?;
        
        // Policy enforcement
        server.policy.check_discovery_allowed(&domain)?;
        
        // Attempt discovery through cache coordinator
        let document = server.coordinator.discover(&domain).await?;
        
        // Record metrics
        server.metrics.record_request_duration(start_time.elapsed());
        
        // Add correlation ID to response headers
        let mut response_headers = HeaderMap::new();
        response_headers.insert(
            "X-Correlation-Id",
            correlation_id.to_string().parse().unwrap()
        );
        
        tracing::info!("Discovery successful", records_count = document.records.len());
        Ok((response_headers, Json(document)))
    }
    
    async fn handle_health() -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }
    
    async fn handle_metrics(State(server): State<Arc<CacheServer>>) -> String {
        // Basic Prometheus text format (full exposition with labels/types in v0.2)
        format!(
            "cache_hits_total {}\ncache_misses_total {}\nactive_connections {}\n",
            server.metrics.hit_count.load(std::sync::atomic::Ordering::Relaxed),
            server.metrics.miss_count.load(std::sync::atomic::Ordering::Relaxed),
            server.metrics.active_connections.load(std::sync::atomic::Ordering::Relaxed),
        )
    }
}

fn extract_client_ip(headers: &HeaderMap) -> Result<IpAddr, ServerError> {
    // TODO: Extract from X-Forwarded-For header
    // TODO: Fallback to connection peer address
    // For MVP, return localhost
    Ok("127.0.0.1".parse().unwrap())
}

### Usage Patterns

The feature flag system enables different usage patterns:

**SDK Users (Lightweight):**
```toml
[dependencies]
open-tethyr = "0.1"  # Default: client feature only
```
```rust
use open_tethyr::OpenTethyr;

let client = OpenTethyr::new("acme.com")?;
let agents = client.discover("api.partner.com").await?;
```

**Server Library Users:**
```toml
[dependencies]
open-tethyr = { version = "0.1", features = ["server"] }
```
```rust
use open_tethyr::{OpenTethyr, CacheServer};

// Can use both client SDK and embed cache server
let client = OpenTethyr::new("acme.com")?;
let server = CacheServer::new(config).await?;
```

**CLI Installation:**
```bash
cargo install open-tethyr-cli
# Provides single 'open-tethyr' binary with all commands
open-tethyr generate --config agents.yaml --output ./ax-records
open-tethyr serve --mode cache --domain acme.com
```
```

## Data Models

### AX Protocol Data Structures

The system implements complete AX 1.0 specification data models. Here's an example of the correct AX record structure:

```json
{
  "record_type": "AX",
  "version": "1.0",
  "agent": {
    "name": "Customer Lookup",
    "description": "Query customer database",
    "provider": "Acme Corp"
  },
  "endpoints": [
    {
      "protocol": "rest",
      "url": "https://api.acme.com/agents/customer-lookup",
      "auth": ["OAuth2"],
      "content_type": "application/json"
    }
  ],
  "capabilities": {
    "async": true,
    "supports_callbacks": true
  },
  "security": {
    "oauth": {
      "issuer": "https://auth.acme.com",
      "authorization_endpoint": "https://auth.acme.com/oauth2/authorize",
      "token_endpoint": "https://auth.acme.com/oauth2/token"
    }
  }
}
```

The corresponding Rust data structures:

```rust
// Core AX record structure (aligned with AX RFC)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExchangeRecord {
    #[serde(default = "default_record_type")]
    pub record_type: String,  // Must be "AX"
    
    #[serde(default = "default_version")]
    pub version: String,      // Must be "1.0"
    
    pub agent: Agent,         // SINGULAR - one agent per AX record
    pub endpoints: Vec<Endpoint>,  // Endpoints at record level per RFC
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Capabilities>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<Limits>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Security>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

fn default_record_type() -> String {
    "AX".to_string()
}

fn default_version() -> String {
    "1.0".to_string()
}

// Agent identity (no endpoints - they're at record level)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub description: String,
    pub provider: String,
}

// Container for multiple AX records (open-tethyr extension for multi-agent files)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExchangeDocument {
    pub records: Vec<AgentExchangeRecord>,
}

// Endpoint configuration (auth methods as strings per RFC)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub protocol: Protocol,
    pub url: String,
    pub auth: Vec<String>,  // e.g., ["OIDC", "OAuth2"] per AX RFC
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Rest,
    GraphQL,
    MCP,
    A2A,
    #[serde(untagged)]
    Custom(String),
}
```

### Configuration Data Models

```rust
// Configuration with inheritance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defaults: Option<AgentDefaults>,
    pub agents: Vec<AgentDefinition>,
}

// Agent definition before inheritance resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    pub description: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    
    pub endpoints: Vec<EndpointDefinition>,
    
    // Optional fields that can inherit from defaults
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Capabilities>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<Limits>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Security>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}
```

### Cache Storage Models

```rust
// Cache entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub document: AgentExchangeDocument,  // Updated to use document container
    pub expires_at: SystemTime,
    pub fetched_at: SystemTime,
    pub domain: String,
    pub cache_hit_count: u64,
    pub last_accessed: SystemTime,
}

// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_entries: usize,
    pub default_ttl: Duration,
    pub cleanup_interval: Duration,
    pub enable_lru: bool,
}
```

### Discovery Flow

The complete agent discovery flow follows this sequence:

```
Discovery Flow:
1. Client initializes with domain "acme.com"
2. DNS lookup: _ax-cache.acme.com TXT record
3. If cache found: query cache for "api.partner.com"
4. Cache miss: fetch https://_agent.api.partner.com/.well-known/agent-exchange.json
5. Parse and validate AX record (record_type="AX", version="1.0")
6. Store in cache with TTL from Cache-Control headers
7. Return AgentExchangeDocument to client

Hierarchical Fallback:
Local Cache Miss → Root Cache (if configured) → Direct HTTPS Fetch

Error Handling:
DNS Failure → Direct Discovery (no error propagation)
Cache Failure → Fallback to next level
Rate Limit → HTTP 429 response
Policy Violation → HTTP 403 response
```

## Error Handling

The system uses structured error handling with `thiserror` for clear error propagation:

```rust
// Core library errors
#[derive(Debug, thiserror::Error)]
pub enum AxError {
    #[error("Validation error: {message}")]
    Validation { message: String },
    
    #[error("Unsupported AX version: {version}")]
    UnsupportedVersion { version: String },
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("DNS error: {0}")]
    Dns(#[from] trust_dns_resolver::error::ResolveError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

// Server-specific errors
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Policy violation: {message}")]
    PolicyViolation { message: String },
    
    #[error("Rate limit exceeded for client {client_ip}")]
    RateLimitExceeded { client_ip: String },
    
    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
}

// HTTP error responses
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ServerError::PolicyViolation { .. } => (StatusCode::FORBIDDEN, self.to_string()),
            ServerError::RateLimitExceeded { .. } => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            ServerError::Cache(CacheError::NotFound { .. }) => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };
        
        let body = Json(serde_json::json!({
            "error": error_message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }));
        
        (status, body).into_response()
    }
}
```

## Testing Strategy

The testing strategy implements both unit tests and property-based tests for comprehensive coverage:

### Unit Testing Approach

- **Component isolation**: Each module has focused unit tests
- **Mock external dependencies**: HTTP clients, DNS resolvers, file systems
- **Error path coverage**: Test all error conditions and edge cases
- **Configuration validation**: Test all configuration validation rules

### Property-Based Testing Framework

The system uses `proptest` for property-based testing of core functionality:

```rust
// Example property test setup
use proptest::prelude::*;

// Generator for valid AX records
fn arb_agent_record() -> impl Strategy<Value = AgentExchangeRecord> {
    (
        prop::collection::vec(arb_agent(), 1..10),
        prop::option::of(arb_record_metadata()),
    ).prop_map(|(agents, metadata)| AgentExchangeRecord {
        version: "1.0".to_string(),
        agents,
        metadata,
    })
}

// Generator for valid agents
fn arb_agent() -> impl Strategy<Value = Agent> {
    (
        "[a-zA-Z][a-zA-Z0-9 -]{2,50}",  // name
        "[a-zA-Z0-9 .,!?-]{10,200}",    // description  
        "[a-zA-Z][a-zA-Z0-9 ]{2,50}",  // provider
        prop::collection::vec(arb_endpoint(), 1..5),
    ).prop_map(|(name, description, provider, endpoints)| Agent {
        name,
        description,
        provider,
        endpoints,
        capabilities: None,
        schema: None,
        limits: None,
        security: None,
        extensions: None,
    })
}
```

### Integration Testing

- **Multi-component workflows**: Test CLI → Core → Server interactions
- **Network simulation**: Mock HTTP servers for testing discovery flows
- **Configuration scenarios**: Test various configuration inheritance patterns
- **Cache coordination**: Test hierarchical caching with multiple cache levels

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

Based on the prework analysis, the following properties validate the core correctness requirements of the system:

### Property 1: AX Record Generation Correctness
*For any* valid YAML configuration, generating AX records should produce valid JSON with record_type="AX", version="1.0", and proper AX RFC structure with endpoints at record level.
**Validates: Requirements 2.1, 9.1, 9.3**

### Property 2: AX Record Validation Correctness  
*For any* AX record, validation should correctly identify compliance with AX 1.0 specification, rejecting records with missing record_type, invalid version, or malformed structure while accepting valid records.
**Validates: Requirements 2.2, 5.5**

### Property 3: Well-Known File Structure Generation
*For any* generated AX output, the file structure should include the standard /.well-known/agent-exchange.json path and correct directory organization.
**Validates: Requirements 2.5, 9.2**

### Property 4: Cache-First Discovery Behavior
*For any* discovery request, when a cache is available, the cache should be checked first before making external requests, and cache hits should not trigger external fetches.
**Validates: Requirements 3.2**

### Property 5: Cache Miss Fallback Behavior
*For any* cache miss, the system should fetch from the target domain's correct AX endpoint (https://_agent.<domain>/.well-known/agent-exchange.json) and validate the response path.
**Validates: Requirements 3.3, 9.6**

### Property 6: Domain Locking Policy Enforcement
*For any* discovery request when domain locking is enabled, only agents from the home domain should be cached and served, with external domain requests being rejected with appropriate HTTP error responses.
**Validates: Requirements 3.4, 12.1, 12.2, 12.5**

### Property 7: TTL Expiration Correctness
*For any* cached record with TTL, expired records should not be returned from cache and should trigger fresh fetches, while non-expired records should be served from cache.
**Validates: Requirements 3.5, 10.6**

### Property 8: Serialization Round-Trip Consistency
*For any* valid AX data structure, serializing then deserializing should produce an equivalent object with all fields preserved correctly, including record_type and version fields.
**Validates: Requirements 5.2, 5.3**

### Property 9: OAuth Provider Template Correctness
*For any* supported OAuth provider (Okta, Auth0, Azure, Cognito, Google, Keycloak), generating OAuth endpoints should follow the provider-specific URL patterns and include all required endpoints.
**Validates: Requirements 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7**

### Property 10: Configuration Inheritance Correctness
*For any* agent configuration with global defaults, the merged configuration should inherit unspecified values from defaults while preserving agent-specific overrides, maintaining type safety throughout the process.
**Validates: Requirements 7.2, 7.3, 7.4, 7.5**

### Property 11: Hierarchical Cache Fallback Chain
*For any* cache miss in a hierarchical setup, the system should follow the correct fallback order (local → root → direct), gracefully handling upstream failures by falling back to the next level without blocking client requests.
**Validates: Requirements 8.2, 8.3, 8.4, 8.6**

### Property 12: Circular Dependency Prevention
*For any* cache configuration, the system should detect and reject circular cache dependencies during validation, preventing infinite loops in the fallback chain.
**Validates: Requirements 8.5**

### Property 13: AX Subdomain URL Construction
*For any* domain, constructing AX endpoint URLs should correctly format the _agent.<domain> subdomain and /.well-known/agent-exchange.json path.
**Validates: Requirements 9.4**

### Property 14: HTTPS Certificate Validation
*For any* external AX fetch, HTTPS certificate chains should be properly validated, rejecting connections with invalid certificates.
**Validates: Requirements 9.5, 15.4**

### Property 15: LRU Cache Eviction Correctness
*For any* cache at maximum capacity, adding new entries should evict the least recently used entries while preserving the most recently accessed entries.
**Validates: Requirements 10.3**

### Property 16: Cache-Control Header Compliance
*For any* AX endpoint response with Cache-Control headers, the cache behavior should respect the directives (max-age, no-cache, etc.) when storing and serving cached records.
**Validates: Requirements 10.8**

### Property 17: DNS Cache Discovery Routing
*For any* client initialization, when a cache endpoint is found via DNS, discovery requests should use the cache; when no cache is found, requests should fall back to direct HTTPS discovery.
**Validates: Requirements 11.2, 11.3**

### Property 18: DNS TXT Record Parsing Correctness
*For any* DNS TXT record in the format "endpoint=<url>", the system should correctly extract and validate the cache endpoint URL.
**Validates: Requirements 11.4, 11.6**

### Property 19: DNS Discovery Error Resilience
*For any* DNS lookup failure during cache discovery, the system should continue with direct discovery without propagating DNS errors to the client.
**Validates: Requirements 11.5**

### Property 20: Configuration Validation Correctness
*For any* configuration input, the system should validate domain names (DNS format), port numbers (1-65535), URLs (HTTPS format), and TTL values (positive integers), rejecting invalid configurations.
**Validates: Requirements 13.4**

### Property 21: Rate Limiting Enforcement
*For any* client IP, when request rate limits are exceeded, the system should return HTTP 429 responses and block further requests until the rate limit window resets.
**Validates: Requirements 15.1, 15.3**

### Property 22: Request Timeout Handling
*For any* external AX fetch, requests should timeout after the configured duration (default 30 seconds) and return appropriate error responses.
**Validates: Requirements 15.5**

### Property 23: HTTP Error Response Mapping
*For any* error condition, the system should return appropriate HTTP status codes: 400 for invalid requests, 404 for not found, 429 for rate limiting, 502 for upstream failures, 503 for service unavailable.
**Validates: Requirements 15.6**

### Property 24: AX Version Validation and Handling
*For any* AX record, the system should validate that the version field is "1.0" and record_type is "AX", skipping records with unsupported versions while processing supported ones correctly.
**Validates: Requirements 18.1, 18.3**

### Property 25: Auth Method Validation
*For any* endpoint auth configuration, the system should validate that auth methods are from the supported set (OIDC, OAuth2, mTLS, JWT, API_KEY) and reject invalid auth method specifications.
**Validates: Requirements 9.3, 15.4**

### Property 26: Cache Size Limit Enforcement
*For any* cache configuration with max_entries limit, the cache should never exceed the specified number of entries, triggering LRU eviction when necessary.
**Validates: Requirements 10.3, 10.5**

## Testing Strategy

The testing strategy implements a dual approach combining unit tests and property-based tests for comprehensive validation:

### Unit Testing Framework

**Test Organization:**
- Each crate maintains focused unit tests in `tests/` directories
- Mock external dependencies using `mockall` for HTTP clients and DNS resolvers
- Test all error conditions and edge cases explicitly
- Validate configuration parsing and validation rules

**Key Unit Test Areas:**
- Configuration parsing and inheritance resolution
- OAuth provider template generation for each supported provider
- AX record validation against specification requirements
- Cache storage operations (put, get, evict, expire)
- DNS TXT record parsing and endpoint extraction
- HTTP client error handling and timeout behavior
- Policy enforcement logic for domain locking and allowlists

### Property-Based Testing Configuration

**Framework:** Uses `proptest` crate for property-based testing
**Test Configuration:** Minimum 100 iterations per property test
**Generator Strategy:** Smart generators that constrain inputs to valid domains

**Property Test Implementation:**
Each correctness property is implemented as a single property-based test with the following tag format:

```rust
// Feature: rust-toolkit-architecture, Property 1: AX Record Generation Correctness
#[proptest]
fn test_ax_record_generation_correctness(config in arb_valid_yaml_config()) {
    let generator = AxGenerator::new();
    let document = generator.generate_record(&config)?;
    
    // Should produce valid JSON
    let json = serde_json::to_string(&document)?;
    let parsed: AgentExchangeDocument = serde_json::from_str(&json)?;
    
    // Each record should follow AX 1.0 specification
    for record in &parsed.records {
        assert_eq!(record.record_type, "AX");
        assert_eq!(record.version, "1.0");
        AxValidator::validate_record(record)?;
    }
}

// Feature: rust-toolkit-architecture, Property 8: Serialization Round-Trip Consistency  
#[proptest]
fn test_serialization_round_trip_consistency(record in arb_ax_record()) {
    // Serialize then deserialize
    let json = serde_json::to_string(&record)?;
    let deserialized: AgentExchangeRecord = serde_json::from_str(&json)?;
    
    // Should preserve all fields including record_type and version
    assert_eq!(record.record_type, deserialized.record_type);
    assert_eq!(record.version, deserialized.version);
    assert_eq!(record.agent.name, deserialized.agent.name);
    assert_eq!(record.endpoints.len(), deserialized.endpoints.len());
}
```
```rust
// Valid domain generator
fn arb_domain() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-z][a-z0-9-]*\.[a-z]{2,}")
        .unwrap()
        .prop_filter("Valid domain", |s| s.len() <= 253)
}

// AX-compliant record generator
fn arb_ax_record() -> impl Strategy<Value = AgentExchangeRecord> {
    (
        arb_agent(),
        prop::collection::vec(arb_endpoint(), 1..5),
        prop::option::of(arb_capabilities()),
    ).prop_map(|(agent, endpoints, capabilities)| AgentExchangeRecord {
        record_type: "AX".to_string(),  // Always AX per RFC
        version: "1.0".to_string(),     // Always 1.0 for MVP
        agent,
        endpoints,
        capabilities,
        schema: None,
        limits: None,
        security: None,
        extensions: None,
    })
}

// Valid agent generator (no endpoints - they're at record level)
fn arb_agent() -> impl Strategy<Value = Agent> {
    (
        "[a-zA-Z][a-zA-Z0-9 -]{2,50}",  // name
        "[a-zA-Z0-9 .,!?-]{10,200}",    // description  
        "[a-zA-Z][a-zA-Z0-9 ]{2,50}",  // provider
    ).prop_map(|(name, description, provider)| Agent {
        name,
        description,
        provider,
    })
}

// Valid endpoint generator with proper auth methods
fn arb_endpoint() -> impl Strategy<Value = Endpoint> {
    (
        arb_protocol(),
        arb_https_url(),
        prop::collection::vec(arb_auth_method(), 1..3),
    ).prop_map(|(protocol, url, auth)| Endpoint {
        protocol,
        url,
        auth,
        content_type: Some("application/json".to_string()),
    })
}

// Valid auth methods per AX RFC
fn arb_auth_method() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "OIDC".to_string(),
        "OAuth2".to_string(),
        "mTLS".to_string(),
        "JWT".to_string(),
        "API_KEY".to_string(),
    ])
}

// Valid YAML configuration generator
fn arb_valid_yaml_config() -> impl Strategy<Value = AgentConfig> {
    (
        prop::option::of(arb_agent_defaults()),
        prop::collection::vec(arb_agent_definition(), 1..10),
    ).prop_map(|(defaults, agents)| AgentConfig { defaults, agents })
}
```

### Integration Testing

**Multi-Component Workflows:**
- CLI generation → Core validation → Server caching integration
- DNS discovery → Cache coordination → HTTP fetching workflows
- Configuration inheritance → OAuth generation → AX record creation

**Network Simulation:**
- Mock HTTP servers using `wiremock` for testing discovery flows
- Simulate DNS responses for cache discovery testing
- Test timeout and error conditions with controlled network failures

**Test Environment Setup:**
```rust
// Integration test helper
pub struct TestEnvironment {
    mock_dns: MockDnsResolver,
    mock_http: MockServer,
    temp_dir: TempDir,
    cache_server: CacheServer,
}

impl TestEnvironment {
    pub async fn setup() -> Self {
        // Setup mock services and temporary directories
        // Configure test cache server with test configuration
        // Return ready-to-use test environment
    }
    
    pub async fn test_full_discovery_flow(&self, domain: &str) -> Result<AgentExchangeRecord, TestError> {
        // Test complete flow from DNS discovery through cache to AX fetch
    }
}
```

### Performance Testing

**Benchmarking Framework:** Uses `criterion` crate for performance benchmarks
**Key Performance Areas:**
- Cache lookup and storage operations
- AX record parsing and validation
- Configuration inheritance resolution
- DNS lookup and HTTP request performance

**Benchmark Examples:**
```rust
fn bench_cache_operations(c: &mut Criterion) {
    let cache = MemoryCache::new(CacheConfig::default());
    let record = create_test_ax_record();
    
    c.bench_function("cache_put", |b| {
        b.iter(|| cache.put("example.com", record.clone(), Duration::from_secs(3600)))
    });
    
    c.bench_function("cache_get", |b| {
        b.iter(|| cache.get("example.com"))
    });
}
```

The testing strategy ensures comprehensive validation of all correctness properties while maintaining fast feedback cycles for development and reliable detection of regressions.