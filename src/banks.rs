mod ibkr;
mod neon;
mod ubs;

use std::path::Path;

use anyhow::{Context, Result};
use chrono::NaiveDate;
use ts_rs::TS;

use crate::cli::BankFormat;

pub fn parse(name: String, content: String, format: BankFormat) -> Result<Vec<LedgerRecord>> {
    Ok(match format {
        BankFormat::Neon => neon::Neon::parse(name, content),
        BankFormat::Ubs => ubs::Ubs::parse(name, content),
        BankFormat::Ibkr => ibkr::Ibkr::parse(name, content),
    }?
    .ledgers[0]
        .records
        .clone())
}

pub fn load(
    name: String,
    path: impl AsRef<Path>,
    format: BankFormat,
) -> anyhow::Result<Vec<LedgerRecord>> {
    match format {
        BankFormat::Neon => load_inner::<neon::Neon>(name, path),
        BankFormat::Ubs => load_inner::<ubs::Ubs>(name, path),
        BankFormat::Ibkr => load_inner::<ibkr::Ibkr>(name, path),
    }
}

fn load_inner<T: Parser>(
    name: String,
    path: impl AsRef<Path>,
) -> anyhow::Result<Vec<LedgerRecord>> {
    let mut records = vec![];
    for dir_entry in std::fs::read_dir(&path).with_context(|| {
        format!(
            "could not read ledger directory {}",
            path.as_ref().display()
        )
    })? {
        let dir_entry = dir_entry?;

        let path = dir_entry.path();
        let loaded = T::parse(
            name.clone(),
            std::fs::read_to_string(&path)
                .with_context(|| format!("could not read ledger CSV {}", path.display()))?,
        )?
        .ledgers[0]
            .records
            .clone();

        records.extend(loaded);
    }
    Ok(records)
}

pub trait Parser {
    fn parse(name: String, content: String) -> Result<ParsedAccount>;
}

#[derive(Debug)]
pub struct ParsedAccount {
    ledgers: Vec<ParsedLedger>,
}

#[derive(Debug)]
pub struct ParsedLedger {
    name: String,
    records: Vec<LedgerRecord>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct LedgerRecord {
    pub date: NaiveDate,
    pub amount: f64,
    pub description: String,
    pub category: String,
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize, TS)]
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
    pub symbol: String,
}
