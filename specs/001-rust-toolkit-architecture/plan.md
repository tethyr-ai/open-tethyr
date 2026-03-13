# Implementation Plan: Open-Tethyr Rust Toolkit Architecture

**Branch**: `001-rust-toolkit-architecture` | **Date**: 2026-03-12 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-rust-toolkit-architecture/spec.md`
**Reference Design**: `.kiro/specs/rust-toolkit-architecture/design.md`

## Summary

Open-tethyr is a distributed caching system for agent discovery implementing the AX (Agent Discovery Exchange) protocol. The system provides a Rust Cargo workspace with two crates: a core library (`open-tethyr`) with AX protocol models, validation, DNS discovery, HTTP client, OAuth templates, and cache logic; and a CLI binary (`open-tethyr`) for record generation, validation, discovery testing, and server management. The cache server is embedded in the library behind a feature flag, providing hierarchical caching, policy enforcement, rate limiting, and structured observability. The architecture uses Rust feature flags to offer a lightweight client SDK by default and opt-in server functionality.

## Technical Context

**Language/Version**: Rust (stable toolchain, edition 2021)
**Primary Dependencies**: serde/serde_json/serde_yaml (serialization), reqwest with rustls-tls (HTTP), trust-dns-resolver (DNS), axum/tower/tower-http (server, opt-in), clap (CLI), lru (cache eviction), tracing/tracing-subscriber (logging), thiserror/anyhow (errors), uuid (correlation IDs), proptest (property testing), criterion (benchmarks), wiremock/mockall (test mocks)
**Storage**: In-memory (HashMap + LRU) for MVP; no persistent storage
**Testing**: cargo test + proptest (property-based) + wiremock (HTTP mocks) + criterion (benchmarks)
**Target Platform**: Linux (musl static), macOS, Windows; Docker containers (amd64)
**Project Type**: Library + CLI + embedded server (Cargo workspace)
**Performance Goals**: <10ms cached lookups, 1000 concurrent requests, <1s record generation for 100 agents
**Constraints**: <50MB static binaries, 30s default request timeout, AX 1.0 protocol compliance
**Scale/Scope**: MVP targeting single-organization deployments with hierarchical cache topology

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*
*Validated against: constitution v1.0.0 (ratified 2026-03-13)*

| Principle | Pre-Phase 0 | Post-Phase 1 | Evidence |
|-----------|-------------|--------------|----------|
| I. Library-First Architecture | PASS | PASS | Two-crate workspace, feature flags (client/server/full), lib.rs re-exports, no circular deps |
| II. AX Protocol Compliance | PASS | PASS | record_type="AX", version="1.0" enforced, approved auth methods only, HTTPS mandatory, well-known paths |
| III. Property-Based Testing | PASS | PASS | 26 property tests planned with proptest, custom generators, wiremock/mockall for mocks |
| IV. Simplicity/YAGNI | PASS | PASS | In-memory HashMap+LRU only, no premature trait abstractions, all deps justified by requirements |
| V. Structured Observability | PASS | PASS | Correlation IDs (UUID v4), structured log fields, /metrics in Prometheus format, audit logging |
| VI. Static Distribution | PASS | PASS | musl static linking, rustls TLS, multi-stage Docker, <50MB binaries |
| Performance Standards | PASS | PASS | Criterion benchmarks for all 6 thresholds, concurrency load test for 1000 req |
| Development Workflow | PASS | PASS | clippy -D warnings, cargo fmt, full test suite, thiserror in lib / anyhow in CLI, config precedence enforced |

## Project Structure

### Documentation (this feature)

```text
specs/001-rust-toolkit-architecture/
├── plan.md              # This file
├── research.md          # Phase 0 output - technology decisions
├── data-model.md        # Phase 1 output - entity definitions
├── quickstart.md        # Phase 1 output - getting started guide
├── contracts/           # Phase 1 output - interface contracts
│   ├── cli-contract.md  # CLI command interface
│   ├── http-api.md      # Cache server HTTP API
│   └── library-api.md   # Public Rust library API
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
open-tethyr/
├── Cargo.toml                    # Workspace root with shared dependencies
├── Cargo.lock
├── .github/
│   └── workflows/
│       ├── ci.yml                # CI: test, lint, format
│       ├── release.yml           # Release: cross-compile, publish
│       └── docker.yml            # Docker: build and push images
├── crates/
│   ├── open-tethyr/              # Core library crate
│   │   ├── Cargo.toml            # Features: default=["client"], server, full
│   │   ├── src/
│   │   │   ├── lib.rs            # Public API re-exports
│   │   │   ├── ax/               # AX protocol: models, validator, generator
│   │   │   │   ├── mod.rs
│   │   │   │   ├── models.rs     # AgentExchangeRecord, Agent, Endpoint, Protocol
│   │   │   │   ├── validator.rs  # AxValidator
│   │   │   │   └── generator.rs  # AxGenerator, FileWriter
│   │   │   ├── cache/            # Cache: storage, coordination, rate limiting
│   │   │   │   ├── mod.rs
│   │   │   │   ├── memory.rs     # MemoryCache with LRU
│   │   │   │   ├── coordinator.rs # CacheCoordinator (hierarchical fallback)
│   │   │   │   ├── rate_limiter.rs # TokenBucket rate limiting
│   │   │   │   └── stats.rs      # CacheStats with atomic counters
│   │   │   ├── config/           # Configuration: models, merger, validator
│   │   │   │   ├── mod.rs
│   │   │   │   ├── models.rs     # AgentConfig, AgentDefaults, ServerConfig
│   │   │   │   ├── merger.rs     # ConfigMerger (type-safe inheritance)
│   │   │   │   └── validator.rs  # ConfigValidator (domain, port, URL, TTL)
│   │   │   ├── dns/              # DNS discovery
│   │   │   │   ├── mod.rs
│   │   │   │   └── discovery.rs  # DnsDiscovery (_ax-cache TXT lookup)
│   │   │   ├── http/             # HTTP client
│   │   │   │   ├── mod.rs
│   │   │   │   └── client.rs     # AxHttpClient (fetch, validate paths)
│   │   │   ├── auth/             # OAuth provider templates
│   │   │   │   ├── mod.rs
│   │   │   │   ├── provider.rs   # OAuthProvider trait, ProviderRegistry
│   │   │   │   ├── okta.rs       # OktaProvider
│   │   │   │   ├── auth0.rs      # Auth0Provider
│   │   │   │   └── generic.rs    # GenericOAuth2Provider (RFC 8414)
│   │   │   ├── client.rs         # OpenTethyr SDK (#[cfg(feature = "client")])
│   │   │   ├── error.rs          # Shared error types (AxError, CacheError, etc.)
│   │   │   └── server/           # Cache server (#[cfg(feature = "server")])
│   │   │       ├── mod.rs
│   │   │       ├── cache_server.rs # CacheServer (axum routes, lifecycle)
│   │   │       ├── handlers.rs   # Request handlers (discover, health, metrics)
│   │   │       ├── middleware.rs  # Rate limit layer, correlation ID layer
│   │   │       └── policy.rs     # PolicyEngine (domain locking, allowlists)
│   │   └── tests/                # Integration tests for core library
│   └── cli/                      # CLI binary crate
│       ├── Cargo.toml            # Depends on open-tethyr with features = ["full"]
│       ├── src/
│       │   ├── main.rs           # Entry point, clap CLI dispatch
│       │   └── commands/
│       │       ├── mod.rs
│       │       ├── generate.rs          # open-tethyr generate
│       │       ├── validate.rs          # open-tethyr validate
│       │       ├── discover.rs          # open-tethyr discover
│       │       ├── serve.rs             # open-tethyr serve
│       │       └── cache_invalidate.rs  # open-tethyr cache-invalidate
│       └── tests/                # CLI integration tests
├── tests/                        # Workspace-level integration tests
├── examples/                     # Usage examples
└── docker/
    └── Dockerfile                # Multi-stage build with musl
```

**Structure Decision**: Cargo workspace with two crates (`open-tethyr` library + `cli` binary). The library uses feature flags (`client` default, `server` opt-in) to support lightweight SDK usage without pulling in server dependencies. This matches the existing Kiro design document architecture and follows Rust ecosystem conventions for library+binary workspace layout.

## Complexity Tracking

No constitution violations identified. All 6 principles and both supplementary sections (Performance Standards, Development Workflow) are satisfied by the current design. See Constitution Check table above for evidence.
