use axum::{debug_handler, extract::State, Json};
use chrono::NaiveDate;
use polars::lazy::frame::IntoLazy;
use polars::prelude::*;
use polars_plan::dsl::col;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::realms::portfolio::state::Ledger;
use crate::state::{PortfolioAdapter, PortfolioState};
use crate::{banks, cli::Format, error::AppError};

#[debug_handler]
pub async fn handler(
    State((adapter, state)): State<(PortfolioAdapter, PortfolioState)>,
    Json(payload): Json<CreateLedgerRequest>,
) -> Result<Json<CreateLedgerResponse>, AppError> {
    let data = banks::parse(payload.transactions_data, payload.format).unwrap();

    let incoming = data.clone().with_columns([
        col("Description").alias("description"),
        col("Category").alias("category"),
        lit("").alias("comments"),
        lit(false).alias("checked"),
    ]);

    let df = if let Some(initial_balance) = payload.initial_balance {
        let initial_date = 0;
        let initial_description = "Initial Balance";
        let initial_category = "initial";
        let initial = df!(
            "Date" => [initial_date],
            "Amount" => [initial_balance],
            "Description" => [initial_description],
            "Category" => [initial_category],
        )?
        .lazy()
        .select(&[
            col("Date").cast(DataType::Date),
            col("Amount"),
            col("Description"),
            col("Category"),
        ]);

        concat([initial, incoming], UnionArgs::default())?
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
        ["Date"],
        SortMultipleOptions {
            descending: vec![false],
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

    let id = slug::slugify(format!("{}-{}", &payload.name, &payload.currency));
    let name = payload.name.clone();
    let currency = payload.currency.clone();
    let format = payload.format;

    let mut guard = state.lock().await;
    guard.accounts.insert(
        id.clone(),
        Ledger {
            id: id.clone(),
            name,
            currency,
            format,
            transactions: df,
            initial_balance: payload.initial_balance,
            initial_date: payload.initial_date,
        },
    );
    adapter.store(&guard)?;
    let account = guard.accounts.get(&id).unwrap();
    Ok(Json(CreateLedgerResponse {
        account: Ledger {
            id: account.id.clone(),
            name: account.name.clone(),
            currency: account.currency.clone(),
            format,
            transactions: account.transactions.clone(),
            initial_balance: account.initial_balance,
            initial_date: account.initial_date,
        },
    }))
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateLedgerRequest {
    pub transactions_data: String,
    pub format: Format,
    pub initial_balance: Option<f64>,
    #[ts(type = "number")]
    pub initial_date: Option<NaiveDate>,
    pub name: String,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateLedgerResponse {
    pub account: Ledger,
}
