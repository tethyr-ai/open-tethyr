//! Configuration System
//!
//! This module provides YAML configuration parsing, inheritance,
//! merging, and validation capabilities.

mod loader;
mod merger;
mod models;
mod validator;

pub use loader::ConfigLoader;
pub use merger::ConfigMerger;
pub use models::{AgentConfig, AgentDefaults, AgentDefinition};
pub use validator::ConfigValidator;
