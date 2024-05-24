use anyhow::anyhow;
use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};

use crate::{error::AppError, realms::portfolio::state::Ledger, state::PortfolioState};

#[debug_handler]
pub async fn handler(
    State(state): State<PortfolioState>,
    Path(id): Path<String>,
) -> Result<Json<Ledger>, AppError> {
    let guard = state.lock().await;
    let account = &guard.accounts.get(&id);
    let Some(account) = account else {
        return Err(anyhow!("{id} was not found"))?;
    };
    let account = Ledger {
        id: account.id.clone(),
        name: account.name.clone(),
        currency: account.currency.clone(),
        format: account.format,
        transactions: account.transactions.clone(),
        initial_balance: account.initial_balance,
        initial_date: account.initial_date,
    };

    Ok(Json(account))
}
