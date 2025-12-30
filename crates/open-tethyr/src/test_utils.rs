//! Test Utilities and Mocks
//!
//! This module provides test utilities, mocks, and fixtures for testing
//! the open-tethyr library components.

#[cfg(test)]
pub mod mocks;

#[cfg(test)]
pub mod fixtures;

#[cfg(test)]
pub mod helpers;

// Re-export commonly used test utilities
#[cfg(test)]
pub use fixtures::*;
#[cfg(test)]
pub use helpers::*;
#[cfg(test)]
pub use mocks::{MockDnsResolver, MockHttpServer};
