//! # Open-Tethyr Rust Toolkit
//!
//! A distributed caching system for agent discovery implementing the Agent Discovery Exchange (AX) protocol.
//!
//! This library provides:
//! - AX protocol compliance and validation
//! - DNS-based cache discovery
//! - Hierarchical caching architecture
//! - OAuth provider templates
//! - Configuration management with inheritance
//! - Policy enforcement and rate limiting
//!
//! ## Features
//!
//! - `client` (default): Client SDK for agent discovery
//! - `server`: Cache server functionality
//! - `full`: Both client and server features
//!
//! ## Example
//!
//! ```rust,no_run
//! use open_tethyr::OpenTethyr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = OpenTethyr::new("acme.com")?;
//!     let doc = client.discover("api.partner.com").await?;
//!     println!("Found {} agents", doc.records.len());
//!     Ok(())
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

// Core modules (always available)
pub mod auth;
pub mod ax;
pub mod cache;
pub mod config;
pub mod dns;
pub mod error;
pub mod http;

// Client SDK (default feature)
#[cfg(feature = "client")]
#[cfg_attr(docsrs, doc(cfg(feature = "client")))]
pub mod client;

#[cfg(feature = "client")]
pub use client::OpenTethyr;

// Server functionality (opt-in)
#[cfg(feature = "server")]
#[cfg_attr(docsrs, doc(cfg(feature = "server")))]
pub mod server;

#[cfg(feature = "server")]
pub use server::CacheServer;

// Re-export commonly used types
pub use ax::{Agent, AgentExchangeDocument, AgentExchangeRecord, Endpoint, Protocol};
pub use config::AgentConfig;

/// Result type alias for the library
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
