//! # Open-Tethyr Rust Toolkit
//!
//! AX (Agent Discovery Exchange) protocol implementation.
//!
//! ## Example
//!
//! ```rust,no_run
//! use open_tethyr::OpenTethyr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = OpenTethyr::new("acme.com")?;
//!     let record = client.discover("api.partner.com").await?;
//!     println!("Found agent: {}", record.agent.name);
//!     Ok(())
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod auth;
pub mod ax;
pub mod cache;
pub mod config;
pub mod dns;
pub mod error;
pub mod http;

#[cfg(feature = "client")]
#[cfg_attr(docsrs, doc(cfg(feature = "client")))]
pub mod client;
#[cfg(feature = "client")]
pub use client::OpenTethyr;

#[cfg(feature = "server")]
#[cfg_attr(docsrs, doc(cfg(feature = "server")))]
pub mod server;
#[cfg(feature = "server")]
pub use server::CacheServer;

pub use ax::{
    parse_ax_json, Agent, AgentExchangeDocument, AgentExchangeRecord, Endpoint, Protocol,
};
pub use config::AgentConfig;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
