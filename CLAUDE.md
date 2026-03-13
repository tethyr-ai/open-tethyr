# feat-tether-foundation Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-03-12

## Active Technologies
- In-memory (HashMap + LRU) for MVP; no persistent storage (001-rust-toolkit-architecture)

- Rust (stable toolchain, edition 2021) + serde/serde_json/serde_yaml (serialization), reqwest with rustls-tls (HTTP), trust-dns-resolver (DNS), axum/tower/tower-http (server, opt-in), clap (CLI), lru (cache eviction), tracing/tracing-subscriber (logging), thiserror/anyhow (errors), uuid (correlation IDs), proptest (property testing), criterion (benchmarks), wiremock/mockall (test mocks) (001-rust-toolkit-architecture)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust (stable toolchain, edition 2021): Follow standard conventions

## Recent Changes
- 001-rust-toolkit-architecture: Added Rust (stable toolchain, edition 2021) + serde/serde_json/serde_yaml (serialization), reqwest with rustls-tls (HTTP), trust-dns-resolver (DNS), axum/tower/tower-http (server, opt-in), clap (CLI), lru (cache eviction), tracing/tracing-subscriber (logging), thiserror/anyhow (errors), uuid (correlation IDs), proptest (property testing), criterion (benchmarks), wiremock/mockall (test mocks)

- 001-rust-toolkit-architecture: Added Rust (stable toolchain, edition 2021) + serde/serde_json/serde_yaml (serialization), reqwest with rustls-tls (HTTP), trust-dns-resolver (DNS), axum/tower/tower-http (server, opt-in), clap (CLI), lru (cache eviction), tracing/tracing-subscriber (logging), thiserror/anyhow (errors), uuid (correlation IDs), proptest (property testing), criterion (benchmarks), wiremock/mockall (test mocks)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
