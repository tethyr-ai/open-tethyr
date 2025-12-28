//! HTTP Client
//!
//! This module provides HTTP client functionality for fetching AX records
//! and file writing utilities for well-known structures.

mod client;
mod file_writer;

pub use client::AxHttpClient;
pub use file_writer::{FileWriter, FileWriterError, WellKnownFiles};
