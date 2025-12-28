//! DNS Discovery
//!
//! This module provides DNS-based discovery for AX cache endpoints
//! using TXT record lookups.

mod discovery;
mod types;

pub use discovery::DnsDiscovery;
pub use types::DnsError;
