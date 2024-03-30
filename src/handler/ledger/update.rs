use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use polars::prelude::*;
use polars_plan::dsl::col;
use serde::{Deserialize, Serialize};

use crate::{
    banks,
    cli::Format,
    error::AppError,
    portfolio::{Account, NamedDataFrame},
    state::PortfolioState,
};

#[debug_handler]
pub async fn handler(
    State(state): State<PortfolioState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateLedgerRequest>,
) -> Result<Json<UpdateLedgerResponse>, AppError> {
    let incoming = banks::parse(payload.transactions_data, payload.format)?
        .group_by([col("Date"), col("Description"), col("Category")])
        .agg([
            col("Amount").sum(),
            col("*").exclude(["Amount"]),
            col("Date").count().alias("transactions"),
        ])
        .with_columns([
            col("Description").alias("description"),
            col("Category").alias("category"),
            lit("").alias("comments"),
            lit(false).alias("checked"),
        ]);

    let mut guard = state.lock().await;

    if let Some(account) = guard.accounts.iter_mut().find(|a| a.id == id) {
        account.transactions.df = concat(
            [account.transactions.df.clone().lazy(), incoming],
            UnionArgs::default(),
        )?
        .unique(
            Some(vec![
                "Date".to_string(),
                "Amount".to_string(),
                "Category".to_string(),
                "Description".to_string(),
            ]),
            UniqueKeepStrategy::First,
        )
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
            col("description"),
            col("category"),
            col("comments"),
            col("checked"),
            col("transactions"),
        ])
        .collect()?;
    }

    guard.to_file()?;
    let account = guard.accounts.last().unwrap();

    Ok(Json(UpdateLedgerResponse {
        account: Account {
            id: account.id.clone(),
            name: account.name.clone(),
            currency: account.currency.clone(),
            transactions: NamedDataFrame::from(&account.transactions),
        },
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateLedgerRequest {
    pub transactions_data: String,
    pub format: Format,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateLedgerResponse {
    pub account: Account<NamedDataFrame>,
}
