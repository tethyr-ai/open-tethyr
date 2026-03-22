# Research: AX Draft Spec Alignment

## R1: Well-Known Path

**Decision**: Use `/.well-known/agent-exchange` (no `.json` extension)
**Rationale**: The AX spec at sempfa/agent-discovery-exchange defines the path as `/.well-known/agent-exchange`. The content type is `application/json` served via HTTP headers, not implied by file extension.
**Alternatives**: Keep `.json` extension for clarity -- rejected because it breaks spec compliance.

## R2: Discovery URL Pattern

**Decision**: Use `https://<domain>/.well-known/agent-exchange` (no `_agent.` subdomain)
**Rationale**: The AX spec defines discovery at the domain itself, not a subdomain. The `_agent.` prefix was an internal convention not present in the spec.
**Alternatives**: Keep `_agent.` prefix as optional fallback -- rejected for simplicity. Will add fallback only if needed.

## R3: Document Structure (Flat vs Wrapper)

**Decision**: Generate flat AX documents. Parse both flat and legacy `{records:[...]}` wrapper.
**Rationale**: The AX JSON schema defines a single object with `record_type` at root. There is no `records` array wrapper in the spec. For backward compatibility, the parser should detect and handle both.
**Alternatives**: Breaking change only (no legacy support) -- rejected because existing deployments may use the wrapper format.

## R4: Multi-Agent Handling

**Decision**: Generate one AX document file per agent in separate directories, OR generate the primary (first) agent at the well-known path.
**Rationale**: The AX spec is a single-document-per-domain model. For multi-agent configs, the first agent becomes the well-known document. Additional agents could be served at custom paths.
**Alternatives**: Keep `records` array for multi-agent -- rejected because it violates the spec schema.

## R5: Required vs Optional Fields

**Decision**: Align with spec -- `agent.provider` and `endpoints[].auth` are optional.
**Rationale**: The `ax-schema.json` lists `required: ["name", "description"]` for agent (no `provider`) and `required: ["protocol", "url"]` for endpoints (no `auth`).
**Alternatives**: Keep stricter validation as a "strict mode" -- rejected for MVP, could add later.

## R6: Auth Method Set

**Decision**: Accept any string as auth method. Warn on unrecognized methods. Add `AWS_IAM` to the known examples list.
**Rationale**: The spec uses `description: "Supported authentication mechanisms (e.g., 'OIDC', 'AWS_IAM', 'OAuth2', 'mTLS', 'JWT')"` -- this is exemplary, not a closed enum. Our `API_KEY` is valid too.
**Alternatives**: Strict enum validation -- rejected because it would reject valid third-party records.

## R7: Security Block Structure

**Decision**: Flat structure with `issuer`, `jwks_url`, `signature`, `metadata_signature` + `additionalProperties`.
**Rationale**: The spec schema defines these four fields directly on the security object. Our nested `oauth` wrapper was an internal design not in the spec. OAuth provider templates will populate `issuer` and `jwks_url` directly.
**Alternatives**: Keep nested `oauth` as an extension alongside flat fields -- rejected for cleanliness.

## R8: Typed Sub-Objects

**Decision**: Add typed optional fields matching the spec for capabilities, schema, and limits. Keep `#[serde(flatten)] extra` for additional properties.
**Rationale**: The spec defines specific fields. Having them typed enables autocompletion and validation while `flatten` preserves unknown fields per `additionalProperties: true`.
**Alternatives**: Keep pure `serde_json::Value` catch-all -- rejected because it loses type safety.

## R9: Backward Compatibility Strategy

**Decision**: Parser accepts both formats. Generator produces spec-compliant output only. Tests updated to match new structure.
**Rationale**: Existing users may have deployed the old format. A graceful migration path is better than a hard break.
**Alternatives**: Version flag to toggle old/new format -- rejected as overly complex for this scope.
