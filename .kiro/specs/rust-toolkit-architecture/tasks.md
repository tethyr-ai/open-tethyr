# Implementation Plan: Open-Tethyr Rust Toolkit Architecture

## Overview

This implementation plan breaks down the open-tethyr Rust toolkit into discrete, incremental tasks that build upon each other. The approach prioritizes core AX protocol compliance, then adds caching functionality, and finally implements the CLI and server components. Each task includes specific requirements references and testing sub-tasks to ensure correctness.

## Tasks

- [x] 1. Set up Cargo workspace and project structure
  - Create root Cargo.toml with workspace configuration
  - Set up crates/open-tethyr and crates/cli directories
  - Configure shared dependencies and feature flags
  - Add basic CI/CD workflow files
  - _Requirements: 1.1, 1.2, 1.3, 17.3_

- [x] 2. Implement core AX protocol data models
  - [x] 2.1 Create AX record data structures with proper serde annotations
    - Implement AgentExchangeRecord with record_type="AX" and version="1.0"
    - Implement Agent struct (name, description, provider only)
    - Implement Endpoint struct with auth as Vec<String>
    - Implement Protocol enum (Rest, GraphQL, MCP, A2A, Custom)
    - Implement AgentExchangeDocument container for multiple records
    - _Requirements: 5.1, 5.2, 9.1, 18.1_

  - [x] 2.2 Write property test for AX record serialization
    - **Property 8: Serialization Round-Trip Consistency**
    - **Validates: Requirements 5.2, 5.3**

  - [x] 2.3 Write property test for AX version validation
    - **Property 24: AX Version Validation and Handling**
    - **Validates: Requirements 18.1, 18.3**

  - [x] 2.4 Implement FileWriter utility
    - Write well-known file structure to disk
    - Create /.well-known/agent-exchange.json paths
    - Support atomic file operations
    - _Requirements: 2.5, 9.2_

- [x] 3. Implement AX protocol validation and generation
  - [x] 3.1 Create AxValidator with comprehensive validation rules
    - Validate record_type is "AX"
    - Validate version is "1.0"
    - Validate required fields and format constraints
    - Validate auth methods are from supported set (OIDC, OAuth2, mTLS, JWT, API_KEY)
    - _Requirements: 2.2, 5.5, 9.3, 18.1_

  - [x] 3.2 Create AxGenerator for record generation from configuration
    - Generate AX-compliant records from YAML config
    - Support configuration inheritance and merging
    - Generate well-known file structure
    - _Requirements: 2.1, 9.1, 9.2_

  - [x] 3.3 Write property test for AX record generation
    - **Property 1: AX Record Generation Correctness**
    - **Validates: Requirements 2.1, 9.1, 9.3**

  - [x] 3.4 Write property test for AX record validation
    - **Property 2: AX Record Validation Correctness**
    - **Validates: Requirements 2.2, 5.5**

- [x] 4. Implement configuration system with inheritance
  - [x] 4.1 Create configuration data models and parsing
    - Implement AgentConfig with defaults and agents sections
    - Support YAML configuration file parsing
    - Implement configuration validation:
      - Domain names match DNS format (regex)
      - Port numbers are 1-65535
      - URLs are HTTPS format
      - TTL values are positive integers
    - _Requirements: 7.1, 13.1, 13.4_

  - [x] 4.2 Implement configuration inheritance and merging
    - Create ConfigMerger for type-safe inheritance
    - Support nested inheritance for complex objects
    - Preserve agent-specific overrides of defaults
    - _Requirements: 7.2, 7.3, 7.4, 7.5_

  - [x] 4.3 Write property test for configuration inheritance
    - **Property 10: Configuration Inheritance Correctness**
    - **Validates: Requirements 7.2, 7.3, 7.4, 7.5**

  - [x] 4.4 Write property test for configuration validation
    - **Property 20: Configuration Validation Correctness**
    - **Validates: Requirements 13.4**

- [x] 5. Implement authentication provider templates
  - [x] 5.1 Create OAuth provider trait and registry
    - Define OAuthProvider trait with endpoint generation
    - Implement provider registry for multiple providers
    - Create OAuthEndpoints data structure
    - _Requirements: 4.5, 6.7_

  - [x] 5.2 Implement OAuth provider templates (MVP)
    - Implement OktaProvider with standard endpoint patterns
    - Implement Auth0Provider with tenant-specific URLs
    - Implement GenericOAuth2Provider for RFC 8414 compliant providers
    - _Requirements: 6.1, 6.2, 6.7_

  - [x] 5.3 Write property test for OAuth provider templates
    - **Property 9: OAuth Provider Template Correctness**
    - **Validates: Requirements 6.1, 6.2, 6.7**

- [x] 6. Implement DNS discovery and HTTP client
  - [x] 6.1 Create DNS discovery module
    - Implement DnsDiscovery with TXT record lookup
    - Parse cache endpoint URLs from DNS TXT records
    - Support _ax-cache.<domain> discovery pattern
    - Handle DNS lookup failures gracefully
    - _Requirements: 4.2, 11.1, 11.4, 11.6_

  - [x] 6.2 Create HTTP client for AX endpoint fetching
    - Implement AxHttpClient with configurable timeout (30s default)
    - Build correct AX URLs (_agent.<domain>/.well-known/agent-exchange.json)
    - Validate HTTPS certificates and well-known paths
    - Support both cache and direct discovery modes
    - _Requirements: 4.3, 9.4, 9.5, 9.6, 15.4, 15.5_

  - [x] 6.3 Write property test for DNS TXT record parsing
    - **Property 18: DNS TXT Record Parsing Correctness**
    - **Validates: Requirements 11.4, 11.6**

  - [x] 6.4 Write property test for AX URL construction
    - **Property 13: AX Subdomain URL Construction**
    - **Validates: Requirements 9.4**

- [x] 7. Implement test utilities and mocks
  - Create MockDnsResolver for testing
  - Create MockHttpServer with wiremock
  - Add test fixtures for AX records
  - _Requirements: 16.3, 16.4_

- [x] 8. Checkpoint - Core functionality validation
  - Ensure all tests pass for AX protocol, configuration, and discovery
  - Verify authentication provider templates generate correct endpoints
  - Test DNS discovery and HTTP client functionality
  - Ask the user if questions arise

- [x] 9. Implement cache storage and coordination
  - [x] 9.1 Create in-memory cache implementation
    - Implement MemoryCache with HashMap storage
    - Implement LRU eviction with configurable max_entries limit
    - Support TTL expiration and cache statistics
    - Implement CacheStats with atomic counters (hits, misses, evictions, memory usage)
    - Parse and respect Cache-Control headers from AX endpoints (max-age, no-cache directives)
    - Implement cache invalidation and cleanup
    - _Requirements: 10.1, 10.2, 10.3, 10.6_

  - [x] 9.2 Create cache coordination for hierarchical caching
    - Implement CacheCoordinator with fallback chain
    - Support DNS-based root cache discovery
    - Handle upstream cache failures gracefully
    - Validate cache configuration for circular dependencies during initialization
    - Build dependency graph and use depth-first search to detect cycles
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6_

  - [x] 9.3 Write property test for cache behavior
    - **Property 4: Cache-First Discovery Behavior**
    - **Validates: Requirements 3.2**

  - [x] 9.4 Write property test for cache miss fallback
    - **Property 5: Cache Miss Fallback Behavior**
    - **Validates: Requirements 3.3, 9.6**

  - [x] 9.5 Write property test for TTL expiration
    - **Property 7: TTL Expiration Correctness**
    - **Validates: Requirements 3.5, 10.6**

  - [x] 9.6 Write property test for LRU eviction
    - **Property 15: LRU Cache Eviction Correctness**
    - **Validates: Requirements 10.3**

  - [x] 9.7 Write property test for hierarchical caching
    - **Property 11: Hierarchical Cache Fallback Chain**
    - **Validates: Requirements 8.2, 8.3, 8.4, 8.6**

  - [x] 9.8 Write property test for circular dependency prevention
    - **Property 12: Circular Dependency Prevention**
    - **Validates: Requirements 8.5**

  - [x] 9.9 Write property test for Cache-Control header compliance
    - **Property 16: Cache-Control Header Compliance**
    - **Validates: Requirements 10.8**

  - [x] 9.10 Write property test for cache size limit enforcement
    - **Property 26: Cache Size Limit Enforcement**
    - **Validates: Requirements 10.3, 10.5**

- [x] 10. Implement policy enforcement and rate limiting
  - [x] 10.1 Create policy enforcement engine
    - Implement domain locking policies
    - Support allowlist configuration for external domains
    - Return appropriate HTTP error responses for violations
    - Log all discovery requests for audit purposes
    - _Requirements: 12.1, 12.2, 12.3, 12.5_

  - [x] 10.2 Create rate limiting with token bucket algorithm
    - Implement TokenBucket with refill mechanism
    - Add per-client IP rate limiting
    - Support configurable rate limits
    - Return HTTP 429 for exceeded limits
    - _Requirements: 15.1, 15.3_

  - [x] 10.3 Write property test for domain locking
    - **Property 6: Domain Locking Policy Enforcement**
    - **Validates: Requirements 3.4, 12.1, 12.2, 12.5**

  - [x] 10.4 Write property test for rate limiting
    - **Property 21: Rate Limiting Enforcement**
    - **Validates: Requirements 15.1, 15.3**

- [ ] 11. Implement cache server (feature = "server")
  - [ ] 11.1 Create cache server with HTTP API
    - Implement CacheServer with axum framework
    - Add discovery endpoint with policy enforcement
    - Include health check endpoint
    - Implement /metrics endpoint with Prometheus text format (cache hits, misses, active connections)
    - Implement SimpleHistogram for request duration tracking with predefined bucket boundaries
    - Add structured logging with correlation IDs
    - _Requirements: 3.1, 14.1, 14.4_

  - [ ] 11.2 Add middleware and error handling
    - Implement rate limiting middleware
    - Add structured JSON error responses with error message, timestamp, correlation ID
    - Include request timeout handling
    - Add correlation ID to response headers
    - _Requirements: 15.5, 15.6_

  - [ ] 11.3 Write property test for HTTP error responses
    - **Property 23: HTTP Error Response Mapping**
    - **Validates: Requirements 15.6**

  - [ ] 11.4 Write property test for request timeout handling
    - **Property 22: Request Timeout Handling**
    - **Validates: Requirements 15.5**

  - [ ] 11.5 Write property test for HTTPS certificate validation
    - **Property 14: HTTPS Certificate Validation**
    - **Validates: Requirements 9.5, 15.4**

- [ ] 12. Implement CLI commands
  - [ ] 12.1 Create CLI structure with clap
    - Implement main CLI with all subcommands in single binary
    - Add generate command for AX record creation
    - Add validate command for AX record validation
    - Add discover command for testing discovery
    - Add serve command for cache server
    - _Requirements: 2.1, 2.2, 2.3, 2.5_

  - [ ] 12.2 Implement generate command
    - Load YAML configuration with inheritance
    - Generate AX records with validation
    - Create well-known file structure output
    - Support validation flag for generated records
    - _Requirements: 2.1, 2.5_

  - [ ] 12.3 Implement discover command
    - Test agent discovery with/without cache
    - Support --cache flag for explicit cache URL
    - Display discovered agents
    - _Requirements: 2.3_

  - [ ] 12.4 Implement validate command
    - Validate existing AX record files
    - Report validation errors clearly
    - _Requirements: 2.2_

  - [ ] 12.5 Implement serve command
    - Start cache server with configuration
    - Support --domain, --port, --config flags
    - _Requirements: 3.1_

  - [ ] 12.6 Write property test for well-known file structure
    - **Property 3: Well-Known File Structure Generation**
    - **Validates: Requirements 2.5, 9.2**

- [ ] 13. Implement client SDK (feature = "client")
  - [ ] 13.1 Create OpenTethyr client SDK
    - Implement client initialization with domain
    - Add automatic cache discovery via DNS
    - Support both cached and direct discovery modes
    - Provide clean API for agent discovery
    - _Requirements: 4.6, 11.2, 11.3_

  - [ ] 13.2 Write property test for DNS cache discovery routing
    - **Property 17: DNS Cache Discovery Routing**
    - **Validates: Requirements 11.2, 11.3**

  - [ ] 13.3 Write property test for DNS error resilience
    - **Property 19: DNS Discovery Error Resilience**
    - **Validates: Requirements 11.5**

  - [ ] 13.4 Write property test for auth method validation
    - **Property 25: Auth Method Validation**
    - **Validates: Requirements 9.3, 15.4**

- [ ] 14. Add comprehensive error handling and logging
  - [ ] 14.1 Implement structured error types
    - Create error types with thiserror for all modules
    - Add HTTP error response mapping
    - Include detailed error context and correlation
    - _Requirements: 14.5, 15.6_

  - [ ] 14.2 Add structured logging throughout
    - Configure tracing with structured fields
    - Add HTTP request/response logging
    - Include correlation ID propagation
    - Support configurable log levels
    - _Requirements: 14.1, 14.2, 14.3_

- [ ] 15. Checkpoint - Integration testing
  - Test complete discovery flow: CLI generate → validate → serve → discover
  - Test hierarchical cache with root fallback
  - Test policy rejection with domain locking
  - Test rate limiting with concurrent requests
  - Test DNS failure graceful degradation
  - Test OAuth provider template integration (generate config with OAuth → verify endpoints in AX record)
  - Ensure all property tests pass
  - Ask the user if questions arise

- [ ] 16. Add build and distribution support
  - [ ] 16.1 Create Docker configuration
    - Add Dockerfile for containerized deployment
    - Configure GitHub Actions for Docker image builds
    - Set up publishing to GitHub Container Registry
    - _Requirements: 17.2, 17.4_

  - [ ] 16.2 Add cross-compilation support
    - Configure cross-compilation for Linux, macOS, Windows
    - Set up static linking with musl for Linux
    - Create GitHub Actions for automated releases
    - _Requirements: 17.1, 17.5, 17.6_

  - [ ] 16.3 Add performance benchmarks
    - Benchmark cache operations (get, put, evict)
    - Benchmark AX record parsing and validation
    - Benchmark configuration inheritance
    - _Requirements: (implied by Criterion in design)_

- [ ] 17. Final validation and documentation
  - [ ] 17.1 Run comprehensive test suite
    - Execute all unit tests and property tests
    - Run integration tests across components
    - Verify performance benchmarks meet requirements
    - _Requirements: 16.1, 16.2, 16.5_

  - [ ] 17.2 Validate AX protocol compliance
    - Generate AX records and manually verify against AX 1.0 specification
    - Test discovery with mock AX-compliant server
    - Verify JSON schema compliance with AX RFC examples
    - _Requirements: 9.1, 9.3, 18.1_

- [ ] 18. Final checkpoint - Complete system validation
  - Ensure all tests pass and system meets requirements
  - Verify Docker images build and run correctly
  - Test CLI installation and usage workflows
  - Ask the user if questions arise

## Notes

- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation and user feedback
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases
- The implementation follows strict AX protocol compliance throughout