use anyhow::anyhow;
use axum::{
    debug_handler,
    extract::{Path, Query, State},
    Json,
};
use chrono::NaiveDate;

use crate::{
    banks::ExtendedLedgerRecord, error::AppError, realms::portfolio::state::Account,
    state::PortfolioState,
};

#[debug_handler]
pub async fn handler(
    State(state): State<PortfolioState>,
    Path(id): Path<String>,
    Query(filter): Query<Filter>,
) -> Result<Json<Account>, AppError> {
    let guard = state.lock().await;
    let account = &guard.accounts.get(&id);
    let Some(account) = account else {
        return Err(anyhow!("{id} was not found"))?;
    };

    let mut transactions: Vec<ExtendedLedgerRecord> = account.records.clone();

    if let Some(from) = filter.from {
        let from = NaiveDate::parse_from_str(&from, "%Y-%m-%d")?;
        transactions.retain(|v| v.date >= from);
    }

    if let Some(to) = filter.to {
        let to = NaiveDate::parse_from_str(&to, "%Y-%m-%d")?;
        transactions.retain(|v| v.date <= to);
    }

    let account = Account {
        id: account.id.clone(),
        name: account.name.clone(),
        currency: account.currency.clone(),
        format: account.format,
        records: transactions,
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
