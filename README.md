# open-tethyr

**Tethering agents through open discovery**

*Distributed cache for AX agent discovery with DNS-based cache coordination*

## Overview

open-tethyr is a distributed caching layer for agent discovery implementing the [Agent Discovery Exchange (AX) protocol](https://github.com/sempfa/agent-discovery-exchange). It provides lazy caching and infrastructure-level control over how agents are discovered and accessed across your organization.

Built on the principle that discovery should be boring, standardized infrastructure - following the AX RFC spec's use of HTTPS and well-known URIs - while giving you complete control over your agent ecosystem.

## Why Discovery Control Matters

As AI agents become critical infrastructure, organizations need control over:

- Discovery governance: which agents can be discovered by your teams and systems
- Access policies: what external agents your organization interacts with  
- Compliance: audit trails of agent discovery and usage patterns
- Security boundaries: prevent unauthorized agent access or data exposure
- Performance: cache agent metadata close to your users and workloads
- Operational resilience: reduce dependencies on external discovery services

The problem with centralized marketplaces is you lose control over discovery, compliance, and security. Your agent interactions are visible to marketplace operators, and you're dependent on their availability and policies.

The open-tethyr approach: your infrastructure, your policies, your control. Use standard HTTPS endpoints to discover agents, cache what matters to your organization, enforce your security boundaries.

## What is open-tethyr?

open-tethyr is the infrastructure layer for controlled agent discovery. It generates AX-compliant discovery records from simple configuration, provides caching for AX agent lookup using well-known HTTPS endpoints, caches discovered agents with client-domain scoping for performance, enforces discovery policies at the cache layer (domain locking, allowlists), and serves as a reference implementation of distributed AX caching.

## What open-tethyr Does NOT Do

- **No built-in web server**: open-tethyr generates AX files - you publish them at the standard `/.well-known/agent-exchange.json` location via S3, nginx, CDN, or any infrastructure you choose
- **Not a marketplace**: no transactions, billing, or UI - pure discovery infrastructure that you control
- **No crawling**: no proactive scanning - caches only fetch agents when your clients request them
- **Discovery only**: how agents authenticate and execute is up to publishers and clients
- **No centralized service**: every organization runs their own infrastructure with zero vendor lock-in

## Core Principle

**Discovery is infrastructure, not a product.**

> The web didn't scale because someone built "the website exchange" - it scaled because discovery was standardized (DNS, HTTP) and open. Agent ecosystems need the same foundation. -- Aaron Sempf ([LinkedIn](https://www.linkedin.com/posts/aaron-sempf_discovery-is-infrastructure-why-agents-need-activity-7410822859855601664-7I1u/))

open-tethyr implements the AX protocol's standard discovery mechanism (HTTPS + well-known URIs) while giving you complete control over discovery policies, caching strategies, and security boundaries.

## Discovery Control Features

### Policy Enforcement

Restrict discovery to internal agents only with domain locking:
```yaml
cache:
  lock_to_home_domain: true  # Only discover *.acme.com agents
```

Explicitly approve partner agents with external allowlists (coming in future release):
```yaml
cache:
  allowed_external_domains:
    - trusted-partner.com
    - approved-vendor.io
```

Your cache enforces what can be discovered. Unauthorized external agents are blocked at the infrastructure level.

### Centralized Visibility & Observability

All discovery requests flow through your infrastructure, giving you cache hit rates that reduce external dependencies, visibility into what agents your teams are discovering, and audit trails of all agent discovery patterns.

Performance benefits include sub-millisecond cache responses versus external HTTPS fetches, the ability to work offline with cached agents, and reduced bandwidth for frequently accessed agents.

### Security Through Isolation

Clients always use their own domain's cache and never trust external cache infrastructure. Your cache, your data, your policies.

Discovery metadata is public (what agents exist, what they do), but execution credentials remain private (OAuth tokens, API keys). The cache never sees or stores execution secrets.

### Compliance & Governance

Log all agent discovery requests, track which teams discover which external agents, identify shadow AI usage patterns, and support compliance reporting requirements.

Your cache runs in your infrastructure (on-prem, VPC, cloud account). Agent metadata is stored in your geography with no data shared with third parties.

## Architecture

### AX Protocol Discovery Mechanism

Following the AX RFC specification, agent discovery uses standard HTTPS and well-known URIs. The agent discovery endpoint is `https://_agent.<domain>/.well-known/agent-exchange.json`.

For example: `https://_agent.api.acme.com/.well-known/agent-exchange.json`

No custom DNS record types are required - it uses standard HTTPS infrastructure.

### Cache Discovery (open-tethyr extension)

For performance and control, open-tethyr adds optional DNS-based cache discovery. Cache discovery uses `_ax-cache.<domain>` DNS TXT records to advertise your cache servers. Clients always use their own domain's cache and never trust external caches.

You control the caching infrastructure. Standard AX discovery works without caching, but caching provides performance, observability, and policy enforcement.

### Simple Cache Model

Each cache is independent with client-domain scoping (clients at acme.com use acme.com cache), lazy loading (only fetches agents when requested), optional root cache via DNS for hierarchical caching, and a fallback chain: regional cache → root cache → direct HTTPS fetch.

No cache-to-cache propagation. No distributed coordination. KISS.

Deploy caches where you need them - regional offices, cloud regions, on-prem data centers - with simple DNS-based coordination for cache discovery.

## How It Works

### Publishing Your Agents

**Step 1**: Define agents in configuration
```yaml
# agents.yaml
agents:
  - name: Customer Lookup
    description: Query customer database
    provider: Acme Corp
    
    endpoints:
      - protocol: rest
        url: https://api.acme.com/agents/customer-lookup
        auth:
          methods: [OAuth2]
```

**Step 2**: Generate AX records
```bash
open-tethyr generate --config agents.yaml --output ./ax-records
```

This outputs `/.well-known/agent-exchange.json` (AX-compliant discovery record at standard location) and individual agent metadata files (optional).

**Step 3**: Publish via your infrastructure

Publish the generated files so they're accessible at `https://_agent.<your-domain>/.well-known/agent-exchange.json`:
```bash
# Option 1: Static hosting (S3 with CloudFront, etc.)
aws s3 sync ./ax-records s3://yourbucket/
# Configure your domain: _agent.api.acme.com → CloudFront → S3

# Option 2: Serve via nginx/Apache
# Configure subdomain: _agent.api.acme.com
cp -r ./ax-records /var/www/_agent.api.acme.com/

# Option 3: API Gateway / serverless function
# Serve /.well-known/agent-exchange.json from Lambda/Cloud Function
```

Your agents are now discoverable via the AX protocol at `https://_agent.api.acme.com/.well-known/agent-exchange.json`.

### Running a Cache Server

Start the cache:
```bash
open-tethyr serve --mode cache --domain acme.com
```

Add a DNS record (optional for cache discovery):
```
_ax-cache.acme.com TXT "endpoint=http://cache.acme.com:8080"
```

Clients at `*.acme.com` auto-discover the cache via DNS lookup of `_ax-cache.acme.com`. When a client requests agents from any domain (e.g., `api.partner.com`), the cache checks local storage for cached records. On a cache miss, it fetches `https://_agent.api.partner.com/.well-known/agent-exchange.json` and returns the cached results to the client.

For hierarchical caching, the cache startup automatically checks DNS for `_ax-cache.{domain}`. If a root cache is found, it uses it as a fallback for cache misses. If not found, it operates standalone. The fallback chain is: Regional cache → Root cache → Direct HTTPS fetch.

### Discovering Agents

Client SDKs are available in multiple languages. Each SDK is initialized with your home domain and automatically discovers your cache via DNS.

**TypeScript/JavaScript:**

Repository: [open-tethyr-ts](https://github.com/tethyr/open-tethyr-ts)
```bash
npm install @open-tethyr/sdk
```
```typescript
import { OpenTethyr } from '@open-tethyr/sdk';

// Initialize with your domain
const client = new OpenTethyr({ domain: 'acme.com' });

// Discover agents - automatically uses acme.com cache (if configured)
// Falls back to direct HTTPS fetch if no cache
const agents = await client.discover('api.partner.com');

console.log(agents);
// [
//   { name: 'customer-lookup', endpoint: '...', ... },
//   { name: 'order-processing', endpoint: '...', ... }
// ]
```

**Python:**

Repository: [open-tethyr-py](https://github.com/tethyr/open-tethyr-py)
```bash
pip install open-tethyr
```
```python
from open_tethyr import OpenTethyr

# Initialize with your domain
client = OpenTethyr(domain='acme.com')

# Discover agents - automatically uses acme.com cache (if configured)
agents = client.discover('api.partner.com')
print(agents)
```

**Rust:**

The Rust SDK is included in this repository as a library crate.
```bash
cargo add open-tethyr
```
```rust
use open_tethyr::OpenTethyr;

// Initialize with your domain
let client = OpenTethyr::new("acme.com")?;

// Discover agents - automatically uses acme.com cache (if configured)
let agents = client.discover("api.partner.com")?;
println!("{:?}", agents);
```

**Go:**

Repository: [open-tethyr-go](https://github.com/tethyr/open-tethyr-go) _(coming soon)_

### How Open-Tethyr works

When a client at acme.com discovers agents at api.partner.com:

1. The SDK initializes with the home domain (`const client = new OpenTethyr({ domain: 'acme.com' });`), storing `acme.com` and performing a DNS lookup for `_ax-cache.acme.com`. If found, it uses the cache endpoint `http://cache.acme.com:8080`. If not found, it uses direct discovery mode.

2. The client requests discovery: `client.discover('api.partner.com')`

3. If a cache is configured, the SDK queries the cache for agents from `api.partner.com`

4. The cache checks local storage

5. On a cache miss, it fetches `https://_agent.api.partner.com/.well-known/agent-exchange.json` and stores it in cache with TTL

6. If no cache is configured, the SDK directly fetches `https://_agent.api.partner.com/.well-known/agent-exchange.json`

7. The SDK returns agents to the client

Discovery works with or without caching (AX protocol standard). Caching adds performance and control (open-tethyr extension). Clients always use their own domain's cache for security through isolation.

## Components

### CLI Tool

Generate AX records:
```bash
open-tethyr generate --config agents.yaml --output ./ax-records
```

Validate AX records:
```bash
open-tethyr validate --file ./ax-records/.well-known/agent-exchange.json
```

Test discovery:
```bash
# Test with cache
open-tethyr discover api.partner.com \
  --cache http://cache.acme.com:8080

# Test direct (no cache)
open-tethyr discover api.partner.com
```

### Cache Server

Cache-only mode:
```bash
open-tethyr serve --mode cache --domain acme.com --port 8080
```

With root cache fallback (auto-discovered via DNS):
```bash
open-tethyr serve --mode cache --domain acme.com
# Checks _ax-cache.acme.com for root, uses as fallback if found
```

Configuration file:
```bash
open-tethyr serve --config cache-config.yaml
```
```yaml
# cache-config.yaml
cache:
  domain: acme.com
  port: 8080
  ttl: 3600                    # Cache TTL in seconds
  max_size: 1000               # Maximum cached entries
  lock_to_home_domain: false   # Only cache agents from home domain
  storage:
    type: memory               # memory, redis, or disk
```

### Client SDKs

Each SDK provides a consistent API initialized with your home domain:

- **TypeScript/JavaScript**: `@open-tethyr/sdk` - [Repository](https://github.com/tethyr/open-tethyr-ts)
- **Python**: `open-tethyr` - [Repository](https://github.com/tethyr/open-tethyr-py)  
- **Rust**: `open-tethyr` - Included in this repository
- **Go**: Coming soon - [Repository](https://github.com/tethyr/open-tethyr-go)

All SDKs automatically discover and use your domain's cache via DNS (if configured), or fall back to direct HTTPS discovery.

## Example Deployments

### Corporate Setup

Configure a corporate cache:
```
_ax-cache.corp.acme.com TXT "endpoint=http://cache.corp.acme.com"
```

Deploy cache server:
```bash
# Docker
docker run -p 8080:8080 ghcr.io/tethyr/open-tethyr:latest \
  serve --mode cache --domain corp.acme.com

# Kubernetes
kubectl apply -f open-tethyr-cache.yaml

# AWS Lambda (container image)
# GCP Cloud Run
# Azure Container Apps
```

All clients at `*.corp.acme.com` automatically use corporate cache with zero client configuration.

### Developer Workflow

Configure a local cache:
```
_ax-cache.localhost TXT "endpoint=http://localhost:8080"
```

Start local cache:
```bash
open-tethyr serve --mode cache --domain localhost --port 8080
```

All local development automatically uses local cache and can work offline with cached discoveries.

### Multi-Region Deployment

Configure regional caches with central fallback:
```
# Root cache
_ax-cache.acme.com TXT "endpoint=http://cache-global.acme.com"

# Regional overrides
_ax-cache.us-east.acme.com TXT "endpoint=http://cache-us-east.acme.com"
_ax-cache.eu-west.acme.com TXT "endpoint=http://cache-eu-west.acme.com"
```

Regional caches check for root cache on startup via DNS, use root as fallback for cache misses, serve requests with lower latency, and operate independently if root is unavailable.

## AX Protocol Reference

### Agent Discovery Endpoint (AX RFC Standard)

**Location**: `https://_agent.<domain>/.well-known/agent-exchange.json`

**Example**: `https://_agent.api.acme.com/.well-known/agent-exchange.json`

**Format**: JSON document following AX 1.0 specification

No custom DNS records are required - uses standard HTTPS infrastructure as specified in the AX RFC.

### Cache Discovery Record (open-tethyr Extension)

**Format:**
```
_ax-cache.<domain> TXT "endpoint=<url>"
```

**Example:**
```
_ax-cache.acme.com TXT "endpoint=http://cache.acme.com:8080"
```

**Required fields:**
- `endpoint`: Full URL to cache server

**Domain scoping:**
- Clients use their own domain's cache
- `client.acme.com` uses cache from `_ax-cache.acme.com`
- Never queries target domain's cache

This is an open-tethyr extension for cache discovery, not part of the AX RFC. Agent discovery itself uses standard HTTPS well-known URIs.

## Security Model

### Client-Domain Scoping

Clients always use cache from their own domain, never from the target domain.

Example:
```
Client domain: acme.com
Target domain: api.partner.com

Cache used: _ax-cache.acme.com (YOUR cache, if configured)
Agents from: https://_agent.api.partner.com/.well-known/agent-exchange.json
```

You control your cache infrastructure. Target domains only publish read-only discovery metadata at standard HTTPS endpoints.

### Domain Locking (Optional)

Cache servers can optionally restrict discovery to home domain only:
```yaml
cache:
  domain: acme.com
  lock_to_home_domain: true
```

This allows `api.acme.com`, `internal.acme.com`, `*.acme.com` while blocking `api.partner.com`, `external.io`, and any non-acme.com domain.

Use cases include internal-only agent discovery, air-gapped environments, compliance/security requirements, and shadow AI prevention.

External domain allowlists for selective B2B discovery are coming in a future release.

### Trust Boundaries

You trust your own DNS infrastructure (for cache discovery only), your own cache infrastructure, and AX records as public discovery metadata (read-only, served via HTTPS).

You don't need to trust target domain's cache infrastructure or third-party caching services.

Discovery is public (finding agents and their capabilities). Execution requires separate authentication (OAuth, API keys, etc.). The cache never sees or stores execution credentials.

## Configuration

### Enterprise Configuration Pattern

open-tethyr supports global defaults with inheritance - configure once, use everywhere:
```yaml
# agents.yaml - Enterprise configuration

# Global defaults applied to all agents
defaults:
  provider: Acme Corp
  
  # Default OAuth configuration for all agents
  auth:
    methods: [OAuth2]
    oauth:
      domain: auth.acme.com
      # Auto-generates standard OAuth endpoints
  
  # Default endpoint settings
  endpoints:
    base_url: https://api.acme.com
    content_type: application/json
  
  # Default capabilities
  capabilities:
    async: true
    supports_callbacks: true
    callback_modes: [webhook, poll]
  
  # Default rate limits
  limits:
    max_concurrent_tasks: 100
    rate_limit_per_minute: 1000
  
  # Enterprise-wide extensions
  extensions:
    acme.compliance:
      data_classification: internal
      audit_required: true

# Agent definitions - inherit defaults, override as needed
agents:
  - name: Customer Lookup
    description: Query customer database
    endpoints:
      - protocol: rest
        url: /agents/customer-lookup  # Relative to base_url
        # Inherits OAuth2 auth from defaults
  
  - name: Order Processing
    description: Process and validate orders
    endpoints:
      - protocol: rest
        url: /agents/order-processing
    
    # Override limits for high-volume agent
    limits:
      max_concurrent_tasks: 500
      rate_limit_per_minute: 5000
  
  - name: Partner Gateway
    description: External partner integration
    provider: Partner Corp  # Override provider
    
    endpoints:
      - protocol: rest
        url: https://partner.example.com/api  # Absolute URL
        auth:
          methods: [mTLS, JWT]  # Override auth
          oauth:
            domain: partner-auth.example.com  # Partner OAuth
```

### Agent Configuration Schema

Full schema with all AX 1.0 fields:
```yaml
agents:
  - name: string                    # Required: Agent name
    description: string             # Required: Agent description
    provider: string                # Required: Provider/organization name
    
    endpoints:                      # Required: At least one endpoint
      - protocol: string            # graphql, mcp, a2a, rest, or custom
        url: string                 # Full URL or relative to base_url
        auth:                       # Auth methods for this endpoint
          methods: []               # OIDC, mTLS, JWT, OAuth2, API_KEY
          required_scopes: []       # OAuth scopes required
        content_type: string        # Default: application/json
    
    capabilities:                   # Optional: Agent capabilities
      intents: []                   # List of supported intents
      async: boolean                # Supports async execution
      supports_callbacks: boolean   # Supports callbacks
      callback_modes: []            # webhook, subscription, poll
    
    schema:                         # Optional: Schema/manifest URLs
      graphql_schema_url: string
      mcp_manifest_url: string
      rest_openapi_url: string
      a2a_schema_url: string
    
    limits:                         # Optional: Rate limits
      max_concurrent_tasks: integer
      rate_limit_per_minute: integer
    
    security:                       # Optional: Security metadata
      oauth:                        # OAuth configuration
        issuer: string
        authorization_endpoint: string
        token_endpoint: string
        jwks_uri: string
        userinfo_endpoint: string
        revocation_endpoint: string
      metadata_signature: string
    
    extensions:                     # Optional: Custom extensions
      key: value
```

### OAuth Configuration

open-tethyr supports standard OAuth 2.0 / OIDC endpoint patterns with smart generation.

Simple domain-based:
```yaml
defaults:
  auth:
    oauth:
      domain: auth.acme.com
      # Auto-generates:
      # issuer: https://auth.acme.com
      # authorization_endpoint: https://auth.acme.com/oauth2/authorize
      # token_endpoint: https://auth.acme.com/oauth2/token
      # jwks_uri: https://auth.acme.com/oauth2/jwks
      # userinfo_endpoint: https://auth.acme.com/oauth2/userinfo
      # revocation_endpoint: https://auth.acme.com/oauth2/revoke
```

Provider-specific templates:
```yaml
defaults:
  auth:
    oauth:
      # Okta
      provider: okta
      domain: acme.okta.com
      # Generates Okta-standard endpoints
      
      # Auth0
      provider: auth0
      domain: acme.us.auth0.com
      # Generates Auth0-standard endpoints
      
      # Azure AD
      provider: azure
      tenant_id: 12345678-1234-1234-1234-123456789012
      # Generates Azure AD endpoints
      
      # AWS Cognito
      provider: cognito
      region: us-east-1
      user_pool_id: us-east-1_ABC123
      # Generates Cognito endpoints
      
      # Google
      provider: google
      # Generates Google OAuth endpoints
      
      # Keycloak
      provider: keycloak
      domain: auth.acme.com
      realm: acme-realm
      # Generates Keycloak endpoints
```

Discovery-based:
```yaml
defaults:
  auth:
    oauth:
      discovery_url: https://auth.acme.com/.well-known/openid-configuration
      # Fetches OAuth configuration at generation time
```

Custom endpoints:
```yaml
defaults:
  auth:
    oauth:
      issuer: https://auth.acme.com
      authorization_endpoint: https://custom.acme.com/authorize
      token_endpoint: https://custom.acme.com/token
      jwks_uri: https://custom.acme.com/jwks.json
```

Supported OAuth providers:
- `okta`: Okta standard pattern
- `auth0`: Auth0 standard pattern
- `azure`: Azure AD / Microsoft Entra ID
- `cognito`: AWS Cognito
- `google`: Google OAuth
- `keycloak`: Keycloak / Red Hat SSO
- `generic`: RFC 8414 compliant (default)

### Configuration Examples

Minimal configuration:
```yaml
agents:
  - name: Hello Agent
    description: Simple hello world agent
    provider: Example Org
    endpoints:
      - protocol: rest
        url: https://api.example.com/agents/hello
        auth:
          methods: [OAuth2]
```

Full-featured multi-protocol agent:
```yaml
agents:
  - name: Multi-Protocol Gateway
    description: Supports GraphQL, MCP, A2A, and REST
    provider: Acme Corp
    
    endpoints:
      - protocol: graphql
        url: https://gateway.acme.com/graphql
        auth:
          methods: [OIDC]
      
      - protocol: mcp
        url: https://gateway.acme.com/mcp
        auth:
          methods: [OIDC]
      
      - protocol: a2a
        url: https://gateway.acme.com/a2a
        auth:
          methods: [mTLS, JWT]
      
      - protocol: rest
        url: https://gateway.acme.com/v1/tasks
        auth:
          methods: [OAuth2]
    
    capabilities:
      intents: [tool_execution, workflow_orchestration, multi_agent_planning]
      async: true
      supports_callbacks: true
      callback_modes: [webhook, subscription, poll]
    
    schema:
      graphql_schema_url: https://gateway.acme.com/.well-known/schema.graphql
      mcp_manifest_url: https://gateway.acme.com/.well-known/mcp-manifest.json
      rest_openapi_url: https://gateway.acme.com/.well-known/openapi.json
    
    limits:
      max_concurrent_tasks: 50
      rate_limit_per_minute: 600
    
    security:
      oauth:
        issuer: https://gateway.acme.com
        jwks_uri: https://gateway.acme.com/.well-known/jwks.json
    
    extensions:
      acme.routing:
        supports_delegation: true
        supports_feedback_exchange: false
```

Enterprise configuration with global defaults:
```yaml
defaults:
  provider: Acme Corp
  
  auth:
    methods: [OAuth2]
    oauth:
      provider: okta
      domain: acme.okta.com
  
  endpoints:
    base_url: https://api.acme.com
  
  capabilities:
    async: true
    supports_callbacks: true
  
  limits:
    max_concurrent_tasks: 100
    rate_limit_per_minute: 1000

agents:
  - name: Customer Lookup
    description: Query customer database
    endpoints:
      - protocol: rest
        url: /agents/customer-lookup
    # All other fields inherited from defaults
  
  - name: Order Processing
    description: Process orders at high volume
    endpoints:
      - protocol: rest
        url: /agents/order-processing
    limits:
      max_concurrent_tasks: 500  # Override for high-volume
```

### Cache Configuration Schema
```yaml
cache:
  domain: string                    # Required: domain for this cache
  port: integer                     # Default: 8080
  ttl: integer                      # Default: 3600 (seconds)
  max_size: integer                 # Default: 1000 (entries)
  lock_to_home_domain: boolean      # Default: false
  storage:
    type: memory|redis|disk         # Default: memory
    path: string                    # For disk storage
    connection: string              # For redis
```

## Deployment Options

### Containerized

Docker:
```dockerfile
FROM ghcr.io/tethyr/open-tethyr:latest
CMD ["serve", "--mode", "cache", "--domain", "acme.com"]
```

Able to deploy to any container platform.

### Static Binary

Single Rust binary with zero runtime dependencies for Linux (x86_64, ARM64), macOS (Intel, Apple Silicon), and Windows (x86_64).

Install:
```bash
# Cargo
cargo install open-tethyr

# Direct download (when releases are available)
# curl -L https://github.com/tethyr/open-tethyr/releases/latest/download/open-tethyr-linux-x64 -o open-tethyr
# chmod +x open-tethyr
```


## Relationship to AX Protocol

open-tethyr is a reference implementation of caching infrastructure for the Agent Discovery Exchange (AX) protocol.

The AX RFC defines the protocol specification for agent discovery records, standard HTTPS + well-known URI discovery mechanism, and JSON document format for agent metadata.

open-tethyr provides tools to generate AX-compliant records, distributed caching infrastructure (optional performance layer), DNS-based cache coordination (open-tethyr extension), policy enforcement capabilities, and a reference implementation.

The AX protocol itself doesn't require caching - agents can be discovered directly via HTTPS. open-tethyr adds optional caching for performance, observability, and policy enforcement.

Any AX-compliant system can interoperate with open-tethyr. Publishers using other tools to generate AX records can still be discovered by open-tethyr clients.

## Philosophy

### Boring is Better

We use HTTPS because it works. We use well-known URIs (RFC 8615) because they work. We don't reinvent infrastructure.

### Standards-First

Implements the AX RFC specification exactly as designed - HTTPS + well-known URIs for discovery, no custom DNS record types.

### Decentralized by Design

No central registry. No required services. Publishers control their own records served from their own infrastructure.

### Control Where It Matters

Discovery should be open and standardized. But organizations need control over their discovery policies, caching, and security boundaries.

### Client-Domain Scoping

Security through isolation. Clients use their own cache infrastructure (if they want caching). Direct discovery always works.

### Lazy Everything

Don't fetch what you don't need. Don't cache what isn't requested. Don't coordinate unless necessary.

### K.I.S.S

Each cache is independent. No propagation. No gossip. Simple hierarchical fallback. Caching is optional.

## SDK Repositories

Client SDKs are maintained in separate repositories for independent versioning and language-specific tooling:

- **TypeScript/JavaScript**: [tethyr/open-tethyr-ts](https://github.com/tethyr/open-tethyr-ts) _(coming soon)_
- **Python**: [tethyr/open-tethyr-py](https://github.com/tethyr/open-tethyr-py) _(coming soon)_
- **Rust**: Included in this repository as library

## Getting Started

### Publish Your First Agent

Install CLI:
```bash
cargo install open-tethyr
```

Create agent config:
```yaml
# agents.yaml
agents:
  - name: Hello Agent
    description: Simple hello world agent
    provider: Example Org
    endpoints:
      - protocol: rest
        url: https://api.example.com/agents/hello
        auth:
          methods: [OAuth2]
```

Generate AX records:
```bash
open-tethyr generate --config agents.yaml --output ./ax-records
```

Publish records at standard AX location. Make the generated `/.well-known/agent-exchange.json` accessible at `https://_agent.<your-domain>/.well-known/agent-exchange.json`:
```bash
# Upload to S3 + CloudFront
aws s3 sync ./ax-records s3://your-bucket/
# Configure DNS: _agent.api.example.com → CloudFront → S3

# Or serve via web server at subdomain
# Configure: _agent.api.example.com
cp -r ./ax-records /var/www/_agent.api.example.com/
```

Your agents are now discoverable at `https://_agent.api.example.com/.well-known/agent-exchange.json`.

### Set Up Controlled Discovery (Optional Caching)

Start cache server:
```bash
docker run -d -p 8080:8080 \
  ghcr.io/tethyr/open-tethyr:latest \
  serve --mode cache --domain corp.example.com
```

Add DNS record for cache discovery:
```
_ax-cache.corp.example.com TXT "endpoint=http://cache.corp.example.com:8080"
```

All clients auto-discover and use your controlled cache:
```typescript
import { OpenTethyr } from '@open-tethyr/sdk';

// Cache auto-discovered via DNS (if configured)
const client = new OpenTethyr({ domain: 'corp.example.com' });
const agents = await client.discover('api.partner.com');
```

Your cache now controls, monitors, and caches all agent discovery for your organization.

### Discover Agents

Install SDK:
```bash
npm install @open-tethyr/sdk
```

Use in code:
```typescript
import { OpenTethyr } from '@open-tethyr/sdk';

// Initialize with your domain
const client = new OpenTethyr({ domain: 'corp.example.com' });

// Discover agents from any domain
// Uses cache if configured, falls back to direct HTTPS discovery
const agents = await client.discover('api.partner.com');

agents.forEach(agent => {
  console.log(`${agent.name}: ${agent.description}`);
});
```

## Building from Source
```bash
# Clone repository
git clone https://github.com/tethyr/open-tethyr.git
cd open-tethyr

# Build all components
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

## Contributing

open-tethyr is open source (MIT). Contributions welcome.

Priority areas include cache storage backends (Valkey, disk), performance optimization, documentation improvements, AX protocol compliance, production deployment examples, policy enforcement capabilities, and OAuth provider templates.

Development setup:
```bash
git clone https://github.com/tethyr/open-tethyr.git
cd open-tethyr
cargo build
cargo test
```

Pull requests: Fork the repository, create a feature branch, add tests for new functionality, ensure `cargo test` passes, and submit PR with clear description.

## Community

- **GitHub Discussions**: Questions, ideas, and community support
- **Issues**: Bug reports and feature requests
- **LinkedIn**: Updates and announcements - follow the build in public

## Credits

Inspired by Aaron Sempf's vision for agent discovery as infrastructure and the [AX protocol proposal](https://github.com/sempfa/agent-discovery-exchange).

## License

MIT

---

**Tethering agents through open discovery**

*Building in public*