//! File Writer for AX well-known structure

use crate::ax::AgentExchangeDocument;
use crate::error::AxError;
use std::path::{Path, PathBuf};

/// Well-known file structure output
pub struct WellKnownFiles {
    pub base_path: PathBuf,
    pub ax_record_path: PathBuf,
}

/// File writer for AX record output
pub struct FileWriter;

impl FileWriter {
    /// Write AX document to the well-known file structure
    pub fn write_structure(
        doc: &AgentExchangeDocument,
        output_dir: &Path,
    ) -> Result<WellKnownFiles, AxError> {
        let well_known_dir = output_dir.join(".well-known");
        std::fs::create_dir_all(&well_known_dir)
            .map_err(|e| AxError::FileWriteFailed(e.to_string()))?;

        let ax_path = well_known_dir.join("agent-exchange.json");
        Self::write_ax_record(doc, &ax_path)?;

        Ok(WellKnownFiles {
            base_path: output_dir.to_path_buf(),
            ax_record_path: ax_path,
        })
    }

    /// Write AX document to a specific path
    pub fn write_ax_record(doc: &AgentExchangeDocument, path: &Path) -> Result<(), AxError> {
        let json = serde_json::to_string_pretty(doc)
            .map_err(|e| AxError::GenerationFailed(e.to_string()))?;
        std::fs::write(path, json).map_err(|e| AxError::FileWriteFailed(e.to_string()))?;
        Ok(())
    }
}
