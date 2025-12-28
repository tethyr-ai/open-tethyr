//! Configuration System
//!
//! This module provides YAML configuration parsing, inheritance,
//! merging, and validation capabilities.

mod models;
mod merger;
mod validator;
mod loader;

pub use models::{AgentConfig, AgentDefaults, AgentDefinition};
pub use merger::ConfigMerger;
pub use validator::ConfigValidator;
pub use loader::ConfigLoader;