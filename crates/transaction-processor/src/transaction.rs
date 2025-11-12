use anyhow::{bail, Result};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Client ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ClientId(pub u16);

/// Transaction ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct TransactionId(pub u32);

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
    pub tx: TransactionId,
    pub client: ClientId,
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
}

/// Client account state (internal representation)
#[derive(Debug, Clone, Default)]
pub struct Account {
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

/// Account output for CSV serialization
#[derive(Debug, Serialize)]
pub struct AccountOutput {
    pub client: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl AccountOutput {
    pub fn new(client: ClientId, account: &Account) -> Self {
        Self {
            client: client.0,
            available: account.available,
            held: account.held,
            total: account.total,
            locked: account.locked,
        }
    }
}

impl Account {
    pub(crate) fn new() -> Self {
        Self::default()
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
