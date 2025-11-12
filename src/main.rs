mod config;

use anyhow::{Context, Result};
use clap::Parser;
use config::{CliConfig, Config};
use std::io;
use transaction_processor::{engine::TransactionEngine, transaction::Transaction};

fn main() -> Result<()> {
    let config = CliConfig::parse();

    process_transactions(&config).context("Failed to process transactions")?;

    Ok(())
}

fn process_transactions<C: Config>(config: &C) -> Result<()> {
    let mut engine = TransactionEngine::new();

    // Read and process transactions from CSV
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(config.input_path())
        .context("Failed to open input file")?;

    for result in reader.deserialize() {
        let transaction: Transaction = result.context("Failed to parse transaction")?;

        // Process transaction, ignoring errors for individual transactions
        // as per spec: "you can ignore it and assume this is an error on our partners
        // side"
        let _ = engine.process(transaction);
    }

    // Write results to stdout (streaming)
    let stdout = io::stdout();
    let handle = stdout.lock();

    let mut writer = csv::WriterBuilder::new().from_writer(handle);

    for account in engine.get_accounts() {
        writer
            .serialize(account)
            .context("Failed to serialize account")?;
    }

    writer.flush().context("Failed to flush stdout")?;

    Ok(())
}
