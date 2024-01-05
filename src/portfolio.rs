use std::{fs::File, path::Path};

use anyhow::Result;
use polars::{
    frame::DataFrame,
    io::{
        parquet::{ParquetReader, ParquetWriter},
        SerReader,
    },
};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use time::macros::format_description;

#[derive(Debug, Serialize, Deserialize)]
pub struct Portfolio {
    pub stocks: Vec<Stock>,
    pub accounts: Vec<Account>,
}

impl Portfolio {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Portfolio> {
        let file = File::open(path)?;
        let portfolio = serde_yaml::from_reader(file)?;
        Ok(portfolio)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stock {
    pub symbol: String,
    pub shares: f64,
    pub cost_basis: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub currency: String,
    pub transactions: NamedDataFrame,
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

#[derive(Debug)]
pub struct NamedDataFrame {
    pub path: String,
    pub df: DataFrame,
}

impl<'de> Deserialize<'de> for NamedDataFrame {
    fn deserialize<D>(deserializer: D) -> Result<NamedDataFrame, D::Error>
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

        Ok(NamedDataFrame { path, df })
    }
}

impl Serialize for NamedDataFrame {
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
