# Open-Tethyr Testing Strategy

## Overview

Open-tethyr is critical infrastructure for agent discovery. The test suite must provide confidence that:

1. The AX protocol implementation is correct and spec-compliant
2. The system handles malformed, adversarial, and unexpected input safely
3. HTTP communication works correctly with real servers
4. The CLI binary is functional and produces correct exit codes
5. Cache behavior (LRU, TTL, hierarchy) is mathematically sound
6. Policy enforcement and rate limiting work under all conditions

## Test Suite: 145 Tests Across 4 Tiers

### Tier 1: Property Tests (26 files, ~70 tests)

**Purpose**: Verify invariants hold across the entire input space.
**When**: Every commit, every PR, every local `make test`.

| Test File | What It Verifies |
|-----------|-----------------|
| `property_serialization.rs` | AX record, capabilities, schema, limits, security all survive JSON round-trip |
| `property_validation.rs` | Valid records pass, invalid rejected with correct errors |
| `property_auth_validation.rs` | Known auth methods accepted, unknown produce warnings |
| `property_url.rs` | AX URLs always follow `https://<domain>/.well-known/agent-exchange` |
| `property_wellknown.rs` | Generated files land at correct path |
| `property_generation.rs` | Generator produces valid flat AX documents |
| `property_config.rs` | Configuration inheritance works (defaults, overrides) |
| `property_config_validation.rs` | Domain, port, URL, TTL validation |
| `property_oauth.rs` | OAuth providers return correct issuer + jwks_url |
| `property_dns.rs` | DNS TXT record parsing |
| `property_version.rs` | AX version validation |
| `property_cache_first.rs` | Cache hits return data, misses return None |
| `property_cache_miss.rs` | Empty cache returns None |
| `property_ttl.rs` | Expired entries not returned |
| `property_lru.rs` | LRU evicts least recently used |
| `property_cache_size.rs` | Cache never exceeds max entries |
| `property_cache_control.rs` | no-cache directive respected |
| `property_hierarchy.rs` | Coordinator creates with/without root cache |
| `property_circular.rs` | Circular dependencies detected |
| `property_policy.rs` | Domain locking, allowlists, subdomain matching |
| `property_rate_limit.rs` | Token bucket enforces limits |
| `property_error_responses.rs` | Error types produce correct messages |
| `property_timeout.rs` | Client respects timeout configuration |
| `property_https.rs` | URLs use HTTPS |
| `property_client_routing.rs` | Client SDK routing logic |
| `property_dns_resilience.rs` | DNS failures don't crash the client |

### Tier 2: HTTP Mock Tests (1 file, 9 tests)

**Purpose**: Verify the HTTP client handles real-world server responses correctly.
**When**: Every PR.

| Test | What It Catches |
|------|----------------|
| `fetch_valid_ax_record_from_mock` | Happy path: valid AX JSON returned |
| `fetch_returns_error_on_404` | Missing domain handled gracefully |
| `fetch_returns_error_on_502` | Upstream failure propagated |
| `fetch_returns_error_on_invalid_json` | Garbled response doesn't panic |
| `fetch_returns_error_on_valid_json_but_not_ax` | Non-AX JSON rejected |
| `fetch_handles_timeout` | Slow server triggers timeout error |
| `fetch_handles_empty_body` | Empty response doesn't panic |
| `fetch_ax_record_with_optional_fields` | Capabilities, security, limits parsed from HTTP |
| `fetch_ax_record_with_unknown_extensions` | Unknown fields preserved, not rejected |

### Tier 2: Server Route Tests (1 file, 7 tests)

**Purpose**: Exercise every HTTP endpoint with real requests through the axum router.
**When**: Every PR.

| Test | What It Catches |
|------|----------------|
| `health_returns_200_with_json` | /health works and returns JSON |
| `health_response_has_correct_shape` | Response includes "status" field |
| `metrics_returns_200` | /metrics works and includes cache_hits |
| `discover_returns_502_on_upstream_failure` | Missing domain returns 502/404, not 500 |
| `discover_returns_403_when_policy_blocks` | Domain locking returns 403 |
| `discover_allows_home_domain_when_locked` | Home domain not blocked |
| `unknown_route_returns_404` | Undefined paths return 404 |

### Tier 2: CLI Binary Tests (1 file, 14 tests)

**Purpose**: Invoke the actual compiled binary and verify user-facing behavior.
**When**: Every PR.

| Test | What It Catches |
|------|----------------|
| `binary_runs_with_help` | Binary starts, clap parses --help |
| `binary_shows_version` | --version works |
| `generate/validate/discover/serve/cache_invalidate_subcommand_exists` | All 5 subcommands register correctly |
| `generate_requires_config_flag` | Missing required args produce error |
| `generate_and_validate_roundtrip` | Full generate -> validate flow via binary |
| `validate_fails_on_invalid_file` | Invalid AX record exits with failure |
| `validate_warns_on_unknown_auth` | Unknown auth method doesn't fail |
| `validate_exits_3_on_validation_error` | Exit code 3 for validation failures |
| `validate_nonexistent_file_fails` | Missing file handled gracefully |
| `unknown_subcommand_fails` | Typos produce error, not panic |

### Tier 2: Malformed Input Tests (1 file, 20 tests)

**Purpose**: Ensure the parser never panics on adversarial input.
**When**: Every PR.

| Test Category | Tests | What It Catches |
|---------------|-------|----------------|
| Wrong JSON types | 5 | null, number, string, array, wrong field types |
| Truncated/broken | 2 | Truncated JSON, deeply nested braces |
| Large input | 1 | 1MB string value (memory safety) |
| Unicode | 1 | Non-ASCII agent names and URLs |
| Extra fields | 1 | Unknown fields don't break parsing |
| Backward compat | 3 | Flat docs, legacy wrapper, multi-record wrapper |
| Minimal valid | 1 | Spec-minimum record (no provider, no auth) |
| Fully populated | 1 | Every optional field present |
| Null fields | 1 | Explicit null for optional fields |
| Empty endpoints | 1 | Parses but fails validation |

### Tier 3: Integration Tests (6 files, ~12 tests)

**Purpose**: Cross-module flows (generate -> validate, OAuth -> AX, cache hierarchy).
**When**: Every PR.

### Tier 3: Stress / Extended (via Makefile)

**Purpose**: Deeper property exploration and performance regression.
**When**: Nightly or pre-release.

```bash
make test-stress     # PROPTEST_CASES=10000
make bench           # Criterion benchmarks
```

## Running Tests

```bash
make test            # All 145 tests (30 seconds)
make test-http       # HTTP mock tests only
make test-server     # Server route tests only
make test-cli        # CLI binary tests only
make test-malformed  # Malformed input tests only
make test-property   # Extended property tests (1000 iterations)
make ci              # Full CI pipeline locally (format + lint + all tests)
```

## CI Pipeline

```
PR opened:
  lint          -> fmt --check, clippy (all features + no default)
  test(stable)  -> cargo test --all-features, --no-default-features
  test(beta)    -> same on beta toolchain
  coverage      -> llvm-cov with threshold check
  security      -> cargo-audit

ci-pass gate: lint + test must succeed; coverage + security advisory
```

## What Each Layer Catches

| Failure Type | Caught By |
|-------------|-----------|
| AX spec non-compliance | Property tests + malformed input |
| Serialization bugs | Round-trip property tests |
| HTTP parsing bugs | Wiremock mock tests |
| Server routing bugs | Axum route tests |
| CLI breakage (bad args, exit codes) | Binary tests |
| Cache correctness (LRU, TTL, eviction) | Property tests |
| Policy enforcement bugs | Property + route tests |
| Rate limiting bugs | Property tests |
| Adversarial input (crashes, panics) | Malformed input tests |
| Performance regression | Criterion benchmarks (Tier 3) |
| Dependency vulnerabilities | cargo-audit in CI |
