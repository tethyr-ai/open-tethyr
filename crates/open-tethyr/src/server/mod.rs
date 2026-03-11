//! Cache server implementation
//!
//! This module provides the HTTP cache server functionality.

pub mod cache_server;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod policy;

pub use cache_server::{AppState, CacheMetrics, CacheServer, ServerConfig};
pub use error::ServerError;
