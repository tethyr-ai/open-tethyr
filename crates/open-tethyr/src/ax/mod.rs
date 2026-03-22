//! AX Protocol Implementation - aligned with official AX draft spec

mod generator;
mod record;
mod types;
mod validator;

pub use generator::AxGenerator;
pub use record::{parse_ax_json, Agent, AgentExchangeDocument, AgentExchangeRecord};
pub use types::{Capabilities, Endpoint, Limits, Protocol, Schema, Security};
pub use validator::{AxValidator, Severity, ValidationItem, ValidationReport};

pub use crate::http::file_writer::{FileWriter, WellKnownFiles};
