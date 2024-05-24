use std::{collections::HashMap, fs::File, path::PathBuf};

use anyhow::Result;
use polars::io::parquet::write::ParquetWriter;

use crate::{banks::load, processing::process};

use super::state::{Ledger, Portfolio, SerdeLedger, SerdePortfolio};

pub trait Adapter: Send + Sync {
    fn load(&self) -> Result<Portfolio>;
    fn store(&self, state: &Portfolio) -> Result<()>;
}

pub struct Production {
    path: PathBuf,
}

impl Production {
    const PORTFOLIO_FILE_NAME: &'static str = "portfolio.yaml";
    const PORTFOLIO_LEDGER_DIR: &'static str = "ledgers";

    pub(crate) fn new(path: PathBuf) -> Production {
        Production { path }
    }
}

impl Adapter for Production {
    fn store(&self, portfolio: &Portfolio) -> Result<()> {
        let accounts = portfolio
            .accounts
            .iter()
            .map(|(id, ledger)| {
                (
                    id.clone(),
                    SerdeLedger {
                        id: ledger.id.clone(),
                        name: ledger.name.clone(),
                        currency: ledger.currency.clone(),
                        format: ledger.format,
                        initial_balance: ledger.initial_balance,
                        initial_date: ledger.initial_date,
                    },
                )
            })
            .collect::<HashMap<String, SerdeLedger>>();
        serde_yaml::to_writer(
            std::fs::File::create("portfolio/portfolio.yaml")?,
            &SerdePortfolio {
                stocks: portfolio.stocks.clone(),
                accounts,
            },
        )?;
        let path = self.path.join(Self::PORTFOLIO_LEDGER_DIR);
        std::fs::create_dir_all(&path)?;
        for (id, ledger) in &portfolio.accounts {
            let mut file = std::fs::File::create(path.join(format!("{}.parquet", id)))?;
            let mut df = ledger.transactions.clone();
            ParquetWriter::new(&mut file).finish(&mut df)?;
        }
        Ok(())
    }

    fn load(&self) -> Result<Portfolio> {
        let file = File::open(self.path.join(Self::PORTFOLIO_FILE_NAME))?;
        let portfolio: SerdePortfolio = serde_yaml::from_reader(file)?;

        let path = self.path.join(Self::PORTFOLIO_LEDGER_DIR);
        let accounts = portfolio
            .accounts
            .into_iter()
            .map(|(id, ledger)| {
                let path = path.join(ledger.id);
                Ok((
                    id.clone(),
                    Ledger {
                        id,
                        name: ledger.name,
                        currency: ledger.currency,
                        format: ledger.format,
                        transactions: process(
                            load(path, ledger.format)?,
                            ledger.initial_balance,
                            ledger.initial_date,
                        )?,
                        initial_balance: ledger.initial_balance,
                        initial_date: ledger.initial_date,
                    },
                ))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        Ok(Portfolio {
            stocks: vec![],
            accounts,
        })
    }
}

pub struct Test;

impl Adapter for Test {
    fn load(&self) -> Result<Portfolio> {
        Ok(Portfolio::default())
    }
    fn store(&self, _state: &Portfolio) -> Result<()> {
        Ok(())
    }
}
