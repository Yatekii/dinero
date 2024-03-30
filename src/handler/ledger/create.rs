use std::time::{SystemTime, UNIX_EPOCH};

use axum::{debug_handler, extract::State, Json};
use polars::lazy::{dsl::UnionArgs, frame::IntoLazy};
use polars::prelude::*;
use polars_plan::dsl::col;
use serde::{Deserialize, Serialize};

use crate::portfolio::{Account, NamedDataFrame};
use crate::{
    banks,
    cli::Format,
    error::AppError,
    portfolio::{self, StoredDataFrame},
    state::PortfolioState,
};

#[debug_handler]
pub async fn handler(
    State(state): State<PortfolioState>,
    Json(payload): Json<CreateLedgerRequest>,
) -> Result<Json<CreateLedgerResponse>, AppError> {
    let incoming = banks::parse(payload.transactions_data, payload.format)?;
    let incoming = incoming.with_columns([
        col("Description").alias("description"),
        col("Category").alias("category"),
        lit("").alias("comments"),
        lit(false).alias("checked"),
    ]);

    let initial_date = 0;
    let initial_description = "Initial Balance";

    let df = if let Some(initial_balance) = payload.initial_balance {
        concat(
            [
                df!(
                    "Date" => [initial_date],
                    "Amount" => [initial_balance],
                    "Description" => [initial_description],
                )?
                .lazy()
                .select(&[
                    col("Date").cast(DataType::Date),
                    col("Amount"),
                    col("Description"),
                    lit("").alias("Category"),
                ]),
                incoming,
            ],
            UnionArgs::default(),
        )?
    } else {
        incoming
    }
    .group_by([col("Date"), col("Description"), col("Category")])
    .agg([
        col("Amount").sum(),
        col("*").exclude(["Amount"]),
        col("Date").count().alias("transactions"),
    ])
    .sort(
        "Date",
        SortOptions {
            descending: false,
            nulls_last: false,
            multithreaded: true,
            maintain_order: true,
        },
    )
    .select(&[
        col("Date"),
        col("Amount"),
        col("Amount").cum_sum(false).alias("balance"),
        col("Description"),
        col("Category"),
        col("Description").alias("description"),
        col("Category").alias("category"),
        lit("").alias("comments"),
        lit(false).alias("checked"),
        col("transactions"),
    ])
    .collect()?;

    let path = format!(
        "portfolio/{}.parquet",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()
    );

    let id = slug::slugify(&payload.name);
    let name = payload.name.clone();
    let currency = payload.currency.clone();

    let mut guard = state.lock().await;
    guard.accounts.push(portfolio::Account {
        id,
        name,
        currency,
        transactions: StoredDataFrame { path, df },
    });
    guard.to_file()?;
    let account = guard.accounts.last().unwrap();
    Ok(Json(CreateLedgerResponse {
        account: Account {
            id: account.id.clone(),
            name: account.name.clone(),
            currency: account.currency.clone(),
            transactions: NamedDataFrame::from(&account.transactions),
        },
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLedgerRequest {
    pub transactions_data: String,
    pub format: Format,
    pub initial_balance: Option<f64>,
    pub name: String,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLedgerResponse {
    pub account: Account<NamedDataFrame>,
}
