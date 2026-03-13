# HTTP API Contract: Cache Server

**Base URL**: `https://{domain}:{port}`

## Endpoints

### GET /discover/{domain}

Discover agents for a target domain through the cache.

**Path parameters**:
- `domain` (string, required): Target domain to discover agents from

**Response headers**:
- `X-Correlation-Id`: UUID v4 for request tracing
- `Content-Type`: application/json

**Success response** (200):

```json
{
  "records": [
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
      "capabilities": { "async": true },
      "security": {
        "oauth": {
          "issuer": "https://auth.acme.com",
          "authorization_endpoint": "https://auth.acme.com/oauth2/authorize",
          "token_endpoint": "https://auth.acme.com/oauth2/token",
          "jwks_uri": "https://auth.acme.com/oauth2/v1/keys",
          "userinfo_endpoint": "https://auth.acme.com/oauth2/v1/userinfo",
          "revocation_endpoint": "https://auth.acme.com/oauth2/v1/revoke"
        }
      }
    }
  ]
}
```

**Error responses**:

| Status | Condition | Body |
|--------|-----------|------|
| 400 | Invalid domain format | `{"error": "Invalid domain: ...", "timestamp": "..."}` |
| 403 | Domain blocked by policy | `{"error": "Policy violation: domain not allowed", "timestamp": "..."}` |
| 404 | No AX record found at domain | `{"error": "No AX record found for ...", "timestamp": "..."}` |
| 429 | Rate limit exceeded | `{"error": "Rate limit exceeded for client ...", "timestamp": "..."}` |
| 502 | Upstream fetch failed (TLS, timeout, invalid response) | `{"error": "Upstream fetch failed: ...", "timestamp": "..."}` |
| 503 | Server unavailable | `{"error": "Service unavailable", "timestamp": "..."}` |

**Cache behavior**:
- Cache hit: Returns cached record immediately, `X-Cache: HIT` header
- Cache miss: Fetches from upstream (hierarchical fallback), caches result, returns
- Expired: Treated as cache miss, triggers re-fetch

---

### GET /health

Health check endpoint.

**Success response** (200):

```json
{
  "status": "healthy",
  "timestamp": "2026-03-12T10:30:00Z"
}
```

---

### GET /metrics

Prometheus-compatible metrics endpoint.

**Success response** (200, text/plain):

```
cache_hits_total 12345
cache_misses_total 678
active_connections 42
```

---

## Error Response Format

All error responses follow this structure:

```json
{
  "error": "Human-readable error message",
  "timestamp": "2026-03-12T10:30:00Z"
}
```

The `X-Correlation-Id` header is included in all responses (success and error) for request tracing.

## Rate Limiting

- Per-client IP using token bucket algorithm
- Configurable requests per minute and per hour
- HTTP 429 response when exceeded
- No `Retry-After` header in MVP (can be added post-MVP)

## DNS Discovery

Clients discover the cache server via DNS TXT record:

```
_ax-cache.{client-domain}  TXT  "endpoint=https://{server-domain}:{port}"
```

The TXT record value must be in the exact format: `endpoint=<URL>`
