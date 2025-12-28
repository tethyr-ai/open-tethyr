# Requirements Document

## Introduction

The open-tethyr Rust toolkit MVP is a distributed caching system for agent discovery implementing the Agent Discovery Exchange (AX) protocol. The system provides lazy caching, infrastructure-level control over agent discovery, and DNS-based cache coordination while maintaining complete organizational control over discovery policies and security boundaries. This MVP focuses on core functionality with CLI tools, cache server, and core library components.

## Glossary

- **AX_Protocol**: Agent Discovery Exchange protocol for standardized agent discovery using HTTPS and well-known URIs
- **Agent_Record**: JSON document containing agent metadata following AX 1.0 specification
- **Cache_Server**: Distributed caching infrastructure that stores and serves agent discovery records
- **CLI_Tool**: Command-line interface for generating AX records, validation, and testing
- **Core_Library**: Rust library providing shared functionality across all components
- **Discovery_Cache**: Client-domain scoped cache for performance and policy enforcement
- **DNS_Coordination**: DNS-based cache discovery using TXT records at _ax-cache.<domain>
- **Domain_Scoping**: Security model where clients only use caches from their own domain
- **Hierarchical_Caching**: Multi-level cache architecture with root and regional caches
- **LRU_Eviction**: Least Recently Used cache eviction policy
- **OAuth_Provider**: Authentication provider with standardized endpoint patterns
- **TTL**: Time To Live for cached records
- **Well_Known_URI**: RFC 8615 standardized URI path for service metadata

## Requirements

### Requirement 1: Cargo Workspace Structure

**User Story:** As a developer, I want a well-organized Cargo workspace, so that I can efficiently develop and maintain multiple related crates.

#### Acceptance Criteria

1. THE Workspace SHALL contain separate crates for CLI, server, and core library
2. THE Workspace SHALL use a root Cargo.toml with workspace configuration
3. WHEN building the workspace, THE System SHALL compile all crates with shared dependencies
4. THE Workspace SHALL support independent versioning of individual crates
5. THE Workspace SHALL include integration tests that span multiple crates

### Requirement 2: CLI Tool Implementation

**User Story:** As a system administrator, I want a command-line tool, so that I can generate AX records, validate configurations, and test discovery.

#### Acceptance Criteria

1. WHEN generating AX records, THE CLI_Tool SHALL create valid JSON documents from YAML configuration
2. WHEN validating AX records, THE CLI_Tool SHALL verify compliance with AX 1.0 specification
3. WHEN testing discovery, THE CLI_Tool SHALL support both cached and direct discovery modes
4. THE CLI_Tool SHALL provide clear error messages for invalid configurations
5. WHEN outputting files, THE CLI_Tool SHALL create the standard /.well-known/agent-exchange.json structure

### Requirement 3: Cache Server Architecture

**User Story:** As an infrastructure operator, I want a high-performance cache server, so that I can provide controlled agent discovery with policy enforcement.

#### Acceptance Criteria

1. WHEN starting the server, THE Cache_Server SHALL bind to the specified port and domain
2. WHEN receiving discovery requests, THE Cache_Server SHALL check local storage first
3. IF a cache miss occurs, THEN THE Cache_Server SHALL fetch from the target domain's AX endpoint
4. THE Cache_Server SHALL enforce domain locking policies when configured
5. WHEN storing cached records, THE Cache_Server SHALL respect TTL configurations
6. THE Cache_Server SHALL support hierarchical caching with DNS-based root cache discovery

### Requirement 4: Core Library Design

**User Story:** As a developer, I want a shared core library, so that I can reuse common functionality across CLI, server, and client components.

#### Acceptance Criteria

1. THE Core_Library SHALL provide AX record parsing and validation functions
2. THE Core_Library SHALL implement DNS-based cache discovery logic
3. THE Core_Library SHALL provide HTTP client functionality for AX endpoint fetching
4. THE Core_Library SHALL include data models for all AX 1.0 specification types
5. THE Core_Library SHALL support OAuth configuration generation for multiple providers
6. THE Core_Library SHALL provide a client API for agent discovery from Rust applications

### Requirement 5: Data Models and Schema

**User Story:** As a developer, I want strongly-typed data models, so that I can ensure AX protocol compliance and prevent runtime errors.

#### Acceptance Criteria

1. THE System SHALL define Rust structs for all AX record components
2. WHEN deserializing JSON, THE System SHALL validate against AX 1.0 schema requirements
3. THE System SHALL support serde serialization and deserialization for all data types
4. THE System SHALL provide ergonomic API for configuration objects
5. WHEN validating agent records, THE System SHALL enforce required fields and format constraints

### Requirement 6: OAuth Provider Templates

**User Story:** As an administrator, I want OAuth provider templates, so that I can easily configure authentication for common providers.

#### Acceptance Criteria

1. THE System SHALL support Okta OAuth configuration with standard endpoint patterns
2. THE System SHALL support Auth0 OAuth configuration with tenant-specific URLs
3. THE System SHALL support Azure AD OAuth configuration with tenant ID handling
4. THE System SHALL support AWS Cognito OAuth configuration with region and user pool
5. THE System SHALL support Google OAuth configuration with standard endpoints
6. THE System SHALL support Keycloak OAuth configuration with realm-specific URLs
7. WHEN generating OAuth endpoints, THE System SHALL follow provider-specific URL patterns

### Requirement 7: Configuration Inheritance

**User Story:** As an administrator, I want global defaults with inheritance, so that I can configure common settings once and override as needed.

#### Acceptance Criteria

1. THE System SHALL support global defaults section in configuration files
2. WHEN processing agent configurations, THE System SHALL inherit from global defaults
3. THE System SHALL allow agent-specific overrides of inherited values
4. THE System SHALL support nested inheritance for complex configuration objects
5. WHEN merging configurations, THE System SHALL preserve type safety and validation

### Requirement 8: Hierarchical Caching Architecture

**User Story:** As an infrastructure operator, I want hierarchical caching, so that I can deploy regional caches with central fallback.

#### Acceptance Criteria

1. WHEN starting a cache server, THE System SHALL check DNS for root cache configuration
2. IF a root cache is found, THEN THE Cache_Server SHALL use it as fallback for cache misses
3. THE Cache_Server SHALL support multi-level fallback chains (regional → root → direct)
4. WHEN root cache is unavailable, THE Cache_Server SHALL continue operating independently
5. THE System SHALL prevent circular cache dependencies through validation
6. WHEN upstream cache requests fail, THE Cache_Server SHALL log the failure and fall back to the next level without blocking client requests

### Requirement 9: AX Protocol Compliance

**User Story:** As a system integrator, I want strict AX protocol compliance, so that I can interoperate with other AX-compliant systems.

#### Acceptance Criteria

1. WHEN generating AX records, THE System SHALL create valid JSON following AX 1.0 specification exactly
2. THE System SHALL generate standard well-known URI paths (/.well-known/agent-exchange.json)
3. THE System SHALL validate all required AX fields are present and correctly formatted
4. THE System SHALL support _agent.<domain> subdomain configuration for AX endpoints
5. WHEN fetching AX records, THE System SHALL validate HTTPS certificate chains properly
6. WHEN fetching AX records, THE System SHALL validate the response is served from /.well-known/agent-exchange.json path

### Requirement 10: Memory-Based Cache Storage

**User Story:** As an operator, I want in-memory cache storage for the MVP, so that I can deploy and test the core caching functionality.

#### Acceptance Criteria

1. THE Cache_Server SHALL use in-memory storage as the default backend for MVP deployment
2. THE Cache_Server SHALL store cached records in memory using HashMap or similar structure
3. THE Cache_Server SHALL implement LRU eviction when memory limits are reached
4. WHEN the server restarts, THE Cache_Server SHALL start with an empty cache
5. THE Cache_Server SHALL support configurable maximum cache size in number of cached agent records
6. WHEN storing records, THE Cache_Server SHALL respect TTL expiration times
7. THE Cache_Server SHALL support manual cache invalidation via CLI command
8. THE Cache_Server SHALL respect Cache-Control headers from AX endpoints

### Requirement 11: DNS-Based Cache Discovery

**User Story:** As a client, I want automatic cache discovery, so that I can use organizational caches without manual configuration.

#### Acceptance Criteria

1. WHEN initializing a client, THE System SHALL perform DNS TXT lookup for _ax-cache.<domain>
2. IF a cache endpoint is found, THEN THE System SHALL use it for discovery requests
3. IF no cache is found, THEN THE System SHALL fall back to direct HTTPS discovery
4. THE System SHALL parse cache endpoint URLs from DNS TXT record format
5. WHEN cache discovery fails, THE System SHALL continue with direct discovery without errors
6. THE System SHALL parse DNS TXT records in format: endpoint=<url>

### Requirement 12: Policy Enforcement

**User Story:** As a security administrator, I want policy enforcement capabilities, so that I can control which agents can be discovered by my organization.

#### Acceptance Criteria

1. WHEN domain locking is enabled, THE Cache_Server SHALL only cache agents from the home domain
2. WHEN external domains are blocked, THE Cache_Server SHALL reject discovery requests for those domains
3. THE Cache_Server SHALL log all discovery requests for audit purposes
4. THE Cache_Server SHALL support allowlist configuration for approved external domains
5. WHEN policy violations occur, THE Cache_Server SHALL return appropriate error responses

### Requirement 13: Configuration Management

**User Story:** As an administrator, I want flexible configuration options, so that I can customize the system for different deployment scenarios.

#### Acceptance Criteria

1. THE System SHALL support YAML configuration files for all components
2. THE System SHALL provide command-line argument overrides for key settings
3. THE System SHALL support environment variable configuration
4. WHEN loading configuration, THE System SHALL validate:
   - Domain names are valid DNS format
   - Port numbers are in valid range (1-65535)
   - URLs are properly formatted HTTPS endpoints
   - TTL values are positive integers
5. THE System SHALL provide clear error messages for configuration problems

### Requirement 14: Structured Logging

**User Story:** As an operator, I want structured logging with the tracing framework, so that I can monitor system health and troubleshoot issues effectively.

#### Acceptance Criteria

1. THE System SHALL use the tracing crate for structured logging throughout all components
2. THE System SHALL support configurable log levels (error, warn, info, debug, trace)
3. THE System SHALL log all HTTP requests and responses with structured fields: method, path, status_code, duration_ms, client_ip, cache_hit
4. THE System SHALL include correlation IDs for tracking requests across components
5. WHEN errors occur, THE System SHALL log structured error context with spans

### Requirement 15: Rate Limiting and Security

**User Story:** As an operator, I want rate limiting on the cache server, so that I can prevent abuse and ensure fair resource usage.

#### Acceptance Criteria

1. THE Cache_Server SHALL implement per-client IP rate limiting
2. THE Cache_Server SHALL support configurable rate limits (requests per minute/hour)
3. WHEN rate limits are exceeded, THE Cache_Server SHALL return HTTP 429 responses
4. THE Cache_Server SHALL validate HTTPS certificates when fetching AX records
5. THE Cache_Server SHALL implement configurable request timeouts with a default of 30 seconds for external AX fetches
6. WHEN returning errors, THE Cache_Server SHALL use HTTP status codes:
   - 400 for invalid requests
   - 404 for not found
   - 429 for rate limiting
   - 502 for upstream fetch failures
   - 503 for service unavailable

### Requirement 16: Testing Strategy

**User Story:** As a developer, I want comprehensive test coverage, so that I can ensure system reliability and catch regressions.

#### Acceptance Criteria

1. THE System SHALL include unit tests for all core functionality
2. THE System SHALL provide integration tests for multi-component workflows
3. THE System SHALL include property-based tests for data validation
4. THE System SHALL support mock HTTP servers for testing network interactions
5. THE System SHALL include tests for all public API functions and critical error paths

### Requirement 17: Build and Distribution

**User Story:** As a user, I want easy installation and deployment options, so that I can quickly get open-tethyr running in my environment.

#### Acceptance Criteria

1. THE System SHALL support cross-compilation for Linux, macOS, and Windows platforms
2. THE System SHALL generate Docker images for containerized deployment
3. THE System SHALL provide GitHub Actions workflows for automated builds and releases
4. THE System SHALL publish Docker images to GitHub Container Registry
5. WHEN building releases, THE System SHALL create static binaries with minimal dependencies
6. THE System SHALL produce statically-linked binaries for Linux using musl libc

### Requirement 18: Protocol Version Handling

**User Story:** As a system operator, I want version compatibility handling, so that the system can handle protocol evolution gracefully.

#### Acceptance Criteria

1. THE System SHALL validate AX record version field is "1.0"
2. WHEN encountering unsupported versions, THE System SHALL log a warning
3. THE System SHALL skip agents with unsupported protocol versions
4. THE System SHALL include version information in error messages