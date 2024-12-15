use std::{collections::HashMap, ops::Deref, path::Path};

use anyhow::Result;
use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serialize};
use time::macros::format_description;
use ts_rs::TS;

use crate::{banks::ExtendedLedger, cli::BankFormat, fx::Currency};

#[derive(Debug, Serialize, Deserialize)]
pub struct Portfolio {
    pub base_currency: Currency,
    pub stocks: Vec<Stock>,
    pub accounts: HashMap<String, Account>,
    pub owner: Owner,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerdePortfolio {
    pub base_currency: Currency,
    #[serde(default)]
    pub stocks: Vec<Stock>,
    #[serde(default)]
    pub accounts: HashMap<String, SerdeAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stock {
    pub symbol: String,
    pub shares: f64,
    pub cost_basis: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerdeAccount {
    pub id: String,
    /// The OIDC owner
    pub owner: Owner,
    pub name: String,
    pub currency: Currency,
    pub format: BankFormat,
    pub initial_balance: Option<f64>,
    pub initial_date: Option<NaiveDate>,
    pub spending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Account {
    pub id: String,
    /// The OIDC owner
    pub owner: Owner,
    pub name: String,
    pub currency: Currency,
    pub format: BankFormat,
    pub ledgers: Vec<ExtendedLedger>,
    pub initial_balance: Option<f64>,
    #[ts(type = "number")]
    pub initial_date: Option<NaiveDate>,
    pub spending: bool,
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

#[derive(
    Debug, Default, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, TS,
)]
#[ts(export)]
#[serde(transparent)]
pub struct Owner(String);

impl Owner {
    pub fn new(subject: String) -> Self {
        Owner(subject)
    }
}

impl Deref for Owner {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<Path> for Owner {
    fn as_ref(&self) -> &Path {
        Path::new(&self.0)
    }
}
