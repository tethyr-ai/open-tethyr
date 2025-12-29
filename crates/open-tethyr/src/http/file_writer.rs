//! File Writer Utility
//!
//! Utilities for writing well-known file structures to disk with atomic operations.

use crate::ax::AgentExchangeDocument;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Well-known file structure for AX records
#[derive(Debug, Clone)]
pub struct WellKnownFiles {
    /// The main agent-exchange.json file content
    pub agent_exchange_json: String,
    /// Additional metadata files (future expansion)
    pub metadata: Vec<(String, String)>,
}

/// File writer for well-known structures
pub struct FileWriter;

impl FileWriter {
    /// Write well-known file structure to disk
    /// Creates the standard /.well-known/agent-exchange.json structure
    pub fn write_structure(
        output_dir: &Path,
        files: &WellKnownFiles,
    ) -> Result<(), FileWriterError> {
        // Create the .well-known directory
        let well_known_dir = output_dir.join(".well-known");
        fs::create_dir_all(&well_known_dir).map_err(|e| FileWriterError::DirectoryCreation {
            path: well_known_dir.clone(),
            source: e,
        })?;

        // Write the main agent-exchange.json file atomically
        let agent_exchange_path = well_known_dir.join("agent-exchange.json");
        Self::write_file_atomic(&agent_exchange_path, &files.agent_exchange_json)?;

        // Write any additional metadata files
        for (filename, content) in &files.metadata {
            let metadata_path = well_known_dir.join(filename);
            Self::write_file_atomic(&metadata_path, content)?;
        }

        Ok(())
    }

    /// Write an AX record document to a specific path
    pub fn write_ax_record(
        path: &Path,
        document: &AgentExchangeDocument,
    ) -> Result<(), FileWriterError> {
        let json_content =
            serde_json::to_string_pretty(document).map_err(FileWriterError::Serialization)?;

        Self::write_file_atomic(path, &json_content)
    }

    /// Generate well-known file structure from an AX document
    pub fn generate_well_known_files(
        document: &AgentExchangeDocument,
    ) -> Result<WellKnownFiles, FileWriterError> {
        let agent_exchange_json =
            serde_json::to_string_pretty(document).map_err(FileWriterError::Serialization)?;

        Ok(WellKnownFiles {
            agent_exchange_json,
            metadata: Vec::new(), // Future expansion for additional metadata
        })
    }

    /// Write file content atomically using a temporary file
    /// This ensures that the file is either completely written or not written at all
    fn write_file_atomic(path: &Path, content: &str) -> Result<(), FileWriterError> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| FileWriterError::DirectoryCreation {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        // Create a temporary file in the same directory as the target
        let temp_dir = path.parent().unwrap_or_else(|| Path::new("."));
        let mut temp_file =
            NamedTempFile::new_in(temp_dir).map_err(|e| FileWriterError::TempFileCreation {
                path: temp_dir.to_path_buf(),
                source: e,
            })?;

        // Write content to temporary file
        temp_file
            .write_all(content.as_bytes())
            .map_err(|e| FileWriterError::Write {
                path: path.to_path_buf(),
                source: e,
            })?;

        // Ensure all data is written to disk
        temp_file.flush().map_err(|e| FileWriterError::Write {
            path: path.to_path_buf(),
            source: e,
        })?;

        // Atomically move the temporary file to the target location
        temp_file
            .persist(path)
            .map_err(|e| FileWriterError::AtomicMove {
                from: e.file.path().to_path_buf(),
                to: path.to_path_buf(),
                source: e.error,
            })?;

        Ok(())
    }
}

/// Errors that can occur during file writing operations
#[derive(Debug, thiserror::Error)]
pub enum FileWriterError {
    #[error("Failed to create directory {path:?}")]
    DirectoryCreation {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Failed to create temporary file in {path:?}")]
    TempFileCreation {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Failed to write to file {path:?}")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Failed to atomically move file from {from:?} to {to:?}")]
    AtomicMove {
        from: PathBuf,
        to: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Failed to serialize data to JSON")]
    Serialization(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ax::{Agent, AgentExchangeRecord, Endpoint, Protocol};
    use tempfile::TempDir;

    fn create_test_document() -> AgentExchangeDocument {
        let agent = Agent {
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            provider: "Test Provider".to_string(),
        };

        let endpoint = Endpoint {
            protocol: Protocol::Rest,
            url: "https://example.com/api".to_string(),
            auth: vec!["OAuth2".to_string()],
            content_type: Some("application/json".to_string()),
        };

        let record = AgentExchangeRecord {
            record_type: "AX".to_string(),
            version: "1.0".to_string(),
            agent,
            endpoints: vec![endpoint],
            capabilities: None,
            schema: None,
            limits: None,
            security: None,
            extensions: None,
        };

        AgentExchangeDocument {
            records: vec![record],
        }
    }

    #[test]
    fn test_write_ax_record() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test-record.json");
        let document = create_test_document();

        let result = FileWriter::write_ax_record(&file_path, &document);
        assert!(result.is_ok());

        // Verify file was created and contains valid JSON
        let content = fs::read_to_string(&file_path).unwrap();
        let parsed: AgentExchangeDocument = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.records.len(), document.records.len());
    }

    #[test]
    fn test_write_well_known_structure() {
        let temp_dir = TempDir::new().unwrap();
        let document = create_test_document();

        let files = FileWriter::generate_well_known_files(&document).unwrap();
        let result = FileWriter::write_structure(temp_dir.path(), &files);
        assert!(result.is_ok());

        // Verify the well-known directory and file were created
        let well_known_path = temp_dir
            .path()
            .join(".well-known")
            .join("agent-exchange.json");
        assert!(well_known_path.exists());

        // Verify content is valid
        let content = fs::read_to_string(&well_known_path).unwrap();
        let parsed: AgentExchangeDocument = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.records.len(), document.records.len());
    }

    #[test]
    fn test_atomic_write_operation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("atomic-test.json");

        // Write some content
        let result = FileWriter::write_file_atomic(&file_path, r#"{"test": "content"}"#);
        assert!(result.is_ok());

        // Verify file exists and has correct content
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, r#"{"test": "content"}"#);
    }
}
