//! Structured Logging Configuration
//!
//! Provides centralized logging configuration with support for:
//! - Configurable log levels
//! - Structured field formatting (JSON and text)
//! - Correlation ID propagation
//! - Environment-based configuration

use std::env;
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level (error, warn, info, debug, trace)
    pub level: Level,
    /// Output format (json or text)
    pub format: LogFormat,
    /// Enable ANSI colors in output
    pub enable_colors: bool,
    /// Include file and line numbers
    pub include_location: bool,
}

/// Log output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Human-readable text format
    Text,
    /// Structured JSON format
    Json,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            format: LogFormat::Text,
            enable_colors: true,
            include_location: false,
        }
    }
}

impl LoggingConfig {
    /// Create a new logging configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the log level
    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Set the output format
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Enable or disable ANSI colors
    pub fn with_colors(mut self, enable: bool) -> Self {
        self.enable_colors = enable;
        self
    }

    /// Enable or disable file/line location info
    pub fn with_location(mut self, enable: bool) -> Self {
        self.include_location = enable;
        self
    }

    /// Create configuration from environment variables
    ///
    /// Supported environment variables:
    /// - `RUST_LOG`: Log level filter (e.g., "info", "debug", "open_tethyr=debug")
    /// - `LOG_FORMAT`: Output format ("json" or "text")
    /// - `LOG_COLORS`: Enable colors ("true" or "false")
    /// - `LOG_LOCATION`: Include file/line info ("true" or "false")
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Parse log level from RUST_LOG if set
        if let Ok(level_str) = env::var("RUST_LOG") {
            if let Ok(level) = level_str.parse::<Level>() {
                config.level = level;
            }
        }

        // Parse format
        if let Ok(format_str) = env::var("LOG_FORMAT") {
            match format_str.to_lowercase().as_str() {
                "json" => config.format = LogFormat::Json,
                "text" => config.format = LogFormat::Text,
                _ => {}
            }
        }

        // Parse colors
        if let Ok(colors_str) = env::var("LOG_COLORS") {
            config.enable_colors = colors_str.to_lowercase() == "true";
        }

        // Parse location
        if let Ok(location_str) = env::var("LOG_LOCATION") {
            config.include_location = location_str.to_lowercase() == "true";
        }

        config
    }

    /// Initialize the global tracing subscriber with this configuration
    pub fn init(self) -> Result<(), Box<dyn std::error::Error>> {
        // Build the env filter
        let env_filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new(self.level.as_str()))?;

        match self.format {
            LogFormat::Json => {
                // JSON format for structured logging
                let fmt_layer = fmt::layer()
                    .json()
                    .with_current_span(true)
                    .with_span_list(true)
                    .with_file(self.include_location)
                    .with_line_number(self.include_location);

                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(fmt_layer)
                    .init();
            }
            LogFormat::Text => {
                // Human-readable text format
                let fmt_layer = fmt::layer()
                    .with_ansi(self.enable_colors)
                    .with_file(self.include_location)
                    .with_line_number(self.include_location)
                    .with_target(true);

                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(fmt_layer)
                    .init();
            }
        }

        Ok(())
    }

    /// Initialize with default configuration
    pub fn init_default() -> Result<(), Box<dyn std::error::Error>> {
        Self::default().init()
    }

    /// Initialize from environment variables
    pub fn init_from_env() -> Result<(), Box<dyn std::error::Error>> {
        Self::from_env().init()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, Level::INFO);
        assert_eq!(config.format, LogFormat::Text);
        assert!(config.enable_colors);
        assert!(!config.include_location);
    }

    #[test]
    fn test_builder_pattern() {
        let config = LoggingConfig::new()
            .with_level(Level::DEBUG)
            .with_format(LogFormat::Json)
            .with_colors(false)
            .with_location(true);

        assert_eq!(config.level, Level::DEBUG);
        assert_eq!(config.format, LogFormat::Json);
        assert!(!config.enable_colors);
        assert!(config.include_location);
    }

    #[test]
    fn test_from_env_defaults() {
        // Clear environment variables
        env::remove_var("RUST_LOG");
        env::remove_var("LOG_FORMAT");
        env::remove_var("LOG_COLORS");
        env::remove_var("LOG_LOCATION");

        let config = LoggingConfig::from_env();
        assert_eq!(config.level, Level::INFO);
        assert_eq!(config.format, LogFormat::Text);
    }

    #[test]
    fn test_from_env_with_vars() {
        env::set_var("LOG_FORMAT", "json");
        env::set_var("LOG_COLORS", "false");
        env::set_var("LOG_LOCATION", "true");

        let config = LoggingConfig::from_env();
        assert_eq!(config.format, LogFormat::Json);
        assert!(!config.enable_colors);
        assert!(config.include_location);

        // Cleanup
        env::remove_var("LOG_FORMAT");
        env::remove_var("LOG_COLORS");
        env::remove_var("LOG_LOCATION");
    }
}
