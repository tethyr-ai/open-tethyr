# Open-Tethyr Setup Guide

How to set up open-tethyr for your domain from scratch.

## Prerequisites

- A domain you control (e.g. `acme.com`)
- A web server where you can host static files (for the AX record)
- DNS access (if you want cache discovery)
- Rust toolchain (if building from source) or a pre-built binary

## Step 1: Install

**From source:**

```bash
git clone https://github.com/tethyr-ai/open-tethyr.git
cd open-tethyr
cargo build --release
# Binary is at target/release/open-tethyr
```

**Or use Make:**

```bash
make build-release
```

## Step 2: Create Your Agent Configuration

Create a file called `agents.yaml`:

```yaml
defaults:
  provider: "Acme Corp"
  auth:
    - "OAuth2"

agents:
  - name: "billing-agent"
    description: "Handles billing inquiries and payment processing"
    endpoints:
      - protocol: "rest"
        url: "https://billing.acme.com/api/v1"
        auth:
          - "OAuth2"
          - "API_KEY"
        content_type: "application/json"
    oauth_provider: "okta"
    oauth_domain: "acme.okta.com"

  - name: "support-agent"
    description: "Customer support and ticket management"
    provider: "Support Inc"  # overrides default provider
    endpoints:
      - protocol: "rest"
        url: "https://support.acme.com/api/v2"
        auth:
          - "JWT"
      - protocol: "graphql"
        url: "https://support.acme.com/graphql"
        auth:
          - "OAuth2"
```

**Key points:**
- `defaults.provider` and `defaults.auth` are inherited by all agents
- Each agent can override any default
- `oauth_provider` + `oauth_domain` auto-generates OAuth endpoint URLs
- Valid auth methods: `OIDC`, `OAuth2`, `mTLS`, `JWT`, `API_KEY`

## Step 3: Generate AX Records

```bash
./target/release/open-tethyr generate \
  --config agents.yaml \
  --output ./public \
  --validate
```

This creates:

```
public/
  .well-known/
    agent-exchange.json    <-- your AX record
```

**Verify the output:**

```bash
cat public/.well-known/agent-exchange.json | python3 -m json.tool
```

You should see something like:

```json
{
  "records": [
    {
      "record_type": "AX",
      "version": "1.0",
      "agent": {
        "name": "billing-agent",
        "description": "Handles billing inquiries and payment processing",
        "provider": "Acme Corp"
      },
      "endpoints": [
        {
          "protocol": "rest",
          "url": "https://billing.acme.com/api/v1",
          "auth": ["OAuth2", "API_KEY"],
          "content_type": "application/json"
        }
      ],
      "security": {
        "oauth": {
          "issuer": "https://acme.okta.com",
          "authorization_endpoint": "https://acme.okta.com/oauth2/authorize",
          "token_endpoint": "https://acme.okta.com/oauth2/token",
          "jwks_uri": "https://acme.okta.com/oauth2/v1/keys",
          "userinfo_endpoint": "https://acme.okta.com/oauth2/v1/userinfo",
          "revocation_endpoint": "https://acme.okta.com/oauth2/v1/revoke"
        }
      }
    }
  ]
}
```

## Step 4: Validate

Double-check your record is AX 1.0 compliant:

```bash
./target/release/open-tethyr validate public/.well-known/agent-exchange.json
```

Expected output:

```
Validation PASSED: 2 record(s) valid
```

## Step 5: Deploy the AX Record

Copy `public/.well-known/agent-exchange.json` to your web server so it's accessible at:

```
https://_agent.acme.com/.well-known/agent-exchange.json
```

**Option A: Nginx**

```nginx
server {
    listen 443 ssl;
    server_name _agent.acme.com;

    location /.well-known/agent-exchange.json {
        root /var/www/acme;
        add_header Content-Type application/json;
        add_header Access-Control-Allow-Origin *;
    }
}
```

**Option B: S3 + CloudFront**

Upload `agent-exchange.json` to an S3 bucket served at `_agent.acme.com/.well-known/`.

**Option C: Any static hosting**

Serve the file at the well-known path with `Content-Type: application/json`.

**DNS:** Create a DNS record for `_agent.acme.com` pointing to your web server.

## Step 6: Test Discovery

From any machine:

```bash
./target/release/open-tethyr discover acme.com
```

Expected output:

```
Discovered 2 agent(s) for acme.com:
  billing-agent - Handles billing inquiries and payment processing (provider: Acme Corp)
    Rest https://billing.acme.com/api/v1
  support-agent - Customer support and ticket management (provider: Support Inc)
    Rest https://support.acme.com/api/v2
    GraphQL https://support.acme.com/graphql
```

## Step 7 (Optional): Deploy a Cache Server

If you want faster discovery and organizational control:

### 7a. Start the cache server

```bash
./target/release/open-tethyr serve \
  --domain acme.com \
  --port 8080 \
  --max-entries 10000 \
  --ttl 3600
```

Or with a config file:

```yaml
# server.yaml
server:
  domain: acme.com
  port: 8080
  cache:
    max_entries: 10000
    default_ttl: 3600
  policy:
    domain_locking: true
    home_domain: acme.com
    allowlist:
      - partner.com
      - trusted.io
  rate_limit:
    requests_per_minute: 60
```

```bash
./target/release/open-tethyr serve --config server.yaml
```

### 7b. Set up DNS cache discovery

Create a DNS TXT record:

```
_ax-cache.acme.com  TXT  "endpoint=https://cache.acme.com:8080"
```

Now any client using `open-tethyr discover acme.com` will automatically route through your cache.

### 7c. Verify the cache is working

```bash
# Health check
curl http://localhost:8080/health

# Discover through cache
curl http://localhost:8080/discover/acme.com

# Check metrics
curl http://localhost:8080/metrics

# Invalidate a cached entry
./target/release/open-tethyr cache-invalidate --domain acme.com --server http://localhost:8080
```

## Step 8 (Optional): Use the Rust SDK

If you're building a Rust application that needs to discover agents:

```toml
# Cargo.toml
[dependencies]
open-tethyr = "0.1"
tokio = { version = "1", features = ["full"] }
```

```rust
use open_tethyr::OpenTethyr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Auto-discovers cache via DNS, falls back to direct
    let client = OpenTethyr::with_dns_discovery("acme.com").await?;
    let doc = client.discover("partner.com").await?;

    for record in &doc.records {
        println!("{} - {}", record.agent.name, record.agent.description);
    }
    Ok(())
}
```

## Step 9 (Optional): Docker Deployment

```bash
# Build the image
docker build -t open-tethyr:latest -f docker/Dockerfile .

# Run the cache server
docker run -d \
  --name open-tethyr-cache \
  -p 8080:8080 \
  open-tethyr:latest \
  serve --domain acme.com --port 8080
```

---

## Quick Reference

| Task | Command |
|------|---------|
| Generate records | `open-tethyr generate -c agents.yaml -o ./public --validate` |
| Validate a file | `open-tethyr validate path/to/agent-exchange.json` |
| Test discovery | `open-tethyr discover acme.com` |
| Start cache server | `open-tethyr serve --port 8080` |
| Clear cache | `open-tethyr cache-invalidate --all --server http://localhost:8080` |
| Check server health | `curl http://localhost:8080/health` |
| View metrics | `curl http://localhost:8080/metrics` |

## Troubleshooting

**"No agents found"** - Check that `_agent.<domain>` resolves and the well-known path returns valid JSON.

**Validation errors** - Run `open-tethyr validate` to see exactly which fields fail. Common issues: empty agent name, missing auth methods, using an unapproved auth method.

**Cache not discovered** - Verify the `_ax-cache.<domain>` TXT record is published: `dig TXT _ax-cache.acme.com`

**Rate limited (HTTP 429)** - The cache server limits requests per IP. Wait for the token bucket to refill, or increase `requests_per_minute` in config.

**Policy violation (HTTP 403)** - Domain locking is enabled and you're requesting a domain not on the allowlist. Add it to `policy.allowlist` or disable `domain_locking`.
