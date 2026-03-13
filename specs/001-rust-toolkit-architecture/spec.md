# Feature Specification: Open-Tethyr Rust Toolkit Architecture

**Feature Branch**: `001-rust-toolkit-architecture`
**Created**: 2026-03-12
**Status**: Draft
**Input**: User description: "create a complete spec from the .kiro/specs/rust-toolkit-architecture/requirements"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Generate AX Discovery Records from Configuration (Priority: P1)

A system administrator wants to publish agent discovery information for their organization. They create a YAML configuration file describing their agents (name, description, endpoints, authentication methods) and use the CLI tool to generate a standards-compliant AX record. The generated JSON file is placed at the well-known URI path so other systems can discover these agents.

**Why this priority**: This is the foundational capability that enables the entire AX ecosystem. Without record generation, no other feature (caching, discovery, validation) has data to work with. It delivers immediate value by letting organizations publish agent metadata.

**Independent Test**: Can be fully tested by writing a YAML config file, running the CLI generate command, and verifying the output JSON matches AX 1.0 specification. Delivers value as a standalone record generation tool.

**Acceptance Scenarios**:

1. **Given** a valid YAML configuration with agent definitions, **When** the administrator runs the generate command, **Then** the system produces a valid AX 1.0 JSON document at `/.well-known/agent-exchange.json`
2. **Given** a YAML configuration with global defaults and agent-specific overrides, **When** the administrator runs the generate command, **Then** agents inherit global defaults and their overrides take precedence
3. **Given** a YAML configuration with an OAuth provider template (e.g., Okta), **When** the administrator runs the generate command, **Then** the generated record includes correctly-formed provider-specific OAuth endpoint URLs
4. **Given** an invalid YAML configuration (missing required fields, invalid domain format), **When** the administrator runs the generate command, **Then** the system displays clear error messages identifying each problem

---

### User Story 2 - Validate Existing AX Records (Priority: P1)

A system integrator receives an AX record from a partner organization and needs to verify it complies with the AX 1.0 specification before integrating. They use the CLI validate command to check the record for completeness, correct formatting, and protocol compliance.

**Why this priority**: Validation is essential for interoperability. Organizations need confidence that records they produce and consume are correct. This pairs with generation as a core capability.

**Independent Test**: Can be fully tested by providing various AX JSON files (valid and invalid) to the validate command and verifying correct pass/fail results with specific error messages.

**Acceptance Scenarios**:

1. **Given** a valid AX 1.0 JSON file, **When** the integrator runs the validate command, **Then** the system confirms the record is valid
2. **Given** an AX JSON file missing required fields, **When** the integrator runs the validate command, **Then** the system lists each missing field with an explanation
3. **Given** an AX JSON file with version "2.0", **When** the integrator runs the validate command, **Then** the system reports an unsupported version warning
4. **Given** an AX JSON file with invalid authentication methods, **When** the integrator runs the validate command, **Then** the system identifies the invalid auth entries

---

### User Story 3 - Deploy and Operate a Cache Server (Priority: P1)

An infrastructure operator deploys the cache server to provide controlled, high-performance agent discovery for their organization. The server caches AX records in memory, enforces organizational policies (domain locking, allowlists), and coordinates with other caches in a hierarchy.

**Why this priority**: The cache server is the core infrastructure component that enables organizational control over agent discovery. It provides performance benefits and policy enforcement which are the primary value proposition of open-tethyr.

**Independent Test**: Can be fully tested by starting the cache server, sending discovery requests, verifying cache behavior (hits, misses, TTL expiration), and confirming policy enforcement responses.

**Acceptance Scenarios**:

1. **Given** a running cache server, **When** a client requests agent discovery for a domain, **Then** the server checks its local cache first and returns cached results if available
2. **Given** a cache miss, **When** the server fetches from the target domain's AX endpoint, **Then** the record is cached with appropriate TTL and returned to the client
3. **Given** domain locking is enabled, **When** a client requests discovery for an external domain not on the allowlist, **Then** the server rejects the request with an appropriate error
4. **Given** a configured hierarchical cache, **When** a local cache miss occurs, **Then** the server queries the root cache before fetching directly from the origin
5. **Given** the root cache is unavailable, **When** the server attempts hierarchical fallback, **Then** it continues operating independently by fetching directly
6. **Given** the cache reaches its configured maximum size, **When** a new record needs caching, **Then** the least recently used record is evicted

---

### User Story 4 - Discover Agents Using the Client Library (Priority: P2)

A developer building a Rust application wants to discover agents programmatically. They use the client SDK to perform agent discovery, which automatically finds the organizational cache via DNS and falls back to direct HTTPS discovery if no cache is available.

**Why this priority**: The client SDK enables programmatic consumption of AX records, making the system useful for application developers. It depends on the cache server and core protocol being functional first.

**Independent Test**: Can be fully tested by initializing the client with a domain, verifying DNS-based cache discovery, and confirming correct fallback behavior when no cache is configured.

**Acceptance Scenarios**:

1. **Given** a domain with a `_ax-cache.<domain>` DNS TXT record, **When** the client initializes, **Then** it discovers and uses the organizational cache automatically
2. **Given** no DNS cache record exists, **When** the client initializes, **Then** it falls back to direct HTTPS discovery without errors
3. **Given** a configured cache endpoint, **When** the client requests agent discovery, **Then** results are returned from the cache
4. **Given** the cache endpoint is unreachable, **When** the client requests agent discovery, **Then** it falls back to direct discovery gracefully

---

### User Story 5 - Test Agent Discovery End-to-End (Priority: P2)

A system administrator wants to verify that their AX setup works correctly from the perspective of a client. They use the CLI discover command to test discovery against a specific domain, optionally specifying a cache endpoint.

**Why this priority**: End-to-end testing completes the operational workflow, but depends on generation and server capabilities being in place.

**Independent Test**: Can be fully tested by running the discover command against a domain hosting AX records and verifying correct agent information is returned.

**Acceptance Scenarios**:

1. **Given** a domain hosting AX records, **When** the administrator runs the discover command, **Then** discovered agents are displayed with their metadata
2. **Given** a --cache flag with a cache server URL, **When** the administrator runs the discover command, **Then** discovery goes through the specified cache
3. **Given** a domain with no AX records, **When** the administrator runs the discover command, **Then** the system reports no agents found with a clear message

---

### User Story 6 - Monitor and Operate the System (Priority: P2)

An operator needs visibility into cache server behavior for troubleshooting and capacity planning. They use structured logs, metrics endpoints, and CLI commands for cache management.

**Why this priority**: Operational visibility is essential for production deployments but is not required for core functionality.

**Independent Test**: Can be fully tested by starting the server, generating traffic, and verifying structured logs contain expected fields and the metrics endpoint returns valid data.

**Acceptance Scenarios**:

1. **Given** a running cache server handling requests, **When** the operator views logs, **Then** each request includes structured fields: method, path, status code, duration, client IP, and cache hit/miss indicator
2. **Given** a running cache server, **When** the operator queries the metrics endpoint, **Then** cache statistics (hits, misses, evictions, active connections) are returned
3. **Given** a request that needs tracing across components, **When** the operator searches logs, **Then** a correlation ID links all related log entries
4. **Given** a stale cache entry, **When** the operator runs the cache invalidation CLI command, **Then** the specified entry is removed from the cache

---

### User Story 7 - Build and Deploy Across Platforms (Priority: P3)

A DevOps engineer needs to deploy open-tethyr across different environments (Linux servers, macOS development machines, containerized infrastructure). They use CI/CD pipelines to produce platform-specific binaries and Docker images.

**Why this priority**: Distribution is important for adoption but is a deployment concern, not a core functional requirement.

**Independent Test**: Can be fully tested by running the build pipeline and verifying binaries are produced for each target platform and Docker images start correctly.

**Acceptance Scenarios**:

1. **Given** a release trigger, **When** the CI/CD pipeline runs, **Then** static binaries are produced for Linux (musl), macOS, and Windows
2. **Given** a release trigger, **When** the CI/CD pipeline runs, **Then** Docker images are published to GitHub Container Registry
3. **Given** a Linux static binary, **When** deployed to a minimal container, **Then** the binary runs without additional runtime dependencies

---

### Edge Cases

- What happens when the cache server receives a request for a domain with an expired TLS certificate? The system rejects the upstream fetch and returns HTTP 502 to the client.
- How does the system handle a DNS TXT record with a malformed cache endpoint URL? The system logs a warning and falls back to direct discovery.
- What happens when the cache is full and multiple concurrent requests trigger evictions simultaneously? The LRU eviction policy handles concurrent access safely.
- How does the system handle an AX endpoint that returns valid JSON but not a valid AX record? The validator rejects it and the cache does not store the invalid response.
- What happens when a hierarchical cache configuration contains circular dependencies? The system detects cycles during initialization and refuses to start with a clear error.
- What happens when the upstream AX endpoint responds with Cache-Control: no-cache? The system respects the directive and does not cache the response.
- How does the system handle rate-limited clients that continue sending requests? Returns HTTP 429 with appropriate retry information.
- What happens when configuration specifies both environment variables and CLI arguments for the same setting? CLI arguments take precedence over environment variables, which take precedence over configuration file values.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST organize code as a Cargo workspace with separate crates for CLI, server, and core library with shared dependencies and independent versioning
- **FR-002**: System MUST provide a CLI tool that generates valid AX 1.0 JSON documents from YAML configuration files
- **FR-003**: System MUST provide a CLI tool that validates AX records for compliance with the AX 1.0 specification, reporting all violations with clear messages
- **FR-004**: System MUST provide a CLI tool that tests agent discovery in both cached and direct modes
- **FR-005**: System MUST generate the standard `/.well-known/agent-exchange.json` file structure for AX record output
- **FR-006**: System MUST provide a cache server that binds to a configurable port and domain, checks local storage first for discovery requests, and fetches from target AX endpoints on cache miss
- **FR-007**: System MUST implement in-memory cache storage with LRU eviction, configurable maximum size (in number of records), TTL expiration, and respect for Cache-Control headers from AX endpoints
- **FR-008**: System MUST support hierarchical caching with DNS-based root cache discovery, multi-level fallback chains (regional to root to direct), and graceful degradation when upstream caches are unavailable
- **FR-009**: System MUST detect and prevent circular cache dependencies during configuration validation
- **FR-010**: System MUST implement DNS-based cache discovery using TXT records at `_ax-cache.<domain>` with `endpoint=<url>` format, falling back to direct HTTPS discovery when no cache is found
- **FR-011**: System MUST enforce domain locking policies, support allowlist configuration for approved external domains, and reject discovery requests for blocked domains with appropriate error responses
- **FR-012**: System MUST log all discovery requests for audit purposes with structured fields: method, path, status code, duration, client IP, and cache hit/miss indicator
- **FR-013**: System MUST implement per-client IP rate limiting with configurable limits (requests per minute/hour) and return HTTP 429 when exceeded
- **FR-014**: System MUST validate HTTPS certificates when fetching AX records and verify responses are served from the `/.well-known/agent-exchange.json` path
- **FR-015**: System MUST provide strongly-typed data models for all AX 1.0 record components with serialization/deserialization support and validation of required fields and format constraints
- **FR-016**: System MUST support OAuth provider templates for Okta (standard endpoint patterns), Auth0 (tenant-specific URLs), and Generic OAuth2 (RFC 8414 compliant providers)
- **FR-017**: System MUST support configuration inheritance with global defaults, agent-specific overrides, and nested inheritance for complex configuration objects with type-safe merging
- **FR-018**: System MUST support YAML configuration files, command-line argument overrides, and environment variable configuration with validation of domain names, port numbers, URLs, and TTL values
- **FR-019**: System MUST use structured logging with configurable log levels (error, warn, info, debug, trace) and include correlation IDs for request tracking across components
- **FR-020**: System MUST return appropriate HTTP status codes: 400 (invalid requests), 404 (not found), 429 (rate limiting), 502 (upstream fetch failures), 503 (service unavailable)
- **FR-021**: System MUST validate AX record version field is "1.0", log warnings for unsupported versions, skip agents with unsupported protocol versions, and include version information in error messages
- **FR-022**: System MUST provide a client API for agent discovery from Rust applications with automatic DNS-based cache discovery and fallback to direct HTTPS discovery
- **FR-023**: System MUST support cache invalidation via CLI command
- **FR-024**: System MUST support cross-compilation for Linux, macOS, and Windows, produce statically-linked binaries for Linux using musl libc, and generate Docker images published to GitHub Container Registry
- **FR-025**: System MUST include unit tests, integration tests spanning multiple crates, property-based tests for data validation, and mock HTTP servers for testing network interactions
- **FR-026**: System MUST support `_agent.<domain>` subdomain configuration for AX endpoints
- **FR-027**: System MUST implement configurable request timeouts with a default of 30 seconds for external AX fetches

### Key Entities

- **AX Record (AgentExchangeRecord)**: The core data unit - a JSON document containing agent metadata following AX 1.0 specification. Includes record type, version, and a collection of agent entries. Identified by domain and served at well-known URIs.
- **Agent**: An individual agent entry within an AX record. Contains name, description, provider information, endpoints, supported protocols, and authentication methods.
- **Endpoint**: A network location where an agent can be reached. Includes URL, supported protocols (REST, GraphQL, MCP, A2A, Custom), and authentication method references.
- **Cache Entry**: A stored AX record within the cache server. Contains the AX record, TTL expiration time, Cache-Control metadata, and access timestamp for LRU tracking.
- **Policy Configuration**: Rules governing cache server behavior. Includes domain locking settings, external domain allowlists, and rate limiting parameters.
- **OAuth Provider Template**: A reusable template for generating OAuth endpoint URLs for specific providers (Okta, Auth0, Generic OAuth2). Contains provider-specific URL patterns and endpoint generation logic.
- **Agent Configuration**: A YAML-based configuration file defining agents to be published. Contains global defaults, agent-specific settings, and supports inheritance and overrides.
- **Cache Hierarchy**: The relationship between cache servers in a multi-level deployment. Includes root cache (discovered via DNS), regional caches, and the fallback chain between them.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: System generates valid AX 1.0 records from configuration in under 1 second for configurations containing up to 100 agents
- **SC-002**: Cache server responds to cached discovery requests in under 10 milliseconds (excluding network latency)
- **SC-003**: Cache server handles at least 1,000 concurrent discovery requests without errors or degradation
- **SC-004**: DNS-based cache discovery completes within 2 seconds, including fallback to direct discovery
- **SC-005**: All generated AX records pass validation against the AX 1.0 specification with zero false positives or false negatives
- **SC-006**: Hierarchical cache fallback (regional to root to direct) completes within 5 seconds under normal network conditions
- **SC-007**: Policy enforcement (domain locking, allowlists) adds less than 1 millisecond of overhead per request
- **SC-008**: System starts from cold cache and begins serving requests within 3 seconds
- **SC-009**: CLI validation of an AX record file completes in under 500 milliseconds
- **SC-010**: Configuration inheritance correctly merges nested settings with 100% accuracy across all test scenarios
- **SC-011**: Rate limiting accurately enforces configured thresholds with less than 5% variance
- **SC-012**: Static binaries are produced for Linux (musl), macOS, and Windows, each under 50MB
- **SC-013**: All three OAuth provider templates (Okta, Auth0, Generic) generate correctly-formed endpoint URLs verified by integration tests
- **SC-014**: Test suite achieves at least 80% code coverage across all crates

## Assumptions

- The AX 1.0 specification is stable and will not undergo breaking changes during initial development
- DNS TXT record propagation is handled by standard DNS infrastructure; the system does not manage DNS records itself
- In-memory cache storage is sufficient for MVP; persistent storage backends will be added post-MVP
- Organizations deploying the cache server have control over their DNS configuration for `_ax-cache.<domain>` records
- OAuth provider endpoint URL patterns (Okta, Auth0) follow their current documented standards
- The system targets Rust stable toolchain; no nightly-only features are required
- Docker deployment targets Linux containers (amd64 architecture primarily, with arm64 as stretch goal)
- Configuration precedence follows standard convention: CLI arguments > environment variables > configuration file > defaults
