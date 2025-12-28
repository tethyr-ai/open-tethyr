//! AX Protocol Implementation
//!
//! This module provides the core AX (Agent Exchange) protocol implementation,
//! including data structures, validation, and generation capabilities.

mod record;
mod validator;
mod generator;
mod types;

pub use record::{AgentExchangeRecord, Agent, AgentExchangeDocument};
pub use validator::AxValidator;
pub use generator::AxGenerator;
pub use types::{Endpoint, Protocol, Security, Capabilities, Schema, Limits};