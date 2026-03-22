# Research: Open-Tethyr Rust Toolkit Architecture

**Branch**: `001-rust-toolkit-architecture` | **Date**: 2026-03-12

## Decision 1: Workspace Structure - Two Crates vs Three

**Decision**: Two crates: `open-tethyr` (library) and `cli` (binary). Server code lives inside the library under `#[cfg(feature = "server")]`.

**Rationale**: The existing Kiro design consolidates server functionality into the main library via feature flags rather than a separate crate. This is the standard Rust pattern for library+server combos (see axum, actix-web examples). It avoids circular dependency issues (CLI needs server features, but a separate server crate would also need core). Feature flags (`client`, `server`, `full`) give consumers fine-grained control over compiled dependencies.

**Alternatives considered**:
- Three separate crates (core, server, cli): More isolation but creates dependency management complexity. The server crate would duplicate types from core or require a shared types crate. Rejected because feature flags achieve the same compile-time separation with less overhead.
- Monolithic single crate: Simpler but forces all consumers to compile server dependencies (axum, tower). Rejected because SDK users need a lightweight dependency.

## Decision 2: HTTP Framework - axum

**Decision**: Use axum 0.7 for the cache server HTTP layer.

**Rationale**: axum is the dominant async HTTP framework in the Rust ecosystem, built on tower middleware and hyper. It integrates naturally with tokio (already required for async DNS and HTTP client). The tower middleware ecosystem provides ready-made rate limiting, tracing, and CORS layers. axum's extractor pattern maps cleanly to the cache server's handler signatures.

**Alternatives considered**:
- actix-web: Mature but uses its own runtime, creating friction with tokio-based dependencies (trust-dns-resolver, reqwest). Rejected for ecosystem consistency.
- warp: Good but less actively maintained than axum, and its filter composition model is harder to reason about for middleware stacking. Rejected for maintainability.
- hyper directly: Too low-level for the routing, middleware, and JSON extraction needed. Rejected for productivity.

## Decision 3: DNS Resolution - trust-dns-resolver (hickory-resolver)

**Decision**: Use trust-dns-resolver (now hickory-resolver) for async DNS TXT record lookups.

**Rationale**: It is the standard async DNS resolver for the Rust/tokio ecosystem. Supports TXT record queries needed for `_ax-cache.<domain>` discovery. Integrates with tokio runtime. The crate has been renamed to hickory-resolver but the API is stable.

**Alternatives considered**:
- System resolver via libc: Synchronous, would require spawning blocking tasks. No native TXT record query support. Rejected for async incompatibility.
- c-ares bindings: Lower-level, less idiomatic Rust API. Rejected for ergonomics.

## Decision 4: TLS Strategy - rustls

**Decision**: Use rustls (via reqwest's `rustls-tls` feature) instead of OpenSSL.

**Rationale**: rustls enables fully static binaries on Linux (musl) without OpenSSL linkage issues. This directly supports FR-024 (static binaries with musl libc). rustls is a pure-Rust TLS implementation with strong security properties and no C dependencies.

**Alternatives considered**:
- OpenSSL (native-tls): More battle-tested but creates static linking nightmares with musl. Cross-compilation becomes significantly harder. Rejected for distribution requirements.
- No TLS (plain HTTP): Not an option - AX protocol requires HTTPS (FR-014). Rejected for security.

## Decision 5: Cache Implementation - HashMap + lru crate

**Decision**: Use `std::collections::HashMap` wrapped in `Arc<RwLock<>>` for cache storage, with the `lru` crate for LRU eviction tracking.

**Rationale**: For the MVP in-memory cache, a HashMap provides O(1) lookups and the `lru` crate provides a well-tested LRU implementation. The `Arc<RwLock<>>` wrapper enables concurrent read access (multiple discovery requests) with exclusive write access (cache updates, evictions). This is the simplest correct approach for the MVP.

**Alternatives considered**:
- dashmap (concurrent HashMap): Better concurrent performance but adds complexity. The RwLock approach is sufficient for MVP scale (1000 concurrent requests). Can migrate later if benchmarks show contention.
- Custom LRU: More control but the `lru` crate is well-tested and widely used. Rejected for YAGNI.
- moka (concurrent cache): Full-featured concurrent cache with TTL support built-in. Strong alternative but adds a heavier dependency. Worth considering post-MVP if the manual TTL + LRU combination proves insufficient.

## Decision 6: Rate Limiting - Token Bucket Algorithm

**Decision**: Implement token bucket rate limiting in-process with per-client-IP tracking.

**Rationale**: Token bucket is the standard algorithm for rate limiting HTTP APIs. It naturally supports burst traffic while enforcing average rate limits. The in-process implementation (vs. external rate limiter like Redis) is appropriate for the MVP single-server deployment model. Per-IP tracking maps directly to FR-013.

**Alternatives considered**:
- Fixed window counter: Simpler but suffers from burst-at-boundary problems. Rejected for accuracy.
- Sliding window log: More accurate but higher memory usage per client. Rejected for MVP simplicity.
- External rate limiter (Redis): Appropriate for distributed deployments but adds infrastructure dependency. Rejected for MVP scope.

## Decision 7: Serialization Format - serde with JSON and YAML

**Decision**: Use serde with serde_json for AX records and serde_yaml for configuration files.

**Rationale**: serde is the de facto Rust serialization framework. JSON is required by the AX 1.0 specification for record format. YAML is the standard for human-authored configuration files. Both are well-supported by serde with derive macros for type-safe serialization.

**Alternatives considered**:
- TOML for configuration: Common in Rust projects but YAML handles nested structures (configuration inheritance) more naturally. Rejected for ergonomics with deep nesting.
- JSON for configuration: Valid but YAML is more readable for complex configurations with comments. Rejected for human authoring experience.

## Decision 8: Error Handling Strategy - thiserror + anyhow

**Decision**: Use `thiserror` for library error types and `anyhow` for CLI application errors.

**Rationale**: `thiserror` provides derive macros for creating structured, typed error enums - essential for a library where consumers need to match on error variants. `anyhow` provides ergonomic error handling for the CLI binary where error types don't need to be part of a public API. This follows the standard Rust convention: libraries use thiserror, applications use anyhow.

**Alternatives considered**:
- thiserror only: Viable but makes CLI error handling more verbose. Rejected for CLI ergonomics.
- Custom error types without macros: More control but significantly more boilerplate. Rejected for productivity.

## Decision 9: CLI Framework - clap with derive

**Decision**: Use clap 4.x with derive macros for the CLI interface.

**Rationale**: clap is the standard CLI framework in the Rust ecosystem. The derive API provides type-safe argument parsing with minimal boilerplate. It supports subcommands (generate, validate, discover, serve) and automatic help/version generation. The single-binary approach with subcommands follows the existing Kiro design.

**Alternatives considered**:
- argh: Lighter weight but less feature-rich (no auto-generated completions, less flexible validation). Rejected for feature completeness.
- structopt: Merged into clap 4. No longer a separate choice.

## Decision 10: Property Testing - proptest

**Decision**: Use proptest for property-based testing of core correctness properties.

**Rationale**: proptest is the most mature property-based testing framework for Rust. It supports strategic generation of complex data structures (AX records, configurations), shrinking for minimal failing cases, and integration with standard cargo test. The 26 correctness properties defined in the design document map directly to proptest test functions.

**Alternatives considered**:
- quickcheck: Older, less flexible generator API. proptest's strategy combinators are more ergonomic for complex types. Rejected for API quality.
- Manual fuzzing: Less systematic, doesn't provide shrinking. Rejected for completeness.

## Decision 11: Observability - tracing ecosystem

**Decision**: Use the `tracing` crate with `tracing-subscriber` for structured logging and the custom `SimpleHistogram` for request duration metrics.

**Rationale**: The tracing ecosystem is the standard for structured, span-based observability in async Rust. It integrates with tower middleware (TraceLayer) for automatic HTTP request logging. Structured fields (method, path, status_code, duration_ms, client_ip, cache_hit) are first-class citizens. The Prometheus text format metrics endpoint uses a simple custom histogram rather than pulling in the full prometheus crate, keeping dependencies minimal for MVP.

**Alternatives considered**:
- log crate: Simpler but lacks structured fields and span context. Rejected for observability requirements (FR-019).
- Full prometheus crate: More complete metrics but heavier dependency for MVP. The SimpleHistogram approach from the design doc is sufficient. Can upgrade post-MVP.
- OpenTelemetry: Full observability stack but significant complexity. Rejected for MVP scope.
