//! Generate command implementation

use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct GenerateCommand {
    /// Configuration file path
    #[arg(short, long)]
    pub config: PathBuf,

    /// Output directory path
    #[arg(short, long)]
    pub output: PathBuf,

    /// Validate generated records
    #[arg(long)]
    pub validate: bool,
}

impl GenerateCommand {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Load YAML config
        let config = open_tethyr::config::load_config(&self.config)?;

        // Validate config
        open_tethyr::config::ConfigValidator::validate_config(&config)?;

        // Generate AX document
        let doc = open_tethyr::ax::AxGenerator::generate_record(&config)?;

        // Optionally validate generated records
        if self.validate {
            for (i, record) in doc.records.iter().enumerate() {
                if let Err(e) = open_tethyr::ax::AxValidator::validate_record(record) {
                    eprintln!("Validation error in record {}: {}", i, e);
                    return Err(e.into());
                }
            }
            eprintln!("All {} records passed validation", doc.records.len());
        }

        // Write to well-known structure
        let result =
            open_tethyr::ax::AxGenerator::generate_well_known_structure(&doc, &self.output)?;
        println!(
            "Generated AX records at {}",
            result.ax_record_path.display()
        );
        println!("  Records: {}", doc.records.len());
        for record in &doc.records {
            println!("  - {} ({})", record.agent.name, record.agent.provider);
        }

        Ok(())
    }
}
