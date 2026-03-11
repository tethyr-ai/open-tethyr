//! Generate command implementation

use clap::Args;
use open_tethyr::ax::{AxGenerator, AxValidator};
use open_tethyr::config::ConfigLoader;
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
    /// Execute the generate command
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Load YAML configuration with inheritance
        let config = ConfigLoader::load_from_file(&self.config).await?;

        // Generate AX records
        let document = AxGenerator::generate_record(&config)?;

        // Validate if requested
        if self.validate {
            for record in &document.records {
                AxValidator::validate_record(record)?;
            }
        }

        // Generate well-known file structure
        let files = AxGenerator::generate_well_known_structure(&document)?;

        // Write to output directory
        AxGenerator::write_well_known_structure(&self.output, &files)?;

        println!(
            "Successfully generated {} AX record(s) to {}",
            document.records.len(),
            self.output.display()
        );

        Ok(())
    }
}
