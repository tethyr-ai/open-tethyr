# Feature Specification: AX Draft Spec Alignment

**Feature Branch**: `002-ax-spec-alignment`
**Created**: 2026-03-14
**Status**: Draft
**Input**: User description: "Re-read the latest AX draft spec and compare to our implementation. The AX spec is available at https://github.com/sempfa/agent-discovery-exchange"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Correct Well-Known Path and Discovery URL (Priority: P1)

A system administrator generates AX records and deploys them for discovery. The records MUST be served at the path defined by the official AX specification so that any compliant client can discover them.

**Why this priority**: If the well-known path is wrong, no standard AX client can find the records. This is the most fundamental interoperability requirement.

**Independent Test**: Generate an AX record, verify the output file is named `agent-exchange` (not `agent-exchange.json`) and the discovery URL is `https://<domain>/.well-known/agent-exchange` (no `_agent.` subdomain prefix).

**Acceptance Scenarios**:

1. **Given** a YAML agent config, **When** the administrator runs the generate command, **Then** the output file is written to `<output>/.well-known/agent-exchange`
2. **Given** a domain `acme.com`, **When** the client constructs the discovery URL, **Then** the URL is `https://acme.com/.well-known/agent-exchange` (no `_agent.` prefix, no `.json` extension)
3. **Given** an existing deployment using the old path (`agent-exchange.json`), **When** the system encounters it, **Then** it falls back gracefully and logs a deprecation warning

---

### User Story 2 - Correct AX Document Structure (Priority: P1)

A system integrator receives an AX document and parses it. The document structure MUST match the official AX JSON schema: a single flat object with `record_type`, `version`, `agent`, `endpoints`, etc. -- NOT a wrapper with a `records` array.

**Why this priority**: If the document structure is wrong, no compliant parser will accept our output, and we cannot parse compliant documents from other implementations.

**Independent Test**: Generate an AX document and validate it against the official `ax-schema.json` from the spec repository. The root object must have `record_type: "AX"` directly, not nested inside a `records` array.

**Acceptance Scenarios**:

1. **Given** an agent config with one agent, **When** the system generates an AX document, **Then** the output is a single AX object (not wrapped in `{records: [...]}`)
2. **Given** an AX document from a third party following the spec, **When** the system parses it, **Then** it reads the flat structure correctly
3. **Given** a legacy document with the `{records: [...]}` wrapper, **When** the system encounters it, **Then** it handles both formats gracefully (flat preferred, wrapper accepted for backward compatibility)
4. **Given** a config with multiple agents, **When** the system generates output, **Then** it produces one AX document file per agent, or a single document for the first agent with others available through separate endpoints

---

### User Story 3 - Correct Field Requirements and Auth Methods (Priority: P1)

A developer uses the open-tethyr library to generate and validate AX records. The required/optional field designations and the set of recognized auth methods MUST match the official schema exactly.

**Why this priority**: Requiring fields that the spec marks optional (or vice versa) breaks interoperability. Using the wrong auth method names prevents integration with real-world agents.

**Independent Test**: Generate a minimal valid AX record with only spec-required fields. Validate against the official schema. Validate records with `AWS_IAM` auth. Verify `API_KEY` is still accepted as an extension.

**Acceptance Scenarios**:

1. **Given** the AX schema, **When** the validator checks required fields, **Then** only `record_type`, `version`, `agent.name`, `agent.description`, and `endpoints[].protocol`, `endpoints[].url` are required; `agent.provider` and `endpoints[].auth` are optional
2. **Given** a record with auth method `AWS_IAM`, **When** the validator checks it, **Then** it is accepted as valid
3. **Given** a record with auth method `API_KEY`, **When** the validator checks it, **Then** it is accepted (recognized extension)
4. **Given** a record with an unknown auth method `CustomAuth`, **When** the validator checks it, **Then** it produces a warning (not an error)

---

### User Story 4 - Typed Capabilities, Schema, Limits, and Security Blocks (Priority: P2)

A developer inspects an AX document with capabilities, schema references, limits, and security information. These blocks MUST have typed fields matching the official schema rather than opaque JSON catch-alls.

**Why this priority**: Typed fields enable IDE autocompletion, compile-time checks, and meaningful validation. The spec defines specific fields for each block.

**Independent Test**: Create an AX record with capabilities (intents, async, callbacks), schema URLs, limits, and security (issuer, jwks_url). Verify all fields serialize/deserialize correctly and match the spec schema.

**Acceptance Scenarios**:

1. **Given** a capabilities block with `intents`, `async`, `supports_callbacks`, `callback_modes`, **When** serialized to JSON, **Then** the output matches the spec schema field names exactly
2. **Given** a schema block with `graphql_schema_url`, `mcp_manifest_url`, `rest_openapi_url`, `introspection`, **When** parsed, **Then** each field is accessible as a typed optional field
3. **Given** a limits block with `max_concurrent_tasks`, `max_task_ttl_seconds`, `rate_limit_per_minute`, **When** serialized, **Then** the field names and types match the spec
4. **Given** a security block with `issuer`, `jwks_url`, `signature`, `metadata_signature`, **When** parsed, **Then** the flat structure is used (not a nested `oauth` wrapper)

---

### Edge Cases

- What happens when a document has both old wrapper format (`{records:[...]}`) and new flat format? The system detects the format automatically and parses whichever is present, preferring the flat format per spec.
- What happens when unknown auth methods are encountered? They are accepted with an informational warning -- the spec uses examples, not a closed set.
- What happens when unknown fields appear in capabilities/schema/limits? They are preserved via `additionalProperties: true` as the spec requires.
- What happens when `agent.provider` is missing? It is accepted as valid -- the spec marks it optional.
- What happens when `endpoints[].auth` is missing? It is accepted as valid -- the spec marks it optional.
- What happens when a config defines multiple agents but the spec is a single-document format? The system generates one file per agent or supports a multi-document convention.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST use `/.well-known/agent-exchange` as the well-known path (no `.json` extension) per the AX specification
- **FR-002**: System MUST construct discovery URLs as `https://<domain>/.well-known/agent-exchange` without a `_agent.` subdomain prefix
- **FR-003**: System MUST generate AX documents as a single flat JSON object with `record_type`, `version`, `agent`, `endpoints` at the root level (not wrapped in a `records` array)
- **FR-004**: System MUST parse both flat AX documents (spec-compliant) and legacy wrapper format (`{records:[...]}`) for backward compatibility
- **FR-005**: System MUST treat `agent.provider` as optional per the spec schema (not required)
- **FR-006**: System MUST treat `endpoints[].auth` as optional per the spec schema (not required)
- **FR-007**: System MUST recognize `AWS_IAM` as a valid auth method per the spec examples
- **FR-008**: System MUST accept unknown auth methods with a warning rather than rejecting them (the spec does not define a closed set)
- **FR-009**: System MUST implement typed `capabilities` fields: `intents` (string array), `async` (boolean), `supports_callbacks` (boolean), `callback_modes` (string array), with additional properties preserved
- **FR-010**: System MUST implement typed `schema` fields: `graphql_schema_url`, `mcp_manifest_url`, `rest_openapi_url` (URI strings), `introspection` (boolean), with additional properties preserved
- **FR-011**: System MUST implement typed `limits` fields: `max_concurrent_tasks`, `max_task_ttl_seconds`, `rate_limit_per_minute` (numbers), with additional properties preserved
- **FR-012**: System MUST implement typed `security` fields as a flat structure: `issuer` (URI), `jwks_url` (URI), `signature` (string), `metadata_signature` (string) -- not a nested `oauth` wrapper, with additional properties preserved
- **FR-013**: System MUST preserve unknown/additional fields in all objects per `additionalProperties: true` in the spec schema
- **FR-014**: All existing tests MUST continue to pass after the changes (backward compatibility preserved or tests updated)
- **FR-015**: The OAuth provider template feature MUST still work, populating `security.issuer` and `security.jwks_url` from provider configurations

### Key Entities

- **AX Document**: The root-level JSON object. Per the spec, this IS the record (flat), not a container of records. Required fields: `record_type`, `version`, `agent`, `endpoints`.
- **Agent**: Object with required `name` and `description`; optional `provider`. Allows additional properties.
- **Endpoint**: Object with required `protocol` and `url`; optional `auth` (string array) and `content_type`. Allows additional properties.
- **Capabilities**: Object with optional `intents`, `async`, `supports_callbacks`, `callback_modes`. Allows additional properties.
- **Schema**: Object with optional `graphql_schema_url`, `mcp_manifest_url`, `rest_openapi_url`, `introspection`. Allows additional properties.
- **Limits**: Object with optional `max_concurrent_tasks`, `max_task_ttl_seconds`, `rate_limit_per_minute`. Allows additional properties.
- **Security**: Flat object with optional `issuer`, `jwks_url`, `signature`, `metadata_signature`. Allows additional properties.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Generated AX documents validate against the official `ax-schema.json` with zero errors
- **SC-002**: Discovery URLs constructed by the system match the pattern `https://<domain>/.well-known/agent-exchange` for 100% of domains
- **SC-003**: All existing tests continue to pass after changes (zero regressions or tests updated to match spec)
- **SC-004**: Records from third-party AX implementations following the spec parse correctly without errors
- **SC-005**: All typed fields in capabilities, schema, limits, and security round-trip through serialization/deserialization with 100% fidelity
- **SC-006**: Legacy wrapper format (`{records:[...]}`) is still parseable for backward compatibility

## Assumptions

- The AX specification at github.com/sempfa/agent-discovery-exchange is the authoritative source
- The spec's `additionalProperties: true` on all objects means we must preserve unknown fields during round-trip
- The auth method list in the spec (`OIDC`, `AWS_IAM`, `OAuth2`, `mTLS`, `JWT`) is exemplary, not exhaustive -- unknown methods should produce warnings, not errors
- Our `API_KEY` auth method is a valid extension beyond the spec examples
- The OAuth provider template feature (Okta, Auth0, Generic) remains valuable and should populate the spec-compliant `security.issuer` and `security.jwks_url` fields instead of the nested `oauth` block
- The `_ax-cache.<domain>` DNS discovery pattern for cache servers is an open-tethyr extension, not part of the AX spec, and should be documented as such
- For multi-agent configs, we will generate a single AX document for the primary agent (first in the list) at the well-known path, consistent with the spec's single-document model
