# Open-Tethyr Feature Guide

## Overview

Open-tethyr is a toolkit for the AX (Agent Discovery Exchange) protocol. It lets organizations publish, discover, cache, and validate agent metadata across domains using a standardized JSON format served at well-known URIs.

## Components

### 1. Core Library (`open-tethyr`)

A Rust library you can embed in your own applications.

**Feature flags:**

| Flag | Default | What it includes |
|------|---------|-----------------|
| `client` | Yes | `OpenTethyr` SDK for agent discovery |
| `server` | No | `CacheServer` with axum, policy engine, rate limiting |
| `full` | No | Both `client` + `server` |

**Usage in Cargo.toml:**

```toml
# Lightweight client only (default)
open-tethyr = "0.1"

# Full server + client
open-tethyr = { version = "0.1", features = ["full"] }
```

### 2. CLI Binary (`open-tethyr`)

A single binary with five subcommands:

| Command | Purpose |
|---------|---------|
| `generate` | Create AX records from YAML config |
| `validate` | Check AX records for protocol compliance |
| `discover` | Test agent discovery against a domain |
| `serve` | Run the cache server |
| `cache-invalidate` | Clear cached entries on a running server |

---

## Feature Details

### Generate AX Records

Create a YAML configuration file describing your agents, then generate the AX 1.0 JSON document that goes at `/.well-known/agent-exchange.json`.

```bash
open-tethyr generate --config agents.yaml --output ./public --validate
```

**What it does:**
- Reads YAML config with global defaults + per-agent overrides
- Merges configuration (agent-specific values override defaults)
- Generates OAuth endpoint URLs if a provider template is specified
- Writes `/.well-known/agent-exchange.json` to the output directory
- Optionally validates every generated record against AX 1.0

**Config inheritance:** Agent fields that are empty fall back to `defaults`. Agent fields that are set override defaults completely (no deep merge on arrays).

### Validate AX Records

Check any AX JSON file for protocol compliance.

```bash
open-tethyr validate ./public/.well-known/agent-exchange.json
```

**What it checks:**
- `record_type` must be `"AX"`
- `version` must be `"1.0"` (other versions produce a warning)
- `agent.name`, `agent.description`, `agent.provider` must be non-empty
- At least one endpoint required
- Each endpoint must have a URL and auth methods
- Auth methods must be from: `OIDC`, `OAuth2`, `mTLS`, `JWT`, `API_KEY`

**Exit codes:** 0 = valid, 3 = validation errors found.

### Discover Agents

Test discovery against a live domain.

```bash
# Auto-discover (tries DNS cache first, then direct)
open-tethyr discover example.com

# Force direct fetch (skip cache)
open-tethyr discover example.com --direct

# Use a specific cache server
open-tethyr discover example.com --cache https://cache.internal:8080

# JSON output
open-tethyr discover example.com --json
```

**Discovery flow:**
1. Check DNS for `_ax-cache.<domain>` TXT record
2. If found, route through the cache server
3. If not found (or cache fails), fetch directly from `https://_agent.<domain>/.well-known/agent-exchange.json`

### Cache Server

Run a high-performance caching proxy for agent discovery.

```bash
# Basic startup
open-tethyr serve --port 8080

# With config file
open-tethyr serve --config server.yaml

# With explicit settings
open-tethyr serve --domain acme.com --port 8080 --max-entries 10000 --ttl 3600
```

**Endpoints:**

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/discover/:domain` | Discover agents (cache-first) |
| GET | `/health` | Health check (returns JSON status) |
| GET | `/metrics` | Prometheus-format metrics |

**Features:**
- **In-memory LRU cache** with configurable max entries and TTL
- **Hierarchical caching**: local cache -> root cache (via DNS) -> direct fetch
- **Domain locking**: restrict discovery to home domain + allowlist
- **Rate limiting**: per-client-IP token bucket
- **Correlation IDs**: every response includes `X-Correlation-Id` header
- **Cache-Control compliance**: respects `no-cache` directives

### Cache Invalidation

Clear entries on a running cache server.

```bash
# Invalidate one domain
open-tethyr cache-invalidate --domain example.com --server http://localhost:8080

# Clear entire cache
open-tethyr cache-invalidate --all --server http://localhost:8080
```

### OAuth Provider Templates

Built-in templates generate correct OAuth endpoint URLs for:

| Provider | Config value | Endpoints generated |
|----------|-------------|-------------------|
| Okta | `okta` | `/oauth2/authorize`, `/oauth2/token`, `/oauth2/v1/keys`, etc. |
| Auth0 | `auth0` | `/authorize`, `/oauth/token`, `/.well-known/jwks.json`, etc. |
| Generic | `generic` | RFC 8414 standard paths (`/authorize`, `/token`, etc.) |

**Usage in YAML config:**

```yaml
agents:
  - name: my-agent
    description: My agent
    oauth_provider: okta
    oauth_domain: dev-123456.okta.com
    endpoints:
      - protocol: rest
        url: https://api.example.com/v1
        auth: [OIDC]
```

The generated AX record will include a `security.oauth` block with all provider-specific endpoint URLs.

---

## Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `OPEN_TETHYR_LOG` | Log level filter (`error`, `warn`, `info`, `debug`, `trace`) | `info` |

**Configuration precedence:** CLI arguments > environment variables > config file > defaults.

## Metrics

The `/metrics` endpoint returns Prometheus text format:

```
cache_hits_total 1234
cache_misses_total 56
cache_evictions_total 7
```
