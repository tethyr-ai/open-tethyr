//! Validate command implementation

use clap::Args;
use open_tethyr::ax::{AgentExchangeDocument, AxValidator};
use std::path::PathBuf;

#[derive(Args)]
pub struct ValidateCommand {
    /// AX record file path
    #[arg(short, long)]
    pub file: PathBuf,
}

impl ValidateCommand {
    /// Execute the validate command
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Read the file
        let content = std::fs::read_to_string(&self.file)?;

        // Parse the JSON
        let document: AgentExchangeDocument = serde_json::from_str(&content)?;

        // Validate each record
        let mut errors = Vec::new();
        for (index, record) in document.records.iter().enumerate() {
            if let Err(e) = AxValidator::validate_record(record) {
                errors.push((index, e));
            }
        }

        // Report results
        if errors.is_empty() {
            println!("✓ Validation successful!");
            println!("  File: {}", self.file.display());
            println!("  Records validated: {}", document.records.len());
            Ok(())
        } else {
            eprintln!("✗ Validation failed!");
            eprintln!("  File: {}", self.file.display());
            eprintln!("  Errors found: {}", errors.len());
            eprintln!();
            for (index, error) in &errors {
                eprintln!("  Record {}: {}", index, error);
            }
            Err(format!("Validation failed with {} error(s)", errors.len()).into())
        }
    }
}
