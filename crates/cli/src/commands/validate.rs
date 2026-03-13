//! Validate command implementation

use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct ValidateCommand {
    /// AX record file path (positional)
    pub file: PathBuf,
}

impl ValidateCommand {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Read and parse the AX JSON file
        let content = std::fs::read_to_string(&self.file)?;
        let doc: open_tethyr::ax::AgentExchangeDocument = serde_json::from_str(&content)?;

        let mut has_errors = false;

        for (i, record) in doc.records.iter().enumerate() {
            let report = open_tethyr::ax::AxValidator::validate_record_detailed(record);

            if report.has_errors() {
                has_errors = true;
                eprintln!("Record {} ({}):", i, record.agent.name);
                for item in report.errors() {
                    eprintln!("  ERROR [{}]: {}", item.field, item.message);
                }
            }

            for item in report.warnings() {
                eprintln!("  WARN  [{}]: {}", item.field, item.message);
            }
        }

        if has_errors {
            eprintln!(
                "\nValidation FAILED: {} record(s) with errors",
                doc.records.len()
            );
            std::process::exit(3);
        } else {
            println!("Validation PASSED: {} record(s) valid", doc.records.len());
        }

        Ok(())
    }
}
