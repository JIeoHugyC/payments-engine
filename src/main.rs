mod config;

use anyhow::{Context, Result};
use clap::Parser;
use config::{CliConfig, Config};
use std::io;
use tracing::{info, warn};
use transaction_processor::{
    engine::TransactionEngine,
    transaction::{AccountOutput, Transaction},
};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let config = CliConfig::parse();

    process_transactions(&config)?;

    info!("Processing completed successfully");

    Ok(())
}

fn process_transactions<C: Config>(config: &C) -> Result<()> {
    let mut engine = TransactionEngine::new();

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(config.input_path())
        .context("Failed to open input file")?;

    let mut processed = 0;
    let mut skipped = 0;

    for result in reader.deserialize() {
        let transaction: Transaction = match result {
            Ok(tx) => tx,
            Err(e) => {
                warn!("Failed to parse transaction: {e}");
                skipped += 1;

                continue;
            }
        };

        if let Err(e) = engine.process(transaction) {
            warn!("Transaction processing error: {e}");
            skipped += 1;
        } else {
            processed += 1;
        }
    }

    info!("Processed {processed} transactions, skipped {skipped} invalid transactions",);

    let stdout = io::stdout();
    let handle = stdout.lock();
    let mut writer = csv::WriterBuilder::new().from_writer(handle);

    for (client_id, account) in &engine.accounts {
        let output = AccountOutput::new(*client_id, account);
        writer
            .serialize(&output)
            .context("Failed to serialize account")?;
    }

    writer.flush().context("Failed to flush stdout")?;

    Ok(())
}
