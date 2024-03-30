use axum::{debug_handler, extract::State, response::Html, Json};
use polars::lazy::frame::IntoLazy;
use polars_core::chunked_array::ops::FillNullLimit;
use polars_ops::frame::AsofJoin;
use polars_plan::dsl::col;
use serde::{Deserialize, Serialize};

use crate::{error::AppError, state::AppState};

pub async fn index_handler() -> Html<String> {
    Html(std::fs::read_to_string("frontend/index.html").unwrap())
}
#[debug_handler]
pub async fn data_handler(State(state): State<AppState>) -> Result<Json<Data>, AppError> {
    let portfolio = &state.portfolio.lock().await;
    let mut df = portfolio.accounts[0]
        .transactions
        .df
        .clone()
        .lazy()
        .select(&[
            col("Date").sort(false),
            col("Balance").alias(&portfolio.accounts[0].name),
        ])
        .collect()?;

    for account in &portfolio.accounts[1..] {
        let rate = {
            let mut cache = state.cache.lock().await;
            cache.get(&account.currency, "CHF").await?
        };
        df = df.join_asof(
            &account
                .transactions
                .df
                .clone()
                .lazy()
                .select(&[col("Date").sort(false), col("Balance")])
                .left_join(
                    rate.rates
                        .clone()
                        .lazy()
                        .select([col("Date").alias("DateRate"), col("Close")]),
                    col("Date"),
                    col("DateRate"),
                )
                .select([
                    col("*").exclude(["DateRate", "Balance", "Close"]),
                    col("Balance").alias(&format!("{} [{}]", account.name, account.currency))
                        * col("Close"),
                ])
                .collect()?,
            "Date",
            "Date",
            polars_ops::frame::AsofStrategy::Forward,
            None,
            None,
        )?;
    }
    df = df
        .lazy()
        .select(&[col("*").forward_fill(FillNullLimit::None)])
        .collect()?;

    let timestamps = df.column("Date")?.date()?.to_vec();
    let mut balances = Vec::new();
    for (transactions, account) in df
        .get_columns()
        .iter()
        .skip(1)
        .zip(portfolio.accounts.iter())
    {
        balances.push(Account {
            id: account.id.clone(),
            name: account.name.clone(),
            currency: "CHF".into(),
            series: transactions.f64()?.to_vec(),
        });
    }

    Ok(Json(Data {
        balances,
        timestamps,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub currency: String,
    pub series: Vec<Option<f64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub balances: Vec<Account>,
    pub timestamps: Vec<Option<i32>>,
}
