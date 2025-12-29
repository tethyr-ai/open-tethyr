//! AX Protocol Implementation
//!
//! This module provides the core AX (Agent Exchange) protocol implementation,
//! including data structures, validation, and generation capabilities.

mod generator;
mod record;
mod types;
mod validator;

pub use generator::{AxGenerator, GenerationError, WellKnownFiles};
pub use record::{Agent, AgentExchangeDocument, AgentExchangeRecord};
pub use types::{Capabilities, Endpoint, Limits, Protocol, Schema, Security};
pub use validator::{AxValidator, ValidationError};
