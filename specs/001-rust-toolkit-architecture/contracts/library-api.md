# Library API Contract: open-tethyr

**Crate name**: `open-tethyr`
**Features**: `client` (default), `server` (opt-in), `full` (both)

## Public API Surface

### Client SDK (feature = "client", default)

```rust
use open_tethyr::OpenTethyr;

// Initialize client with automatic DNS cache discovery
let client = OpenTethyr::new("acme.com")?;

// Discover agents from a target domain
let document: AgentExchangeDocument = client.discover("partner.com").await?;

// Discover using a specific cache URL
let document = client.discover_with_cache("partner.com", "https://cache.acme.com:8080").await?;
```

**OpenTethyr**:
- `new(domain: &str) -> Result<Self, ClientError>` - Initialize with domain, auto-discover cache via DNS
- `discover(target_domain: &str) -> Result<AgentExchangeDocument, ClientError>` - Discover agents (cache-first, fallback to direct)
- `discover_with_cache(target_domain: &str, cache_url: &str) -> Result<AgentExchangeDocument, ClientError>` - Discover using specific cache

### Core Types (always available)

```rust
use open_tethyr::ax::{
    AgentExchangeRecord, AgentExchangeDocument, Agent, Endpoint, Protocol,
    Capabilities, Schema, Limits, Security,
};
use open_tethyr::ax::{AxValidator, AxGenerator};
use open_tethyr::config::{AgentConfig, ConfigMerger, ConfigValidator};
use open_tethyr::auth::{OAuthProvider, ProviderRegistry, OAuthEndpoints};
use open_tethyr::dns::DnsDiscovery;
use open_tethyr::http::AxHttpClient;
use open_tethyr::cache::{MemoryCache, CacheConfig, CacheStats};
```

**AxValidator**:
- `validate_record(record: &AgentExchangeRecord) -> Result<(), ValidationError>` - Full AX 1.0 validation
- `validate_agent(agent: &Agent) -> Result<(), ValidationError>` - Validate agent fields
- `validate_version(version: &str) -> Result<(), ValidationError>` - Check version is "1.0"
- `validate_endpoints(endpoints: &[Endpoint]) -> Result<(), ValidationError>` - Validate endpoint list
- `validate_auth_methods(auth: &[String]) -> Result<(), ValidationError>` - Check auth method names

**AxGenerator**:
- `generate_record(config: &AgentConfig) -> Result<AgentExchangeDocument, GenerationError>` - Generate AX document from config
- `generate_well_known_structure(document: &AgentExchangeDocument) -> Result<WellKnownFiles, GenerationError>` - Generate file structure

**ConfigMerger**:
- `merge_agent(agent: &AgentDefinition, defaults: &AgentDefaults) -> Agent` - Merge agent with defaults
- `merge_auth(agent_auth: &Option<AuthConfig>, default_auth: &Option<AuthConfig>) -> Option<AuthConfig>` - Merge auth configs

**ConfigValidator**:
- `validate_config(config: &AgentConfig) -> Result<(), ValidationError>` - Full config validation
- `validate_domain(domain: &str) -> Result<(), ValidationError>` - DNS format check
- `validate_url(url: &str) -> Result<(), ValidationError>` - HTTPS URL check
- `validate_port(port: u16) -> Result<(), ValidationError>` - Port range check

**DnsDiscovery**:
- `discover_cache(domain: &str) -> Result<Option<String>, DnsError>` - Lookup `_ax-cache.{domain}` TXT
- `discover_root_cache(domain: &str) -> Result<Option<String>, DnsError>` - Lookup root cache

**AxHttpClient**:
- `new(timeout: Duration) -> Self` - Create with configurable timeout
- `fetch_ax_record(domain: &str) -> Result<AgentExchangeDocument, HttpError>` - Fetch from `_agent.{domain}`
- `fetch_from_cache(cache_url: &str, domain: &str) -> Result<AgentExchangeDocument, HttpError>` - Fetch via cache

**MemoryCache**:
- `get(domain: &str) -> Option<AgentExchangeDocument>` - Lookup cached record
- `put(domain: &str, document: AgentExchangeDocument, ttl: Duration)` - Store with TTL
- `invalidate(domain: &str) -> bool` - Remove specific entry
- `clear()` - Remove all entries
- `size() -> usize` - Current entry count
- `stats() -> CacheStats` - Cache statistics

### Server (feature = "server")

```rust
use open_tethyr::CacheServer;
use open_tethyr::server::{ServerConfig, PolicyEngine, CacheMetrics};

let server = CacheServer::new(config).await?;
server.start().await?;
```

**CacheServer**:
- `new(config: ServerConfig) -> Result<Self, ServerError>` - Initialize server with config
- `start() -> Result<(), ServerError>` - Start listening and serving requests

### OAuth Provider Templates

```rust
use open_tethyr::auth::{ProviderRegistry, OAuthConfig};

let registry = ProviderRegistry::new(); // Pre-loaded with Okta, Auth0, Generic
let endpoints = registry.generate_oauth_config("okta", &config)?;
```

**ProviderRegistry**:
- `new() -> Self` - Create with MVP providers pre-registered
- `register_provider(name: String, provider: Box<dyn OAuthProvider>)` - Add custom provider
- `generate_oauth_config(provider: &str, config: &OAuthConfig) -> Result<OAuthEndpoints, OAuthError>` - Generate endpoints

## Error Types

```rust
// Core errors
pub enum AxError {
    Validation { message: String },
    UnsupportedVersion { version: String },
    Http(reqwest::Error),
    Dns(ResolveError),
    Serialization(serde_json::Error),
}

// Server errors (feature = "server")
pub enum ServerError {
    PolicyViolation { message: String },
    RateLimitExceeded { client_ip: String },
    Cache(CacheError),
    Config(ConfigError),
}

// Client errors (feature = "client")
pub enum ClientError {
    Discovery { domain: String, source: Box<dyn Error> },
    DnsLookup(DnsError),
    Http(HttpError),
}
```

## Feature Flag Behavior

| Feature | Modules Available | Dependencies Added |
|---------|------------------|--------------------|
| (none) | ax, cache, config, dns, http, auth | serde, reqwest, trust-dns, lru |
| client (default) | + client (OpenTethyr) | (none additional) |
| server | + server (CacheServer) | axum, tower, tower-http |
| full | All modules | All dependencies |
