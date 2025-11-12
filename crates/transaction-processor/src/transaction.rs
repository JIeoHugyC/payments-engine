use anyhow::{bail, Result};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Transaction type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// Transaction record from CSV
#[derive(Debug, Clone, Deserialize)]
pub struct Transaction {
    #[serde(default)]
    pub amount: Option<Decimal>,
    /// Transaction ID
    pub tx: u32,
    /// Client ID
    pub client: u16,
    /// Transaction type
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
}

/// Client account state
/// Field order matches required CSV output: client,available,held,total,locked
#[derive(Debug, Clone, Serialize, Default)]
pub struct Account {
    pub client: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Account {
    pub(crate) fn new(client: u16) -> Self {
        Self {
            client,
            ..Default::default()
        }
    }

    pub(crate) fn deposit(&mut self, amount: Decimal) {
        self.available += amount;
        self.total += amount;
    }

    pub(crate) fn withdraw(&mut self, amount: Decimal) -> Result<()> {
        if self.available >= amount {
            self.available -= amount;
            self.total -= amount;

            return Ok(());
        }

        bail!("Insufficient funds")
    }

    pub(crate) fn dispute(&mut self, amount: Decimal) {
        self.available -= amount;
        self.held += amount;
    }

    pub(crate) fn resolve(&mut self, amount: Decimal) {
        self.held -= amount;
        self.available += amount;
    }

    pub(crate) fn chargeback(&mut self, amount: Decimal) {
        self.held -= amount;
        self.total -= amount;
        self.locked = true;
    }
}
