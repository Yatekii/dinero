mod ibkr;
mod neon;
mod revolut;
mod ubs;

use std::path::Path;

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{cli::BankFormat, fx::Symbol};

pub fn load(name: &str, path: impl AsRef<Path>, format: BankFormat) -> anyhow::Result<Vec<Ledger>> {
    match format {
        BankFormat::Neon => load_inner::<neon::Neon>(name, path),
        BankFormat::Ubs => load_inner::<ubs::Ubs>(name, path),
        BankFormat::Ibkr => load_inner::<ibkr::Ibkr>(name, path),
        BankFormat::Revolut => load_inner::<revolut::Revolut>(name, path),
    }
}

fn load_inner<T: Parser>(name: &str, path: impl AsRef<Path>) -> anyhow::Result<Vec<Ledger>> {
    let loaded = T::parse(
        name,
        std::fs::read_to_string(&path)
            .with_context(|| format!("could not read ledger CSV {}", path.as_ref().display()))?,
    )?
    .ledgers;
    Ok(loaded)
}

pub trait Parser {
    fn parse(name: &str, content: String) -> Result<ParsedAccount>;
}

#[derive(Debug)]
pub struct ParsedAccount {
    ledgers: Vec<Ledger>,
}

#[derive(Debug)]
pub struct Ledger {
    pub name: String,
    pub symbol: Symbol,
    pub records: Vec<LedgerRecord>,
    pub kind: LedgerKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExtendedLedger {
    pub name: String,
    pub symbol: Symbol,
    pub records: Vec<ExtendedLedgerRecord>,
    pub kind: LedgerKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum LedgerKind {
    Bank,
    Stock,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LedgerRecord {
    pub date: NaiveDate,
    pub amount: f64,
    pub description: String,
    pub category: String,
}

impl From<StockLedgerRecord> for LedgerRecord {
    fn from(
        StockLedgerRecord {
            date,
            amount,
            description,
            category,
            ..
        }: StockLedgerRecord,
    ) -> Self {
        Self {
            date,
            amount,
            description,
            category,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StockLedgerRecord {
    pub date: NaiveDate,
    pub amount: f64,
    pub price: f64,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize, TS)]
#[ts(export)]
pub struct ExtendedLedgerRecord {
    #[ts(type = "number")]
    pub date: NaiveDate,
    pub amount: f64,
    pub description: String,
    pub original_description: String,
    pub category: String,
    pub original_category: String,
    pub comments: String,
    pub checked: bool,
}
