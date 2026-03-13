use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct ValidateCommand {
    pub file: PathBuf,
}

impl ValidateCommand {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(&self.file)?;
        let records = open_tethyr::parse_ax_json(&content)?;
        let mut has_errors = false;
        for (i, record) in records.iter().enumerate() {
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
            eprintln!("\nValidation FAILED");
            std::process::exit(3);
        } else {
            println!("Validation PASSED: {} record(s) valid", records.len());
        }
        Ok(())
    }
}
