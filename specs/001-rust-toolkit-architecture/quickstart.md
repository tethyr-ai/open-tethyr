# Quickstart: Open-Tethyr Rust Toolkit

**Branch**: `001-rust-toolkit-architecture` | **Date**: 2026-03-12

## Prerequisites

- Rust stable toolchain (rustup recommended)
- Docker (optional, for containerized deployment)

## Build from Source

```bash
# Clone and build
git clone https://github.com/your-org/open-tethyr.git
cd open-tethyr
cargo build --release

# The CLI binary is at target/release/open-tethyr
```

## Install CLI

```bash
cargo install --path crates/cli
# Installs 'open-tethyr' binary to ~/.cargo/bin/
```

## 1. Generate AX Records

Create a YAML configuration file (`agents.yaml`):

```yaml
defaults:
  provider: "Acme Corp"
  capabilities:
    async: true

agents:
  - name: "Customer Lookup"
    description: "Query customer database by ID or email"
    endpoints:
      - protocol: rest
        url: "https://api.acme.com/agents/customer-lookup"
        auth: ["OAuth2"]
        content_type: "application/json"
    security:
      oauth:
        issuer: "https://auth.acme.com"
        authorization_endpoint: "https://auth.acme.com/oauth2/authorize"
        token_endpoint: "https://auth.acme.com/oauth2/token"
        jwks_uri: "https://auth.acme.com/oauth2/v1/keys"
        userinfo_endpoint: "https://auth.acme.com/oauth2/v1/userinfo"
        revocation_endpoint: "https://auth.acme.com/oauth2/v1/revoke"

  - name: "Order Status"
    description: "Check order status and tracking information"
    endpoints:
      - protocol: rest
        url: "https://api.acme.com/agents/order-status"
        auth: ["API_KEY"]
```

Generate the AX records:

```bash
open-tethyr generate --config agents.yaml --output ./public --validate
# Creates: ./public/.well-known/agent-exchange.json
```

## 2. Validate AX Records

```bash
# Validate a generated or received AX record
open-tethyr validate ./public/.well-known/agent-exchange.json
```

## 3. Start a Cache Server

```bash
# Start cache server on port 8080
open-tethyr serve --domain acme.com --port 8080

# With a config file for advanced options
open-tethyr serve --config server.yaml
```

Example `server.yaml`:

```yaml
domain: acme.com
port: 8080
cache:
  max_entries: 10000
  default_ttl: 3600  # 1 hour
policy:
  domain_locking: true
  home_domain: acme.com
  allowlist:
    - partner.com
    - vendor.com
rate_limit:
  requests_per_minute: 60
  requests_per_hour: 1000
log_level: info
```

## 4. Test Discovery

```bash
# Direct discovery (no cache)
open-tethyr discover api.partner.com

# Discovery through a cache server
open-tethyr discover api.partner.com --cache https://cache.acme.com:8080
```

## 5. Use as a Rust Library (SDK)

Add to your `Cargo.toml`:

```toml
[dependencies]
open-tethyr = "0.1"  # Default: lightweight client SDK only
```

Use in your application:

```rust
use open_tethyr::OpenTethyr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client - auto-discovers cache via DNS
    let client = OpenTethyr::new("acme.com")?;

    // Discover agents from a partner domain
    let agents = client.discover("api.partner.com").await?;

    for record in &agents.records {
        println!("Agent: {} - {}", record.agent.name, record.agent.description);
        for endpoint in &record.endpoints {
            println!("  Endpoint: {} ({})", endpoint.url, endpoint.protocol);
        }
    }

    Ok(())
}
```

## 6. DNS Setup (for cache discovery)

To enable automatic cache discovery, add a DNS TXT record:

```
_ax-cache.acme.com  TXT  "endpoint=https://cache.acme.com:8080"
```

Clients initializing with domain "acme.com" will automatically discover and use this cache.

## 7. Docker Deployment

```bash
# Build Docker image
docker build -t open-tethyr -f docker/Dockerfile .

# Run cache server
docker run -p 8080:8080 open-tethyr serve --domain acme.com --port 8080
```

## Development

```bash
# Run all tests
cargo test --workspace

# Run property-based tests (more iterations)
PROPTEST_CASES=1000 cargo test --workspace

# Run benchmarks
cargo bench --workspace

# Check formatting and lints
cargo fmt --check --all
cargo clippy --workspace --all-features
```
