use std::path::PathBuf;

use polars::{
    datatypes::DataType,
    lazy::{
        dsl::{col, lit},
        frame::{LazyCsvReader, LazyFileListReader},
    },
};
use polars_plan::dsl::StrptimeOptions;

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
        let df = LazyCsvReader::new(&self.path)
            .has_header(true)
            .with_try_parse_dates(true)
            .with_separator(b';')
            .finish()?
            .filter(col("Balance").is_not_null())
            .reverse()
            .select(&[
                col("Booking date")
                    .alias("Date")
                    .str()
                    .to_date(StrptimeOptions {
                        format: Some("%d.%m.%Y".to_string()),
                        ..Default::default()
                    })
                    .cast(DataType::Date),
                col("Debit")
                    .alias("Amount")
                    .str()
                    .replace(lit("'"), lit(""), true)
                    .cast(DataType::Float64),
                col("Balance")
                    .str()
                    .replace(lit("'"), lit(""), true)
                    .cast(DataType::Float64),
            ])
            .collect()?;

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
