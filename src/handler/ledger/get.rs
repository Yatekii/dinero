use anyhow::anyhow;
use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};

use crate::{
    error::AppError,
    portfolio::{Account, NamedDataFrame},
    state::AppState,
};

#[debug_handler]
pub async fn handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Account<NamedDataFrame>>, AppError> {
    let guard = state.portfolio.lock().await;
    let account = &guard.accounts.iter().find(|a| a.id == id);
    let Some(account) = account else {
        return Err(anyhow!("{id} was not found"))?;
    };
    let account = Account {
        id: account.id.clone(),
        name: account.name.clone(),
        currency: account.currency.clone(),
        transactions: NamedDataFrame::from(&account.transactions),
    };

    Ok(Json(account))
}
