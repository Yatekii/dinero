use std::{fs::File, path::Path};

use anyhow::Result;
use polars::{
    frame::DataFrame,
    io::{
        parquet::{ParquetReader, ParquetWriter},
        SerReader,
    },
};
use serde::{
    de::{DeserializeOwned, Visitor},
    Deserialize, Deserializer, Serialize,
};
use time::macros::format_description;

#[derive(Debug, Serialize, Deserialize)]
pub struct Portfolio<T: std::fmt::Debug> {
    pub stocks: Vec<Stock>,
    pub accounts: Vec<Account<T>>,
}

impl<T: std::fmt::Debug> Default for Portfolio<T> {
    fn default() -> Self {
        Self {
            stocks: Default::default(),
            accounts: Default::default(),
        }
    }
}

impl<T: std::fmt::Debug + Serialize + DeserializeOwned + Into<StoredDataFrame>> Portfolio<T> {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Portfolio<T>> {
        let file = File::open(path)?;
        let portfolio = serde_yaml::from_reader(file)?;
        Ok(portfolio)
    }
}

impl<'a, T: std::fmt::Debug + Serialize + DeserializeOwned + Into<StoredDataFrame> + 'a>
    Portfolio<T>
where
    StoredDataFrame: From<&'a T>,
{
    pub fn to_file(&'a self) -> Result<()> {
        let accounts = self
            .accounts
            .iter()
            .map(|a| Account {
                id: a.id.clone(),
                name: a.name.clone(),
                currency: a.currency.clone(),
                transactions: StoredDataFrame::from(&a.transactions),
            })
            .collect::<Vec<Account<StoredDataFrame>>>();
        let portfolio = Portfolio {
            stocks: self.stocks.clone(),
            accounts,
        };
        serde_yaml::to_writer(
            std::fs::File::create("portfolio/portfolio.yaml")?,
            &portfolio,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stock {
    pub symbol: String,
    pub shares: f64,
    pub cost_basis: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account<T: std::fmt::Debug> {
    pub id: String,
    pub name: String,
    pub currency: String,
    pub transactions: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(deserialize_with = "parse_date")]
    pub date: time::Date,
    #[serde(default)]
    pub description: String,
    pub amount: f64,
    pub balance: f64,
    #[serde(default)]
    pub action: Action,
}

fn parse_date<'de, D>(deserializer: D) -> Result<time::Date, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    time::Date::parse(&s, &format_description!("[year]-[month]-[day]"))
        .map_err(serde::de::Error::custom)
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum Action {
    #[default]
    Update,
    Interest,
    Fee,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamedDataFrame {
    pub path: String,
    pub df: DataFrame,
}

impl From<&StoredDataFrame> for NamedDataFrame {
    fn from(value: &StoredDataFrame) -> Self {
        Self {
            path: value.path.clone(),
            df: value.df.clone(),
        }
    }
}

#[derive(Debug, Default)]
pub struct StoredDataFrame {
    pub path: String,
    pub df: DataFrame,
}

impl From<&NamedDataFrame> for StoredDataFrame {
    fn from(value: &NamedDataFrame) -> Self {
        Self {
            path: value.path.clone(),
            df: value.df.clone(),
        }
    }
}
impl From<&StoredDataFrame> for StoredDataFrame {
    fn from(value: &StoredDataFrame) -> Self {
        Self {
            path: value.path.clone(),
            df: value.df.clone(),
        }
    }
}

impl<'de> Deserialize<'de> for StoredDataFrame {
    fn deserialize<D>(deserializer: D) -> Result<StoredDataFrame, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DFVisitor;

        impl<'de> Visitor<'de> for DFVisitor {
            type Value = String;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a path to a valid parquet file")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(value.to_string())
            }
        }

        let path = deserializer.deserialize_string(DFVisitor)?;

        let df = ParquetReader::new(std::fs::File::open(&path).unwrap())
            .finish()
            .map_err(serde::de::Error::custom)?;

        Ok(StoredDataFrame { path, df })
    }
}

impl Serialize for StoredDataFrame {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut file = std::fs::File::create(&self.path).map_err(serde::ser::Error::custom)?;
        let mut df = self.df.clone();
        ParquetWriter::new(&mut file)
            .finish(&mut df)
            .map_err(serde::ser::Error::custom)?;

        serializer.serialize_str(&self.path)
    }
}
