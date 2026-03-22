<!--
  Sync Impact Report
  ===================
  Version change: (new) -> 1.0.0
  Modified principles: N/A (initial ratification)
  Added sections:
    - Core Principles (6 principles)
    - Performance Standards
    - Development Workflow
    - Governance
  Removed sections: N/A (initial ratification)
  Templates requiring updates:
    - .specify/templates/plan-template.md: No update needed (generic Constitution Check section)
    - .specify/templates/spec-template.md: No update needed (generic structure)
    - .specify/templates/tasks-template.md: No update needed (generic structure)
  Follow-up TODOs: None
-->

# Open-Tethyr Constitution

## Core Principles

### I. Library-First Architecture

Every feature MUST start as functionality in the core library crate
(`open-tethyr`). The library MUST be self-contained, independently
testable, and usable without the CLI or server components.

- The Cargo workspace MUST contain exactly two crates: `open-tethyr`
  (library) and `cli` (binary). Server code lives in the library
  behind a feature flag.
- Feature flags (`client`, `server`, `full`) MUST control compiled
  dependencies. The default feature set MUST be `client` only,
  keeping the dependency footprint minimal for SDK consumers.
- Public API surface MUST be re-exported through `lib.rs`. Internal
  modules MUST NOT be exposed.
- No circular dependencies between modules. Dependency direction
  flows: CLI -> library; server feature -> core modules; never the
  reverse.

**Rationale**: SDK consumers MUST NOT be forced to compile server
dependencies (axum, tower). Library-first ensures every capability
is programmatically accessible before being wrapped in CLI or HTTP.

### II. AX Protocol Compliance (NON-NEGOTIABLE)

All generated, validated, and cached AX records MUST conform to the
AX 1.0 specification. No shortcuts, partial compliance, or
undocumented extensions are permitted.

- Every `AgentExchangeRecord` MUST have `record_type: "AX"` and
  `version: "1.0"`.
- Authentication methods MUST be from the approved set: `OIDC`,
  `OAuth2`, `mTLS`, `JWT`, `API_KEY`.
- AX records MUST be served at and fetched from the well-known path
  `/.well-known/agent-exchange.json`.
- HTTPS MUST be enforced for all AX endpoint communication.
  Certificate validation MUST NOT be disabled.
- Unsupported AX versions MUST be logged as warnings and skipped,
  never silently accepted.

**Rationale**: Interoperability depends on strict protocol adherence.
A non-compliant record poisons the discovery ecosystem for all
participants.

### III. Property-Based Test Discipline

Core correctness properties MUST be validated with property-based
tests (proptest). Unit and integration tests complement but do not
replace property tests for invariant verification.

- Every data model (AX records, configuration, cache entries) MUST
  have a serialization round-trip property test.
- Every validation function MUST have a property test proving valid
  inputs pass and invalid inputs are rejected with correct errors.
- Every stateful component (cache, rate limiter, coordinator) MUST
  have property tests for its key invariants (e.g., LRU eviction
  order, TTL expiration, token bucket refill).
- Property tests MUST use `proptest` with custom strategy generators
  for domain types. Generators MUST produce realistic, non-trivial
  inputs.
- Integration tests MUST use mock infrastructure (wiremock for HTTP,
  mock resolvers for DNS). Tests MUST NOT depend on external
  services.

**Rationale**: Example-based tests catch known bugs. Property tests
catch unknown bugs by exploring the input space. For a protocol
implementation, correctness across all valid inputs is essential.

### IV. Simplicity and YAGNI

Choose the simplest correct implementation. Do not add abstractions,
indirections, or capabilities until a concrete requirement demands
them.

- In-memory storage (HashMap + LRU) is sufficient until benchmarks
  prove otherwise. Do not introduce persistent storage, external
  caches, or database layers preemptively.
- Do not add generic trait abstractions for components that have
  exactly one implementation (e.g., no `CacheBackend` trait when
  only `MemoryCache` exists).
- Dependencies MUST be justified by a specific requirement. Prefer
  the Rust standard library over external crates when the standard
  library solution is adequate.
- Configuration options MUST map to a documented requirement. Do not
  add configurable knobs "just in case".
- When two approaches have equivalent correctness, prefer the one
  with fewer lines of code and fewer dependencies.

**Rationale**: Premature abstraction creates maintenance burden and
cognitive overhead with no user value. The MVP targets
single-organization deployments; enterprise-scale patterns can be
added when enterprise-scale requirements arrive.

### V. Structured Observability

Every request-handling path MUST produce structured, machine-parsable
log entries with correlation IDs. Metrics MUST be exposed for
operational monitoring.

- All HTTP request logs MUST include structured fields: `method`,
  `path`, `status_code`, `duration_ms`, `client_ip`, `cache_hit`.
- Every request MUST carry a correlation ID (UUID v4) propagated
  through all log entries and returned in the `X-Correlation-Id`
  response header.
- Log levels MUST be configurable at runtime via environment variable
  (`OPEN_TETHYR_LOG`) and configuration file. Supported levels:
  `error`, `warn`, `info`, `debug`, `trace`.
- The `/metrics` endpoint MUST expose cache statistics (hits, misses,
  evictions, active connections) and request duration histograms in
  Prometheus text format.
- Discovery requests MUST be logged for audit: domain, client IP,
  policy result, cache result.

**Rationale**: Production operators cannot troubleshoot what they
cannot observe. Structured logging enables automated alerting and
dashboarding without log parsing fragility.

### VI. Static Distribution

Release binaries MUST be statically linked and self-contained.
Deployment MUST NOT require runtime dependencies beyond the OS
kernel.

- Linux binaries MUST be compiled with musl libc for full static
  linking.
- Release artifacts MUST be produced for Linux (x86_64-musl),
  macOS (x86_64), and Windows (x86_64-msvc).
- Docker images MUST use multi-stage builds with a minimal runtime
  image (scratch or distroless).
- Static binaries MUST be under 50MB each.
- The `rustls` TLS backend MUST be used instead of OpenSSL to
  avoid dynamic linking requirements.

**Rationale**: Static binaries eliminate "works on my machine"
deployment failures and enable zero-dependency container images.

## Performance Standards

Measurable performance thresholds that MUST be met before release:

- Cached discovery lookups MUST complete in under 10 milliseconds
  (excluding network latency).
- The cache server MUST handle at least 1,000 concurrent discovery
  requests without errors or degradation.
- AX record generation from configuration MUST complete in under
  1 second for configurations containing up to 100 agents.
- DNS-based cache discovery MUST complete within 2 seconds,
  including fallback to direct discovery.
- Policy enforcement (domain locking, allowlists) MUST add less
  than 1 millisecond of overhead per request.
- Rate limiting MUST enforce configured thresholds with less than
  5% variance.

Performance MUST be validated with criterion benchmarks for cache
operations, AX record parsing, and configuration merging.

## Development Workflow

Rules governing how code changes are proposed, reviewed, and merged:

- All code MUST pass `cargo clippy --workspace --all-features
  -- -D warnings` with zero warnings before merge.
- All code MUST be formatted with `cargo fmt --all`.
- The full test suite (`cargo test --workspace --all-features`)
  MUST pass before merge.
- Configuration precedence MUST follow: CLI arguments > environment
  variables > configuration file > defaults. This order MUST NOT
  be violated.
- Error types in the library MUST use `thiserror` derive macros.
  The CLI binary MAY use `anyhow` for application-level errors.
- HTTP error responses MUST return structured JSON with `error`,
  `timestamp`, and `correlation_id` fields. Status code mapping:
  400 (invalid request), 403 (policy violation), 404 (not found),
  429 (rate limited), 502 (upstream failure).

## Governance

This constitution is the authoritative source of engineering
principles for the open-tethyr project. It supersedes informal
conventions, prior discussions, and ad-hoc decisions.

- **Amendments** require documentation of the change, rationale,
  and a migration plan for any code that violates the new rule.
- **Compliance** is verified at plan time (Constitution Check in
  plan.md) and at code review. Violations MUST be documented in
  the Complexity Tracking table with justification.
- **Version policy**: MAJOR for principle removals or redefinitions,
  MINOR for new principles or material expansions, PATCH for
  clarifications and wording fixes.
- **Runtime guidance**: Use `CLAUDE.md` for agent-specific
  development guidance that supplements (but does not override)
  this constitution.

**Version**: 1.0.0 | **Ratified**: 2026-03-13 | **Last Amended**: 2026-03-13
