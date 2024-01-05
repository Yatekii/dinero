use std::path::PathBuf;

use polars::lazy::frame::{LazyCsvReader, LazyFileListReader};

use crate::portfolio::Account;

pub struct Parser {
    path: PathBuf,
    name: String,
    currency: String,
}

impl Parser {
    pub fn new(path: PathBuf, name: String, currency: String) -> Self {
        Self {
            path,
            name,
            currency,
        }
    }

    pub fn parse(&self) -> anyhow::Result<Account> {
        let q = LazyCsvReader::new(&self.path)
            .has_header(true)
            .with_try_parse_dates(true)
            .with_separator(b';')
            .finish()?;

        let mut df = q.collect()?;

        df = df.drop_many(&[
            "Original amount",
            "Original currency",
            "Category",
            "Description",
            "Exchange rate",
            "Exchange rate date",
            "Subject",
            "Tags",
            "Wise",
            "Spaces",
        ]);

        Ok(Account {
            name: self.name.clone(),
            currency: self.currency.clone(),
            transactions: crate::portfolio::NamedDataFrame {
                path: self.path.to_string_lossy().to_string(),
                df,
            },
        })
    }
}
