mod ibkr;

use std::path::Path;

use anyhow::{Context, Result};
use polars::{lazy::frame::LazyFrame, prelude::*};

use crate::cli::BrokerFormat;

pub fn parse(content: String, format: BrokerFormat) -> Result<LazyFrame> {
    Ok(match format {
        BrokerFormat::Ibkr => ibkr::Ibkr::parse(content),
    }?
    .ledgers[0]
        .df
        .clone())
}

pub fn load(path: impl AsRef<Path>, format: BrokerFormat) -> anyhow::Result<LazyFrame> {
    match format {
        BrokerFormat::Ibkr => load_inner::<ibkr::Ibkr>(path),
    }
}

fn load_inner<T: Parser>(path: impl AsRef<Path>) -> anyhow::Result<LazyFrame> {
    let mut df = None;
    for dir_entry in std::fs::read_dir(&path).with_context(|| {
        format!(
            "could not read ledger directory {}",
            path.as_ref().display()
        )
    })? {
        let dir_entry = dir_entry?;

        let path = dir_entry.path();
        let loaded = T::parse(
            std::fs::read_to_string(&path)
                .with_context(|| format!("could not read ledger CSV {}", path.display()))?,
        )?
        .ledgers[0]
            .df
            .clone();

        df = Some(if let Some(df) = df {
            concat([df, loaded], UnionArgs::default())?
        } else {
            loaded
        });

        df.clone().unwrap().collect().unwrap().get_column_names();
    }
    Ok(df.expect("a dataframe"))
}

pub trait Parser {
    fn parse(content: String) -> Result<ParsedAccount>;
}

pub struct ParsedAccount {
    ledgers: Vec<ParsedLedger>,
}

pub struct ParsedLedger {
    name: String,
    df: LazyFrame,
}
