use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct GenerateCommand {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(short, long)]
    pub output: PathBuf,
    #[arg(long)]
    pub validate: bool,
}

impl GenerateCommand {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = open_tethyr::config::load_config(&self.config)?;
        open_tethyr::config::ConfigValidator::validate_config(&config)?;
        let record = open_tethyr::ax::AxGenerator::generate_record(&config)?;
        if self.validate {
            open_tethyr::ax::AxValidator::validate_record(&record)?;
            eprintln!("Record passed validation");
        }
        let result =
            open_tethyr::ax::AxGenerator::generate_well_known_structure(&record, &self.output)?;
        println!("Generated AX record at {}", result.ax_record_path.display());
        println!(
            "  Agent: {} ({})",
            record.agent.name,
            record.agent.provider.as_deref().unwrap_or("unspecified")
        );
        Ok(())
    }
}
