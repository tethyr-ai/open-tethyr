# Data Model: AX Draft Spec Alignment

## Entity Changes (from feature 001 -> 002)

### AgentExchangeRecord (renamed semantically to "AX Document")

The root object. Per spec, this IS the AX document -- not wrapped in a container.

| Field | Type | Required | Change |
|-------|------|----------|--------|
| record_type | string (const "AX") | Yes | No change |
| version | string | Yes | No change |
| agent | Agent | Yes | No change |
| endpoints | Endpoint[] (min 1) | Yes | No change |
| capabilities | Capabilities | No | **NEW typed fields** |
| schema | Schema | No | **NEW typed fields** |
| limits | Limits | No | **NEW typed fields** |
| security | Security | No | **RESTRUCTURED: flat, not nested oauth** |
| extensions | object | No | No change (freeform) |

### Agent

| Field | Type | Required | Change |
|-------|------|----------|--------|
| name | string | Yes | No change |
| description | string | Yes | No change |
| provider | string | **No** | **CHANGED: was required, now optional per spec** |

### Endpoint

| Field | Type | Required | Change |
|-------|------|----------|--------|
| protocol | string | Yes | No change |
| url | string (URI) | Yes | No change |
| auth | string[] | **No** | **CHANGED: was required, now optional per spec** |
| content_type | string | No | No change |

### Capabilities (NEW typed fields)

| Field | Type | Required |
|-------|------|----------|
| intents | string[] | No |
| async | boolean | No |
| supports_callbacks | boolean | No |
| callback_modes | string[] | No |
| (additional) | any | No (via flatten) |

### Schema (NEW typed fields)

| Field | Type | Required |
|-------|------|----------|
| graphql_schema_url | string (URI) | No |
| mcp_manifest_url | string (URI) | No |
| rest_openapi_url | string (URI) | No |
| introspection | boolean | No |
| (additional) | any | No (via flatten) |

### Limits (NEW typed fields)

| Field | Type | Required |
|-------|------|----------|
| max_concurrent_tasks | number | No |
| max_task_ttl_seconds | number | No |
| rate_limit_per_minute | number | No |
| (additional) | any | No (via flatten) |

### Security (RESTRUCTURED)

| Field | Type | Required | Change |
|-------|------|----------|--------|
| issuer | string (URI) | No | **MOVED: was inside oauth sub-object** |
| jwks_url | string (URI) | No | **NEW: replaces oauth.jwks_uri** |
| signature | string | No | **NEW** |
| metadata_signature | string | No | **NEW** |
| (additional) | any | No (via flatten) |

**Removed**: `oauth` nested object with `authorization_endpoint`, `token_endpoint`, `userinfo_endpoint`, `revocation_endpoint`. OAuth provider templates now populate `issuer` and `jwks_url` directly.

### AgentExchangeDocument (BACKWARD COMPAT)

Kept for parsing legacy format only. Not used for generation.

| Field | Type | Notes |
|-------|------|-------|
| records | AgentExchangeRecord[] | Legacy wrapper format |

## Path Changes

| Context | Old | New (spec-compliant) |
|---------|-----|---------------------|
| Well-known path | `/.well-known/agent-exchange.json` | `/.well-known/agent-exchange` |
| Discovery URL | `https://_agent.<domain>/.well-known/agent-exchange.json` | `https://<domain>/.well-known/agent-exchange` |
| Output filename | `agent-exchange.json` | `agent-exchange` |

## Auth Method Changes

| Old | New |
|-----|-----|
| Closed set: OIDC, OAuth2, mTLS, JWT, API_KEY | Open set with known examples: OIDC, AWS_IAM, OAuth2, mTLS, JWT, API_KEY |
| Unknown methods rejected with error | Unknown methods accepted with warning |
