//! File Writer - AX spec compliant: agent-exchange (no .json extension)

use crate::ax::AgentExchangeRecord;
use crate::error::AxError;
use std::path::{Path, PathBuf};

pub struct WellKnownFiles {
    pub base_path: PathBuf,
    pub ax_record_path: PathBuf,
}

pub struct FileWriter;

impl FileWriter {
    pub fn write_structure(
        record: &AgentExchangeRecord,
        output_dir: &Path,
    ) -> Result<WellKnownFiles, AxError> {
        let well_known_dir = output_dir.join(".well-known");
        std::fs::create_dir_all(&well_known_dir)
            .map_err(|e| AxError::FileWriteFailed(e.to_string()))?;
        let ax_path = well_known_dir.join("agent-exchange");
        Self::write_ax_record(record, &ax_path)?;
        Ok(WellKnownFiles {
            base_path: output_dir.to_path_buf(),
            ax_record_path: ax_path,
        })
    }

    pub fn write_ax_record(record: &AgentExchangeRecord, path: &Path) -> Result<(), AxError> {
        let json = serde_json::to_string_pretty(record)
            .map_err(|e| AxError::GenerationFailed(e.to_string()))?;
        std::fs::write(path, json).map_err(|e| AxError::FileWriteFailed(e.to_string()))?;
        Ok(())
    }
}
