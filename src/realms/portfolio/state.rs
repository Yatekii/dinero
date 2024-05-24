use std::collections::HashMap;

use anyhow::Result;
use chrono::NaiveDate;
use polars::frame::DataFrame;
use serde::{Deserialize, Deserializer, Serialize};
use time::macros::format_description;
use ts_rs::TS;

use crate::cli::Format;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Portfolio {
    pub stocks: Vec<Stock>,
    pub accounts: HashMap<String, Ledger>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SerdePortfolio {
    #[serde(default)]
    pub stocks: Vec<Stock>,
    #[serde(default)]
    pub accounts: HashMap<String, SerdeLedger>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stock {
    pub symbol: String,
    pub shares: f64,
    pub cost_basis: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerdeLedger {
    pub id: String,
    pub name: String,
    pub currency: String,
    pub format: Format,
    pub initial_balance: Option<f64>,
    pub initial_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Ledger {
    pub id: String,
    pub name: String,
    pub currency: String,
    pub format: Format,
    #[ts(type = "{
        columns: { values: number[] }[];
    }")]
    pub transactions: DataFrame,
    pub initial_balance: Option<f64>,
    #[ts(type = "number")]
    pub initial_date: Option<NaiveDate>,
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
