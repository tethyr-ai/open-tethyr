# Tasks: AX Draft Spec Alignment

**Input**: Design documents from `/specs/002-ax-spec-alignment/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md

**Tests**: Existing property tests and integration tests will be updated to match the new spec-compliant structure. New tests added for backward compatibility parsing.

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Foundational Type Changes

**Purpose**: Update core types to match the official AX schema. All subsequent tasks depend on these.

- [x] T001 Update Security struct to flat structure with issuer, jwks_url, signature, metadata_signature fields (remove nested oauth wrapper) with serde flatten for additional properties in crates/open-tethyr/src/ax/types.rs
- [x] T002 [P] Update Capabilities struct with typed fields: intents (Vec<String>), async_exec (bool), supports_callbacks (bool), callback_modes (Vec<String>) with serde flatten for additional properties in crates/open-tethyr/src/ax/types.rs
- [x] T003 [P] Update Schema struct with typed fields: graphql_schema_url, mcp_manifest_url, rest_openapi_url (Option<String>), introspection (Option<bool>) with serde flatten for additional properties in crates/open-tethyr/src/ax/types.rs
- [x] T004 [P] Update Limits struct with typed fields: max_concurrent_tasks, max_task_ttl_seconds, rate_limit_per_minute (Option<f64>) with serde flatten for additional properties in crates/open-tethyr/src/ax/types.rs
- [x] T005 Update AgentExchangeRecord: make agent.provider optional (Option<String>), keep endpoints[].auth as Vec<String> but don't require non-empty in crates/open-tethyr/src/ax/record.rs
- [x] T006 Add AWS_IAM to APPROVED_AUTH_METHODS list and change validate_auth_methods to warn on unknown methods instead of error in crates/open-tethyr/src/error.rs
- [x] T007 Update ax/mod.rs re-exports to remove OAuthEndpoints and add new typed field exports in crates/open-tethyr/src/ax/mod.rs
- [x] T008 Verify types compile with `cargo check --workspace --all-features`

---

## Phase 2: US1 - Correct Well-Known Path and Discovery URL (Priority: P1)

**Goal**: Fix the well-known path and discovery URL to match the AX spec.

- [x] T009 [US1] Update AxHttpClient::build_ax_url to return `https://<domain>/.well-known/agent-exchange` (remove _agent. prefix and .json extension) in crates/open-tethyr/src/http/client.rs
- [x] T010 [US1] Update AxHttpClient::validate_well_known_path to check for `/.well-known/agent-exchange` (no .json) in crates/open-tethyr/src/http/client.rs
- [x] T011 [US1] Update FileWriter::write_structure to output file named `agent-exchange` (not `agent-exchange.json`) in crates/open-tethyr/src/http/file_writer.rs
- [x] T012 [US1] Update property_url.rs tests: URL format assertions to `https://<domain>/.well-known/agent-exchange` in crates/open-tethyr/tests/property_url.rs
- [x] T013 [US1] Update property_wellknown.rs test: output path assertion to `.well-known/agent-exchange` in crates/open-tethyr/tests/property_wellknown.rs

---

## Phase 3: US2 - Correct AX Document Structure (Priority: P1)

**Goal**: Generate flat AX documents, parse both flat and legacy wrapper.

- [x] T014 [US2] Update AxGenerator::generate_record to return a single AgentExchangeRecord (flat document) instead of AgentExchangeDocument wrapper in crates/open-tethyr/src/ax/generator.rs
- [x] T015 [US2] Add parsing function that detects and handles both flat AX documents and legacy {records:[...]} wrapper format in crates/open-tethyr/src/ax/record.rs
- [x] T016 [US2] Update GenerateCommand to write flat AX JSON (single record per file) in crates/cli/src/commands/generate.rs
- [x] T017 [US2] Update ValidateCommand to handle both flat and wrapper formats in crates/cli/src/commands/validate.rs
- [x] T018 [US2] Update DiscoverCommand to handle both flat and wrapper response formats in crates/cli/src/commands/discover.rs
- [x] T019 [US2] Update OpenTethyr client discover() return type to handle flat AX document in crates/open-tethyr/src/client.rs
- [x] T020 [US2] Update CacheCoordinator to store and return flat AX JSON in crates/open-tethyr/src/cache/coordinator.rs
- [x] T021 [US2] Update server handlers to return flat AX JSON from discover endpoint in crates/open-tethyr/src/server/handlers.rs
- [x] T022 [US2] Update lib.rs re-exports and doc example for flat document model in crates/open-tethyr/src/lib.rs

---

## Phase 4: US3 - Correct Field Requirements and Auth Methods (Priority: P1)

**Goal**: Relax required fields and fix auth method handling per spec.

- [x] T023 [US3] Update AxValidator::validate_record to not require agent.provider (accept empty/missing) in crates/open-tethyr/src/ax/validator.rs
- [x] T024 [US3] Update AxValidator::validate_endpoints to not require endpoints[].auth (accept empty/missing) in crates/open-tethyr/src/ax/validator.rs
- [x] T025 [US3] Update AxValidator::validate_auth_methods to return warnings for unknown methods instead of errors in crates/open-tethyr/src/ax/validator.rs
- [x] T026 [US3] Update property_validation.rs tests for relaxed field requirements in crates/open-tethyr/tests/property_validation.rs
- [x] T027 [US3] Update property_auth_validation.rs to test AWS_IAM accepted and unknown methods produce warnings in crates/open-tethyr/tests/property_auth_validation.rs

---

## Phase 5: US4 - Typed Capabilities, Schema, Limits, Security (Priority: P2)

**Goal**: Verify typed fields serialize/deserialize correctly.

- [x] T028 [US4] Update AxGenerator to populate security.issuer and security.jwks_url from OAuth provider templates (instead of nested oauth block) in crates/open-tethyr/src/ax/generator.rs
- [x] T029 [US4] Update OAuthEndpoints type or remove it; OAuth providers now return (issuer, jwks_url) pair for flat security in crates/open-tethyr/src/auth/provider.rs and crates/open-tethyr/src/auth/okta.rs, auth0.rs, generic.rs
- [x] T030 [US4] Write property test for capabilities round-trip: intents, async, supports_callbacks, callback_modes in crates/open-tethyr/tests/property_serialization.rs
- [x] T031 [US4] Write property test for schema round-trip: graphql_schema_url, mcp_manifest_url, rest_openapi_url, introspection in crates/open-tethyr/tests/property_serialization.rs
- [x] T032 [US4] Write property test for limits round-trip: max_concurrent_tasks, max_task_ttl_seconds, rate_limit_per_minute in crates/open-tethyr/tests/property_serialization.rs
- [x] T033 [US4] Write property test for flat security round-trip: issuer, jwks_url, signature, metadata_signature in crates/open-tethyr/tests/property_serialization.rs

---

## Phase 6: Polish & Cross-Cutting

**Purpose**: Update all remaining tests, integration tests, docs, and verify everything passes.

- [x] T034 Update property_generation.rs tests for flat document output and new security structure in crates/open-tethyr/tests/property_generation.rs
- [x] T035 Update property_oauth.rs tests for flat security output (issuer + jwks_url, not nested oauth) in crates/open-tethyr/tests/property_oauth.rs
- [x] T036 [P] Update integration_e2e.rs test for flat document and new paths in crates/open-tethyr/tests/integration_e2e.rs
- [x] T037 [P] Update integration_oauth.rs test for flat security structure in crates/open-tethyr/tests/integration_oauth.rs
- [x] T038 [P] Update test_generate_validate.rs CLI test for new output path and flat structure in crates/cli/tests/test_generate_validate.rs
- [x] T039 [P] Update test_validate_detailed.rs for relaxed field requirements in crates/cli/tests/test_validate_detailed.rs
- [x] T040 Update docs/features.md and docs/setup-guide.md for new paths, structure, and field names in docs/
- [x] T041 Run `cargo clippy --workspace --all-features -- -D warnings` and fix all warnings
- [x] T042 Run `cargo fmt --all` and verify formatting
- [x] T043 Run `cargo test --workspace --all-features` and verify all tests pass (target: 94+ tests, zero failures)

---

## Dependencies

- **Phase 1** (T001-T008): No dependencies, must complete first
- **Phase 2** (T009-T013): Depends on Phase 1 (types must compile)
- **Phase 3** (T014-T022): Depends on Phase 1 (new types used in generator/parser)
- **Phase 4** (T023-T027): Depends on Phase 1 (auth method changes in error.rs)
- **Phase 5** (T028-T033): Depends on Phase 1 (Security struct restructured)
- **Phase 6** (T034-T043): Depends on all previous phases

**Phases 2-5 can run in parallel** after Phase 1 completes (different files).

## Implementation Strategy

### MVP (Phases 1-2 only)
Fix the well-known path and discovery URL. This alone makes discovery interoperable.

### Core (Add Phases 3-4)
Fix document structure and field requirements. Full AX spec compliance for basic records.

### Complete (Add Phase 5-6)
Typed sub-objects, updated tests, updated docs. Full alignment with official schema.
