# Implementation Plan: AX Draft Spec Alignment

**Branch**: `002-ax-spec-alignment` | **Date**: 2026-03-14 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-ax-spec-alignment/spec.md`

## Summary

Align the open-tethyr implementation with the official AX draft specification at sempfa/agent-discovery-exchange. This is a refactoring feature that corrects 9 mismatches: well-known path (remove `.json` extension), discovery URL (remove `_agent.` subdomain), document structure (flat object, not `records` wrapper), field requirements (`provider`/`auth` now optional), auth methods (add `AWS_IAM`, warn on unknown), security block (flat, not nested `oauth`), and typed capabilities/schema/limits fields. No new crates or dependencies required.

## Technical Context

**Language/Version**: Rust (stable toolchain, edition 2021) -- same as feature 001
**Primary Dependencies**: No changes -- same crate set as feature 001
**Storage**: N/A -- in-memory only
**Testing**: cargo test + proptest -- existing test infrastructure
**Target Platform**: Same as feature 001
**Project Type**: Library + CLI (refactoring, no new crates)
**Performance Goals**: No regression from current performance
**Constraints**: Backward compatibility with legacy `{records:[...]}` format
**Scale/Scope**: ~10 source files modified, ~15 test files updated

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*
*Validated against: constitution v1.0.0 (ratified 2026-03-13)*

| Principle | Pre-Phase 0 | Post-Phase 1 | Evidence |
|-----------|-------------|--------------|----------|
| I. Library-First Architecture | PASS | PASS | No structural changes -- same two-crate workspace |
| II. AX Protocol Compliance (NON-NEGOTIABLE) | PASS | PASS | This feature IMPROVES compliance by aligning with official spec |
| III. Property-Based Testing | PASS | PASS | Existing property tests updated + new round-trip tests for typed fields |
| IV. Simplicity/YAGNI | PASS | PASS | Only adding fields the spec defines -- no speculative additions |
| V. Structured Observability | PASS | PASS | No changes to logging/metrics |
| VI. Static Distribution | PASS | PASS | No changes to build/distribution |

## Project Structure

### Documentation (this feature)

```text
specs/002-ax-spec-alignment/
├── plan.md              # This file
├── research.md          # Phase 0: spec analysis
├── data-model.md        # Phase 1: updated entity definitions
└── tasks.md             # Phase 2: task breakdown (via /speckit.tasks)
```

### Source Code (files modified)

```text
crates/open-tethyr/src/
├── ax/
│   ├── types.rs         # Capabilities, Schema, Limits, Security restructured
│   ├── record.rs        # AgentExchangeRecord flat, AgentExchangeDocument compat
│   ├── validator.rs     # Relaxed required fields, auth warning mode
│   └── generator.rs     # Output flat document, populate security.issuer/jwks_url
├── http/
│   ├── client.rs        # URL: domain/.well-known/agent-exchange (no _agent., no .json)
│   └── file_writer.rs   # Output filename: agent-exchange (no .json)
├── error.rs             # Add AWS_IAM to approved list, add warning-level validation
└── client.rs            # Discovery URL updated

crates/cli/src/commands/
├── generate.rs          # Multi-agent -> single doc per agent
└── validate.rs          # Handle flat + legacy wrapper formats

crates/open-tethyr/tests/
├── property_*.rs        # Updated for new structure/field names
└── integration_*.rs     # Updated for new paths/structure
```

**Structure Decision**: No structural changes. This is a refactoring of existing files to align field names, paths, and document structure with the official AX specification.

## Complexity Tracking

No constitution violations to justify. This feature strictly improves AX protocol compliance.
