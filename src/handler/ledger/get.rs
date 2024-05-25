use anyhow::anyhow;
use axum::{
    debug_handler,
    extract::{Path, Query, State},
    Json,
};
use chrono::NaiveDate;
use polars::lazy::frame::IntoLazy;
use polars_plan::dsl::{col, lit};

use crate::{error::AppError, realms::portfolio::state::Ledger, state::PortfolioState};

#[debug_handler]
pub async fn handler(
    State(state): State<PortfolioState>,
    Path(id): Path<String>,
    Query(filter): Query<Filter>,
) -> Result<Json<Ledger>, AppError> {
    let guard = state.lock().await;
    let account = &guard.accounts.get(&id);
    let Some(account) = account else {
        return Err(anyhow!("{id} was not found"))?;
    };

    let mut transactions = account.transactions.clone().lazy();

    if let Some(from) = filter.from {
        let from = NaiveDate::parse_from_str(&from, "%Y-%m-%d")?;
        transactions = transactions.filter(col("Date").gt_eq(lit(from)))
    }

    if let Some(to) = filter.to {
        let to = NaiveDate::parse_from_str(&to, "%Y-%m-%d")?;
        transactions = transactions.filter(col("Date").lt_eq(lit(to)))
    }

    let account = Ledger {
        id: account.id.clone(),
        name: account.name.clone(),
        currency: account.currency.clone(),
        format: account.format,
        transactions: transactions.collect()?,
        initial_balance: account.initial_balance,
        initial_date: account.initial_date,
        spending: account.spending,
    };

    Ok(Json(account))
}

#[derive(serde::Deserialize)]
pub struct Filter {
    from: Option<String>,
    to: Option<String>,
}
