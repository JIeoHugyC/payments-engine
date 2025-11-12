pub mod engine;
pub mod transaction;

use engine::TransactionEngine;
use transaction::Transaction;

/// Process a batch of transactions and return the final account states
pub fn process_batch(transactions: impl Iterator<Item = Transaction>) -> Vec<transaction::Account> {
    let mut engine = TransactionEngine::new();

    for tx in transactions {
        let _ = engine.process(tx);
    }

    engine.get_accounts()
}
