//! AX Protocol Implementation

mod generator;
mod record;
mod types;
mod validator;

pub use generator::AxGenerator;
pub use record::{Agent, AgentExchangeDocument, AgentExchangeRecord};
pub use types::{Capabilities, Endpoint, Limits, OAuthEndpoints, Protocol, Schema, Security};
pub use validator::{AxValidator, Severity, ValidationItem, ValidationReport};

// Re-export file writer types through ax module for convenience
pub use crate::http::file_writer::{FileWriter, WellKnownFiles};
