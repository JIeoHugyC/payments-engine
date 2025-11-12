use clap::Parser;
use std::path::{Path, PathBuf};

/// Trait for reading configuration parameters
pub trait Config {
    fn input_path(&self) -> &Path;
}

/// CLI configuration
#[derive(Parser, Debug)]
#[command(
    name = "payments-engine",
    about = "A simple toy payments engine that processes transactions from CSV",
    version
)]
pub struct CliConfig {
    /// Path to the input CSV file containing transactions
    #[arg(value_name = "INPUT_FILE")]
    input_file: PathBuf,
}

impl Config for CliConfig {
    fn input_path(&self) -> &Path {
        &self.input_file
    }
}
