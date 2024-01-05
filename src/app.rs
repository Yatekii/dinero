use std::sync::Arc;

use axum::{extract::State, response::Html, Json};
use polars::lazy::frame::IntoLazy;
use polars_plan::dsl::col;
use serde::{Deserialize, Serialize};

use crate::{error::AppError, portfolio::Portfolio};

pub async fn index_handler() -> Html<String> {
    Html(std::fs::read_to_string("frontend/index.html").unwrap())
}

pub async fn data_handler(State(state): State<Arc<Portfolio>>) -> Result<Json<Data>, AppError> {
    let mut df = state.accounts[0]
        .transactions
        .df
        .clone()
        .lazy()
        .select(&[col("Date"), col("Balance").alias(&state.accounts[0].name)]);
    for account in &state.accounts {
        df = df.left_join(
            account
                .transactions
                .df
                .clone()
                .lazy()
                .select(&[col("Date"), col("Balance").alias(&account.name)]),
            col("Date"),
            col("Date"),
        );
    }
    let df = df.collect()?;

    let timestamps = df.column("Date")?.date()?.to_vec();

    let mut balances = Vec::new();
    for account in df.get_columns() {
        if account.name() != "Date" {
            balances.push(Account {
                name: account.name().into(),
                currency: "CHF".into(),
                series: account.f64()?.to_vec(),
            });
        }
    }

    Ok(Json(Data {
        balances,
        timestamps,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub currency: String,
    pub series: Vec<Option<f64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub balances: Vec<Account>,
    pub timestamps: Vec<Option<i32>>,
}
