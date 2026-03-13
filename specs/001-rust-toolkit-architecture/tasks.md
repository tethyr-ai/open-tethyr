# Tasks: Open-Tethyr Rust Toolkit Architecture

**Input**: Design documents from `/specs/001-rust-toolkit-architecture/`
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/, research.md, quickstart.md

**Tests**: Property-based tests and integration tests are included as the feature specification (FR-025) and the Kiro design explicitly require comprehensive testing with proptest, wiremock, and criterion.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Cargo workspace initialization, dependency configuration, and project scaffolding

- [x] T001 Create root Cargo.toml with workspace configuration defining members ["crates/open-tethyr", "crates/cli"] and all shared workspace.dependencies (serde, serde_json, serde_yaml, reqwest, tokio, trust-dns-resolver, axum, tower, tower-http, clap, lru, tracing, tracing-subscriber, thiserror, anyhow, uuid, proptest, criterion, mockall, wiremock) in Cargo.toml
- [x] T002 [P] Create crates/open-tethyr/Cargo.toml with feature flags: default=["client"], client=[], server=["axum","tower","tower-http"], full=["client","server"]; reference workspace dependencies
- [x] T003 [P] Create crates/cli/Cargo.toml with binary name "open-tethyr", depending on open-tethyr with features=["full"] in crates/cli/Cargo.toml
- [x] T004 [P] Create library entry point with module declarations and feature-gated re-exports in crates/open-tethyr/src/lib.rs (pub mod ax, cache, config, dns, http, auth, error; #[cfg(feature="client")] pub mod client; #[cfg(feature="server")] pub mod server)
- [x] T005 [P] Create shared error types using thiserror: AxError, CacheError, ConfigError, DnsError, HttpError, ServerError, ClientError, OAuthError in crates/open-tethyr/src/error.rs
- [x] T006 [P] Create module entry points (mod.rs) for ax/, cache/, config/, dns/, http/, auth/, server/ directories under crates/open-tethyr/src/ with appropriate pub use re-exports
- [x] T007 [P] Create CLI entry point with clap Parser, Subcommand enum (Generate, Validate, Discover, Serve, CacheInvalidate), and tokio::main dispatch in crates/cli/src/main.rs and crates/cli/src/commands/mod.rs
- [x] T008 [P] Create directory scaffolding for tests/, examples/, docker/, .github/workflows/, crates/open-tethyr/tests/, crates/cli/tests/ with placeholder files
- [x] T009 Verify workspace builds with `cargo check --workspace --all-features` and `cargo check --workspace` (default features only)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core AX protocol types, configuration system, and shared infrastructure that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T010 Implement AX protocol data models: AgentExchangeRecord (record_type, version, agent, endpoints, capabilities, schema, limits, security, extensions), Agent (name, description, provider), Endpoint (protocol, url, auth, content_type), Protocol enum (Rest, GraphQL, MCP, A2A, Custom), AgentExchangeDocument (records), Capabilities, Schema, Limits, Security, OAuthEndpoints with serde derive annotations and default functions in crates/open-tethyr/src/ax/models.rs
- [x] T011 [P] Implement AxValidator with validate_record(), validate_agent(), validate_version(), validate_endpoints(), validate_auth_methods() enforcing record_type="AX", version="1.0", non-empty required fields, auth methods from {OIDC, OAuth2, mTLS, JWT, API_KEY} in crates/open-tethyr/src/ax/validator.rs
- [x] T012 [P] Implement configuration data models: AgentConfig (defaults, agents), AgentDefaults (provider, auth, endpoints, capabilities, limits, extensions), AgentDefinition (name, description, provider, endpoints, capabilities, limits, security, extensions), EndpointDefinition, ServerConfig (domain, port, cache, policy, rate_limit, log_level), CacheConfig (max_entries, default_ttl, cleanup_interval, enable_lru), PolicyConfig (domain_locking, home_domain, allowlist), RateLimitConfig (requests_per_minute, requests_per_hour) with serde derive in crates/open-tethyr/src/config/models.rs
- [x] T013 [P] Implement ConfigValidator with validate_config(), validate_domain() (DNS format regex), validate_url() (HTTPS scheme check), validate_port() (1-65535 range), validate_ttl() (positive integer) in crates/open-tethyr/src/config/validator.rs
- [x] T014 Implement ConfigMerger with merge_agent() and merge_auth() for type-safe configuration inheritance: agent-specific values override defaults, nested objects merge field-by-field, Vec fields replaced not merged, missing fields fall back to defaults in crates/open-tethyr/src/config/merger.rs
- [x] T015 [P] Implement OAuthProvider trait (generate_endpoints, provider_name), OAuthConfig struct, ProviderRegistry (new with pre-loaded providers, register_provider, generate_oauth_config) in crates/open-tethyr/src/auth/provider.rs
- [x] T016 [P] Implement OktaProvider generating endpoints: issuer=https://{domain}, authorization=https://{domain}/oauth2/authorize, token=https://{domain}/oauth2/token, jwks=https://{domain}/oauth2/v1/keys, userinfo=https://{domain}/oauth2/v1/userinfo, revocation=https://{domain}/oauth2/v1/revoke in crates/open-tethyr/src/auth/okta.rs
- [x] T017 [P] Implement Auth0Provider generating endpoints: issuer=https://{domain}/, authorization=https://{domain}/authorize, token=https://{domain}/oauth/token, jwks=https://{domain}/.well-known/jwks.json, userinfo=https://{domain}/userinfo, revocation=https://{domain}/oauth/revoke in crates/open-tethyr/src/auth/auth0.rs
- [x] T018 [P] Implement GenericOAuth2Provider for RFC 8414 compliant providers with configurable base URL and standard endpoint paths in crates/open-tethyr/src/auth/generic.rs
- [x] T019 [P] Implement DnsDiscovery with TokioAsyncResolver: discover_cache() for _ax-cache.{domain} TXT lookup, discover_root_cache(), parse_cache_endpoint() parsing "endpoint=<url>" format, graceful error handling returning None on DNS failures in crates/open-tethyr/src/dns/discovery.rs
- [x] T020 [P] Implement AxHttpClient with configurable timeout (default 30s), fetch_ax_record() building URL https://_agent.{domain}/.well-known/agent-exchange.json, fetch_from_cache(), build_ax_url(), validate_well_known_path(), HTTPS certificate validation via rustls in crates/open-tethyr/src/http/client.rs
- [x] T021 Implement AxGenerator with generate_record() (config to AgentExchangeDocument using ConfigMerger) and generate_well_known_structure() (document to WellKnownFiles), and FileWriter with write_structure() and write_ax_record() for disk output in crates/open-tethyr/src/ax/generator.rs
- [x] T022 Write property test (Property 8): serialization round-trip consistency for AgentExchangeRecord - serialize to JSON then deserialize, verify all fields preserved including record_type, version, agent.name, endpoints.len() in crates/open-tethyr/tests/property_serialization.rs
- [x] T023 [P] Write property test (Property 2): AX record validation correctness - valid records pass, records with missing record_type/invalid version/malformed structure fail with correct errors in crates/open-tethyr/tests/property_validation.rs
- [x] T024 [P] Write property test (Property 10): configuration inheritance correctness - merged config inherits defaults for missing fields, preserves overrides, handles nested objects in crates/open-tethyr/tests/property_config.rs
- [x] T025 [P] Write property test (Property 20): configuration validation - valid domains/ports/URLs/TTLs pass, invalid ones rejected with correct errors in crates/open-tethyr/tests/property_config_validation.rs
- [x] T026 [P] Write property test (Property 9): OAuth provider template correctness - each provider generates valid HTTPS URLs following provider-specific patterns in crates/open-tethyr/tests/property_oauth.rs
- [x] T027 [P] Write property test (Property 18): DNS TXT record parsing - "endpoint=<url>" format correctly extracted, malformed records rejected in crates/open-tethyr/tests/property_dns.rs
- [x] T028 [P] Write property test (Property 13): AX subdomain URL construction - build_ax_url correctly formats https://_agent.{domain}/.well-known/agent-exchange.json for any valid domain in crates/open-tethyr/tests/property_url.rs
- [x] T029 Create test utilities: MockDnsResolver, test fixtures for valid/invalid AX records, arb_agent_record/arb_agent/arb_endpoint/arb_auth_method/arb_domain/arb_valid_yaml_config proptest generators in crates/open-tethyr/tests/test_utils.rs
- [x] T030 [P] Implement environment variable configuration support: OPEN_TETHYR_LOG, OPEN_TETHYR_CONFIG, OPEN_TETHYR_DOMAIN, OPEN_TETHYR_PORT with precedence CLI > env > config file > defaults (constitution mandate) in crates/cli/src/main.rs
- [x] T031 Verify all foundational tests pass with `cargo test --workspace --all-features`

**Checkpoint**: Foundation ready - all core types, validation, configuration, OAuth, DNS, HTTP client, and generators working. User story implementation can now begin.

---

## Phase 3: User Story 1 - Generate AX Discovery Records from Configuration (Priority: P1)

**Goal**: System administrators can create YAML configs and generate valid AX 1.0 JSON records at well-known URI paths with configuration inheritance and OAuth provider templates.

**Independent Test**: Write a YAML config file, run `open-tethyr generate --config agents.yaml --output ./public --validate`, verify output JSON matches AX 1.0 specification.

### Tests for User Story 1

- [x] T032 [P] [US1] Write property test (Property 1): AX record generation correctness - for any valid YAML config, generated document has record_type="AX", version="1.0", valid structure per AX RFC in crates/open-tethyr/tests/property_generation.rs
- [x] T033 [P] [US1] Write property test (Property 24): AX version validation and handling - version="1.0" accepted, unsupported versions logged and skipped in crates/open-tethyr/tests/property_version.rs
- [x] T034 [P] [US1] Write property test (Property 3): well-known file structure generation - output includes /.well-known/agent-exchange.json path with correct directory structure in crates/open-tethyr/tests/property_wellknown.rs

### Implementation for User Story 1

- [x] T035 [US1] Implement GenerateCommand with clap Args: --config (PathBuf, required), --output (PathBuf, required), --validate (bool flag); execute() loads YAML config via ConfigMerger, generates via AxGenerator, optionally validates via AxValidator, writes via FileWriter in crates/cli/src/commands/generate.rs
- [x] T036 [US1] Implement basic ValidateCommand with clap Args: positional path to AX JSON file; execute() reads file, parses JSON, runs AxValidator::validate_record() on each record, reports errors to stderr with field paths in crates/cli/src/commands/validate.rs (Note: provides minimal validate for --validate flag in generate; US2 enhances this with detailed reporting)
- [ ] T037 [US1] Write CLI integration test: generate command produces valid AX JSON from sample YAML config, validate command passes on generated output, validate command fails on intentionally invalid input in crates/cli/tests/test_generate_validate.rs

**Checkpoint**: User Story 1 complete - administrators can generate and validate AX records via CLI.

---

## Phase 4: User Story 2 - Validate Existing AX Records (Priority: P1)

**Goal**: System integrators can validate received AX records for AX 1.0 compliance with clear error reporting.

**Independent Test**: Provide various AX JSON files (valid, missing fields, wrong version, bad auth methods) to the validate command and verify correct pass/fail with specific error messages.

### Tests for User Story 2

- [x] T038 [P] [US2] Write property test (Property 25): auth method validation - auth methods from {OIDC, OAuth2, mTLS, JWT, API_KEY} accepted, others rejected in crates/open-tethyr/tests/property_auth_validation.rs

### Implementation for User Story 2

- [x] T039 [US2] Enhance AxValidator to produce detailed ValidationReport with per-field error paths, severity levels (error vs warning), and human-readable messages for: missing required fields, invalid record_type, unsupported version (warning), invalid auth methods, empty endpoints in crates/open-tethyr/src/ax/validator.rs
- [x] T040 [US2] Enhance ValidateCommand output formatting: summary line for valid records, itemized error list for invalid records with field path and description, exit code 0 for valid / 3 for invalid in crates/cli/src/commands/validate.rs
- [ ] T041 [US2] Write integration test: validate command with valid AX record exits 0, with missing agent.name exits 3, with version "2.0" shows warning, with auth ["INVALID"] reports error in crates/cli/tests/test_validate_detailed.rs

**Checkpoint**: User Story 2 complete - integrators can validate any AX record with detailed error reporting.

---

## Phase 5: User Story 3 - Deploy and Operate a Cache Server (Priority: P1)

**Goal**: Infrastructure operators can deploy a cache server with in-memory storage, LRU eviction, TTL, hierarchical caching, policy enforcement, and rate limiting.

**Independent Test**: Start cache server, send discovery requests, verify cache hits/misses/TTL/eviction, test policy enforcement responses, verify hierarchical fallback.

### Tests for User Story 3

- [ ] T042 [P] [US3] Write property test (Property 4): cache-first discovery - when cache has entry, no external fetch occurs in crates/open-tethyr/tests/property_cache_first.rs
- [ ] T043 [P] [US3] Write property test (Property 5): cache miss fallback - on miss, system fetches from correct AX endpoint and validates response path in crates/open-tethyr/tests/property_cache_miss.rs
- [ ] T044 [P] [US3] Write property test (Property 7): TTL expiration - expired records not returned, trigger re-fetch; non-expired served from cache in crates/open-tethyr/tests/property_ttl.rs
- [ ] T045 [P] [US3] Write property test (Property 15): LRU eviction - at max capacity, least recently used evicted, most recently accessed preserved in crates/open-tethyr/tests/property_lru.rs
- [ ] T046 [P] [US3] Write property test (Property 26): cache size limit enforcement - cache never exceeds max_entries in crates/open-tethyr/tests/property_cache_size.rs
- [ ] T047 [P] [US3] Write property test (Property 16): Cache-Control header compliance - respects max-age and no-cache directives in crates/open-tethyr/tests/property_cache_control.rs
- [ ] T048 [P] [US3] Write property test (Property 11): hierarchical cache fallback chain - local miss -> root -> direct, upstream failure falls back gracefully in crates/open-tethyr/tests/property_hierarchy.rs
- [ ] T049 [P] [US3] Write property test (Property 12): circular dependency prevention - cyclic cache configs detected and rejected at startup in crates/open-tethyr/tests/property_circular.rs
- [ ] T050 [P] [US3] Write property test (Property 6): domain locking policy enforcement - locked domain rejects external requests, allowlisted domains permitted in crates/open-tethyr/tests/property_policy.rs
- [ ] T051 [P] [US3] Write property test (Property 21): rate limiting enforcement - exceeding limit returns rejection, tokens refill over time in crates/open-tethyr/tests/property_rate_limit.rs
- [ ] T052 [P] [US3] Write property test (Property 23): HTTP error response mapping - PolicyViolation->403, RateLimitExceeded->429, NotFound->404, upstream failure->502 in crates/open-tethyr/tests/property_error_responses.rs
- [ ] T053 [P] [US3] Write property test (Property 22): request timeout handling - external fetches timeout after configured duration in crates/open-tethyr/tests/property_timeout.rs
- [ ] T054 [P] [US3] Write property test (Property 14): HTTPS certificate validation - invalid certificates rejected in crates/open-tethyr/tests/property_https.rs

### Implementation for User Story 3

- [x] T055 [US3] Implement MemoryCache with Arc<RwLock<HashMap<String, CacheEntry>>> and lru::LruCache: get() checks TTL, put() with LRU eviction when max_entries reached, invalidate(), clear(), size() in crates/open-tethyr/src/cache/memory.rs
- [x] T056 [US3] Implement CacheStats with AtomicUsize/AtomicU64 counters: total_entries, hit_count, miss_count, eviction_count, memory_usage_bytes; record_hit(), record_miss(), record_eviction(), update_memory_usage() in crates/open-tethyr/src/cache/stats.rs
- [x] T057 [US3] Implement CacheCoordinator with local MemoryCache, optional root_cache_url (from DNS), fallback chain: local -> root cache -> direct fetch; detect circular deps via DFS during init in crates/open-tethyr/src/cache/coordinator.rs
- [x] T058 [US3] Implement TokenBucket (tokens, capacity, refill_rate, last_refill) with consume() and refill(); RateLimiter with Arc<RwLock<HashMap<IpAddr, TokenBucket>>>, check_rate_limit(), reset_limits() in crates/open-tethyr/src/cache/rate_limiter.rs
- [x] T059 [US3] Implement PolicyEngine with domain_locking flag, home_domain, allowlist; check_discovery_allowed() returning Ok or PolicyViolation error in crates/open-tethyr/src/server/policy.rs
- [x] T060 [US3] Implement CacheServer struct (cache, coordinator, policy, rate_limiter, metrics, config), new() initialization, start() with axum Router, build_routes() wiring /discover/:domain, /health, /metrics in crates/open-tethyr/src/server/cache_server.rs
- [x] T061 [US3] Implement request handlers: handle_discover() with correlation ID, rate limiting, policy check, coordinator.discover(), structured logging; handle_health() returning JSON status; handle_metrics() in Prometheus text format in crates/open-tethyr/src/server/handlers.rs
- [x] T062 [US3] Implement middleware: RateLimitLayer wrapping rate limiter check, CorrelationIdLayer adding X-Correlation-Id to responses, request timeout via tower::timeout in crates/open-tethyr/src/server/middleware.rs
- [x] T063 [US3] Implement ServerError IntoResponse mapping: PolicyViolation->403, RateLimitExceeded->429, CacheError::NotFound->404, upstream failures->502; JSON body with error message, timestamp, correlation_id in crates/open-tethyr/src/server/cache_server.rs
- [x] T064 [US3] Implement CacheMetrics with AtomicU64 hit/miss counters, SimpleHistogram for request duration (buckets: 1ms, 5ms, 10ms, 50ms, 100ms, 500ms, 1s, 5s), AtomicU32 active_connections in crates/open-tethyr/src/server/handlers.rs
- [x] T065 [US3] Implement ServeCommand with clap Args: --domain, --port (default 8080), --config (optional YAML path), --max-entries (default 10000), --ttl (default 3600); execute() builds ServerConfig, starts CacheServer in crates/cli/src/commands/serve.rs
- [ ] T066 [US3] Write integration test: start cache server, send discovery request via HTTP, verify cache miss triggers upstream fetch (wiremock), second request hits cache, verify /health and /metrics endpoints respond correctly in crates/open-tethyr/tests/integration_server.rs
- [ ] T067 [US3] Write integration test: cache server with domain_locking=true rejects external domain requests with 403, allows home_domain and allowlisted domains in crates/open-tethyr/tests/integration_policy.rs

**Checkpoint**: User Story 3 complete - operators can deploy cache server with full caching, policy enforcement, rate limiting, and observability.

---

## Phase 6: User Story 4 - Discover Agents Using the Client Library (Priority: P2)

**Goal**: Rust developers can use the OpenTethyr client SDK to discover agents with automatic DNS-based cache discovery and graceful fallback.

**Independent Test**: Initialize client with a domain, verify DNS-based cache discovery, confirm fallback when no cache exists.

### Tests for User Story 4

- [ ] T068 [P] [US4] Write property test (Property 17): DNS cache discovery routing - cache found via DNS -> requests go through cache; no cache -> direct discovery in crates/open-tethyr/tests/property_client_routing.rs
- [ ] T069 [P] [US4] Write property test (Property 19): DNS discovery error resilience - DNS failures don't propagate, client falls back to direct discovery in crates/open-tethyr/tests/property_dns_resilience.rs

### Implementation for User Story 4

- [x] T070 [US4] Implement OpenTethyr client struct (domain, cache_url, http_client, dns_discovery); new() with automatic DNS cache discovery; discover() trying cache then direct; discover_with_cache() for explicit cache URL in crates/open-tethyr/src/client.rs
- [ ] T071 [US4] Write integration test: OpenTethyr client with mock DNS returning cache endpoint routes requests to cache (wiremock); client with no DNS record falls back to direct fetch; client with unreachable cache falls back to direct in crates/open-tethyr/tests/integration_client.rs

**Checkpoint**: User Story 4 complete - developers can use the client SDK for agent discovery.

---

## Phase 7: User Story 5 - Test Agent Discovery End-to-End (Priority: P2)

**Goal**: Administrators can test discovery against domains using the CLI discover command with optional cache specification.

**Independent Test**: Run discover command against a domain hosting AX records, verify agent information displayed.

### Implementation for User Story 5

- [x] T072 [US5] Implement DiscoverCommand with clap Args: positional DOMAIN, --cache (optional URL), --direct (skip cache), --json (JSON output), --timeout (default 30s); execute() uses OpenTethyr client or direct fetch, formats output in crates/cli/src/commands/discover.rs
- [ ] T073 [US5] Write CLI integration test: discover command with mock AX endpoint (wiremock) displays agent names and endpoints; --json flag outputs valid JSON; --cache flag routes through specified cache URL; domain with no AX records reports "no agents found" in crates/cli/tests/test_discover.rs

**Checkpoint**: User Story 5 complete - administrators can test discovery end-to-end via CLI.

---

## Phase 8: User Story 6 - Monitor and Operate the System (Priority: P2)

**Goal**: Operators have structured logging with correlation IDs, metrics endpoint, and cache management capabilities.

**Independent Test**: Start server, generate traffic, verify logs have structured fields and metrics endpoint returns valid data.

### Implementation for User Story 6

- [ ] T074 [US6] Configure tracing-subscriber with JSON format, configurable log level from ServerConfig.log_level and OPEN_TETHYR_LOG env var, structured fields for all HTTP requests (method, path, status_code, duration_ms, client_ip, cache_hit) in crates/open-tethyr/src/server/cache_server.rs
- [ ] T075 [US6] Add audit logging: log every discovery request with domain, client_ip, policy_result, cache_result, duration using tracing::info_span and structured fields in crates/open-tethyr/src/server/handlers.rs
- [ ] T076 [US6] Implement cache invalidation CLI subcommand `open-tethyr cache-invalidate` with clap Args: --domain <DOMAIN> (invalidate single domain), --all (clear entire cache), --server <URL> (target cache server, required); execute() sends DELETE to /cache/:domain or /cache endpoints on the target server; add corresponding DELETE handlers in crates/open-tethyr/src/server/handlers.rs and wire routes in cache_server.rs; CLI command in crates/cli/src/commands/cache_invalidate.rs
- [ ] T077 [US6] Write integration test: start server, send requests, capture structured logs and verify they contain required fields (method, path, status_code, duration_ms, client_ip, cache_hit, correlation_id); verify /metrics returns updated counters in crates/open-tethyr/tests/integration_observability.rs

**Checkpoint**: User Story 6 complete - operators have full observability and cache management.

---

## Phase 9: User Story 7 - Build and Deploy Across Platforms (Priority: P3)

**Goal**: DevOps engineers can build cross-platform static binaries and Docker images via CI/CD.

**Independent Test**: Run build pipeline, verify binaries for Linux/macOS/Windows and Docker images start correctly.

### Implementation for User Story 7

- [ ] T078 [P] [US7] Create CI workflow: cargo fmt --check, cargo clippy --workspace --all-features, cargo test --workspace --all-features in .github/workflows/ci.yml
- [ ] T079 [P] [US7] Create release workflow: cross-compile for x86_64-unknown-linux-musl (static), x86_64-apple-darwin, x86_64-pc-windows-msvc; upload binaries as GitHub release assets in .github/workflows/release.yml
- [ ] T080 [P] [US7] Create Docker workflow: multi-stage Dockerfile (builder with musl -> minimal runtime), push to ghcr.io in .github/workflows/docker.yml and docker/Dockerfile
- [ ] T081 [US7] Create performance benchmarks with criterion: bench_cache_put, bench_cache_get, bench_cache_evict, bench_ax_parse, bench_ax_validate, bench_config_merge in crates/open-tethyr/benches/benchmarks.rs

**Checkpoint**: User Story 7 complete - CI/CD pipeline produces cross-platform binaries and Docker images.

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, cross-story integration, and quality improvements

- [ ] T082 Write workspace-level integration test: full end-to-end flow - generate AX records from YAML config, validate them, start cache server, discover through cache, verify correctness in tests/integration_e2e.rs
- [ ] T083 [P] Write workspace-level integration test: hierarchical cache - start root cache (wiremock), start regional cache pointing to root, verify fallback chain (local -> root -> direct) and graceful degradation when root unavailable in tests/integration_hierarchy.rs
- [ ] T084 [P] Write workspace-level integration test: OAuth provider templates integrated with generation - YAML config referencing Okta/Auth0 providers generates AX records with correct security.oauth endpoints in tests/integration_oauth.rs
- [ ] T085 [P] Write concurrency load test: spawn 1,000 concurrent tokio tasks each sending a discovery request to the cache server (wiremock upstream), verify all requests complete without errors or panics (validates SC-003) in tests/integration_concurrency.rs
- [ ] T086 [P] Add criterion benchmarks for success criteria timing validation: bench_dns_discovery_with_fallback (SC-004 <2s), bench_hierarchical_fallback_chain (SC-006 <5s), bench_server_cold_start (SC-008 <3s), bench_cli_validate (SC-009 <500ms) in crates/open-tethyr/benches/timing_benchmarks.rs
- [ ] T087 Run quickstart.md validation: execute each quickstart scenario and verify expected outcomes
- [ ] T088 Run `cargo clippy --workspace --all-features -- -D warnings` and fix all warnings
- [ ] T089 Run `cargo fmt --all` and verify formatting
- [ ] T090 Verify all property tests pass with extended iterations: `PROPTEST_CASES=1000 cargo test --workspace --all-features`
- [ ] T091 Measure code coverage with cargo-tarpaulin or cargo-llvm-cov: run `cargo tarpaulin --workspace --all-features --out Html` and verify at least 80% line coverage across all crates (validates SC-014)
- [ ] T092 Final validation: `cargo test --workspace --all-features` passes all tests, `cargo build --release --workspace` produces binaries under 50MB

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **US1 Generate Records (Phase 3)**: Depends on Foundational (Phase 2)
- **US2 Validate Records (Phase 4)**: Depends on Foundational (Phase 2); optionally after US1 for shared validate command
- **US3 Cache Server (Phase 5)**: Depends on Foundational (Phase 2)
- **US4 Client SDK (Phase 6)**: Depends on Foundational (Phase 2); benefits from US3 for cache testing
- **US5 Discovery CLI (Phase 7)**: Depends on US4 (uses OpenTethyr client)
- **US6 Monitoring (Phase 8)**: Depends on US3 (extends server)
- **US7 Build/Deploy (Phase 9)**: Can start after Foundational; CI/CD independent of feature code
- **Polish (Phase 10)**: Depends on all desired user stories being complete

### User Story Dependencies

- **US1 (P1)**: Foundational only - fully independent
- **US2 (P1)**: Foundational only - shares validator with US1 but independently testable
- **US3 (P1)**: Foundational only - fully independent
- **US4 (P2)**: Foundational only - can test with mocks, no hard dependency on US3
- **US5 (P2)**: Depends on US4 (client SDK) for discover command implementation
- **US6 (P2)**: Depends on US3 (server must exist to add observability)
- **US7 (P3)**: Foundational only - CI/CD pipeline is independent

### Within Each User Story

- Property tests written alongside or before implementation
- Models/types before services
- Services before CLI commands
- Core implementation before integration tests
- Story complete at checkpoint

### Parallel Opportunities

- **Phase 1**: T002, T003, T004, T005, T006, T007, T008 all in parallel (different files)
- **Phase 2**: T011-T029 extensively parallelizable (different files, independent modules)
- **Phase 3-4**: US1 and US2 can run in parallel (different CLI commands, shared validator)
- **Phase 3+5**: US1 and US3 can run in parallel (CLI generation vs server, no overlap)
- **Phase 5 tests**: T042-T054 all in parallel (different test files)
- **Phase 5 impl**: T055-T059 partially parallel (different modules)
- **Phase 6-7**: US4 and US7 can run in parallel (client SDK vs CI/CD, no overlap)
- **Phase 9**: T078, T079, T080 all in parallel (different workflow files)
- **Phase 10**: T082, T083, T084, T085, T086 all in parallel (different test files)

---

## Parallel Example: User Story 3

```bash
# Launch all property tests in parallel (different files):
Task T042: "Property test cache-first discovery in property_cache_first.rs"
Task T043: "Property test cache miss fallback in property_cache_miss.rs"
Task T044: "Property test TTL expiration in property_ttl.rs"
Task T045: "Property test LRU eviction in property_lru.rs"
Task T046: "Property test cache size limit in property_cache_size.rs"
Task T047: "Property test Cache-Control headers in property_cache_control.rs"
Task T048: "Property test hierarchical fallback in property_hierarchy.rs"
Task T049: "Property test circular dependency in property_circular.rs"
Task T050: "Property test domain locking in property_policy.rs"
Task T051: "Property test rate limiting in property_rate_limit.rs"
Task T052: "Property test error responses in property_error_responses.rs"
Task T053: "Property test request timeout in property_timeout.rs"
Task T054: "Property test HTTPS validation in property_https.rs"

# Launch parallelizable implementation tasks:
Task T055: "MemoryCache in cache/memory.rs"
Task T056: "CacheStats in cache/stats.rs"
Task T058: "RateLimiter in cache/rate_limiter.rs"
Task T059: "PolicyEngine in server/policy.rs"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Generate records)
4. Complete Phase 4: User Story 2 (Validate records)
5. **STOP and VALIDATE**: Test record generation and validation independently
6. Deploy/demo if ready - administrators can publish and validate AX records

### Core Product (Add User Story 3)

7. Complete Phase 5: User Story 3 (Cache server)
8. **STOP and VALIDATE**: Cache server with full caching, policies, rate limiting
9. System is now functionally complete for infrastructure operators

### Full Product (Add User Stories 4-7)

10. Complete Phase 6: User Story 4 (Client SDK)
11. Complete Phase 7: User Story 5 (Discovery CLI)
12. Complete Phase 8: User Story 6 (Monitoring)
13. Complete Phase 9: User Story 7 (Build/Deploy)
14. Complete Phase 10: Polish
15. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Generate) + User Story 2 (Validate)
   - Developer B: User Story 3 (Cache Server)
   - Developer C: User Story 7 (CI/CD) + User Story 4 (Client SDK)
3. After US3 + US4 complete:
   - Developer A: User Story 5 (Discovery CLI)
   - Developer B: User Story 6 (Monitoring)
   - Developer C: Polish

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] label maps task to specific user story for traceability
- Property tests follow the 26 correctness properties defined in the Kiro design document
- Each user story should be independently completable and testable at its checkpoint
- The Kiro tasks breakdown was consulted for requirement traceability and property test numbering
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
