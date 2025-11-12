use crate::transaction::{Account, Transaction, TransactionType};
use anyhow::{anyhow, bail, Result};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Stored transaction for dispute handling
#[derive(Debug, Clone)]
struct StoredTransaction {
    amount: Decimal,
    client: u16,
    disputed: bool,
}

/// Main transaction processing engine
#[derive(Default)]
pub struct TransactionEngine {
    accounts: HashMap<u16, Account>,
    transactions: HashMap<u32, StoredTransaction>,
}

impl TransactionEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&mut self, transaction: Transaction) -> Result<()> {
        match transaction.tx_type {
            TransactionType::Deposit => self.process_deposit(transaction),
            TransactionType::Withdrawal => self.process_withdrawal(transaction),
            TransactionType::Dispute => self.process_dispute(transaction),
            TransactionType::Resolve => self.process_resolve(transaction),
            TransactionType::Chargeback => self.process_chargeback(transaction),
        }
    }

    fn process_deposit(&mut self, tx: Transaction) -> Result<()> {
        let amount = tx
            .amount
            .ok_or_else(|| anyhow!("Deposit requires amount"))?;

        let account = self
            .accounts
            .entry(tx.client)
            .or_insert_with(|| Account::new(tx.client));

        if account.locked {
            bail!("Account is locked");
        }

        // Check for duplicate transaction ID
        if self.transactions.contains_key(&tx.tx) {
            bail!("Duplicate transaction ID");
        }

        account.deposit(amount);

        // Store transaction for potential disputes
        self.transactions.insert(
            tx.tx,
            StoredTransaction {
                client: tx.client,
                amount,
                disputed: false,
            },
        );

        Ok(())
    }

    fn process_withdrawal(&mut self, tx: Transaction) -> Result<()> {
        let amount = tx
            .amount
            .ok_or_else(|| anyhow!("Withdrawal requires amount"))?;

        let account = self
            .accounts
            .entry(tx.client)
            .or_insert_with(|| Account::new(tx.client));

        if account.locked {
            bail!("Account is locked");
        }

        // Check for duplicate transaction ID
        if self.transactions.contains_key(&tx.tx) {
            bail!("Duplicate transaction ID");
        }

        account.withdraw(amount)?;

        // Store transaction for potential disputes
        self.transactions.insert(
            tx.tx,
            StoredTransaction {
                client: tx.client,
                amount,
                disputed: false,
            },
        );

        Ok(())
    }

    fn process_dispute(&mut self, tx: Transaction) -> Result<()> {
        let stored = self
            .transactions
            .get_mut(&tx.tx)
            .ok_or_else(|| anyhow!("Transaction not found"))?;

        if stored.client != tx.client {
            bail!("Transaction belongs to different client");
        }

        if stored.disputed {
            bail!("Transaction already disputed");
        }

        stored.disputed = true;

        let account = self
            .accounts
            .get_mut(&tx.client)
            .ok_or_else(|| anyhow!("Account not found"))?;

        account.dispute(stored.amount);

        Ok(())
    }

    fn process_resolve(&mut self, tx: Transaction) -> Result<()> {
        let stored = self
            .transactions
            .get_mut(&tx.tx)
            .ok_or_else(|| anyhow!("Transaction not found"))?;

        if stored.client != tx.client {
            bail!("Transaction belongs to different client");
        }

        if !stored.disputed {
            bail!("Transaction not under dispute");
        }

        stored.disputed = false;

        let account = self
            .accounts
            .get_mut(&tx.client)
            .ok_or_else(|| anyhow!("Account not found"))?;

        account.resolve(stored.amount);

        Ok(())
    }

    fn process_chargeback(&mut self, tx: Transaction) -> Result<()> {
        let stored = self
            .transactions
            .get(&tx.tx)
            .ok_or_else(|| anyhow!("Transaction not found"))?;

        if stored.client != tx.client {
            bail!("Transaction belongs to different client");
        }

        if !stored.disputed {
            bail!("Transaction not under dispute");
        }

        let amount = stored.amount;

        let account = self
            .accounts
            .get_mut(&tx.client)
            .ok_or_else(|| anyhow!("Account not found"))?;

        account.chargeback(amount);

        Ok(())
    }

    pub fn get_accounts(&self) -> Vec<Account> {
        self.accounts.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_deposit() {
        let mut engine = TransactionEngine::new();

        let tx = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(Decimal::new(100, 1)), // 10.0
        };

        engine.process(tx).unwrap();

        let accounts = engine.get_accounts();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].available, Decimal::new(100, 1));
        assert_eq!(accounts[0].total, Decimal::new(100, 1));
    }

    #[test]
    fn test_withdrawal() {
        let mut engine = TransactionEngine::new();

        engine
            .process(Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(Decimal::new(100, 1)),
            })
            .unwrap();

        engine
            .process(Transaction {
                tx_type: TransactionType::Withdrawal,
                client: 1,
                tx: 2,
                amount: Some(Decimal::new(50, 1)),
            })
            .unwrap();

        let accounts = engine.get_accounts();
        assert_eq!(accounts[0].available, Decimal::new(50, 1));
        assert_eq!(accounts[0].total, Decimal::new(50, 1));
    }

    #[test]
    fn test_insufficient_funds() {
        let mut engine = TransactionEngine::new();

        engine
            .process(Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(Decimal::new(50, 1)),
            })
            .unwrap();

        let result = engine.process(Transaction {
            tx_type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some(Decimal::new(100, 1)),
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_dispute_resolve() {
        let mut engine = TransactionEngine::new();

        engine
            .process(Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(Decimal::new(100, 1)),
            })
            .unwrap();

        engine
            .process(Transaction {
                tx_type: TransactionType::Dispute,
                client: 1,
                tx: 1,
                amount: None,
            })
            .unwrap();

        let accounts = engine.get_accounts();
        assert_eq!(accounts[0].available, Decimal::ZERO);
        assert_eq!(accounts[0].held, Decimal::new(100, 1));
        assert_eq!(accounts[0].total, Decimal::new(100, 1));

        engine
            .process(Transaction {
                tx_type: TransactionType::Resolve,
                client: 1,
                tx: 1,
                amount: None,
            })
            .unwrap();

        let accounts = engine.get_accounts();
        assert_eq!(accounts[0].available, Decimal::new(100, 1));
        assert_eq!(accounts[0].held, Decimal::ZERO);
    }

    #[test]
    fn test_chargeback() {
        let mut engine = TransactionEngine::new();

        engine
            .process(Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(Decimal::new(100, 1)),
            })
            .unwrap();

        engine
            .process(Transaction {
                tx_type: TransactionType::Dispute,
                client: 1,
                tx: 1,
                amount: None,
            })
            .unwrap();

        engine
            .process(Transaction {
                tx_type: TransactionType::Chargeback,
                client: 1,
                tx: 1,
                amount: None,
            })
            .unwrap();

        let accounts = engine.get_accounts();
        assert_eq!(accounts[0].available, Decimal::ZERO);
        assert_eq!(accounts[0].held, Decimal::ZERO);
        assert_eq!(accounts[0].total, Decimal::ZERO);
        assert!(accounts[0].locked);
    }
}
