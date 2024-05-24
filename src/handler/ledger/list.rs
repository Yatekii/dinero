use axum::{debug_handler, extract::State, Json};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{error::AppError, state::PortfolioState};

#[debug_handler]
pub async fn handler(
    State(state): State<PortfolioState>,
) -> Result<Json<ListLedgerResponse>, AppError> {
    let guard = state.lock().await;
    Ok(Json(ListLedgerResponse {
        ledgers: guard
            .accounts
            .iter()
            .map(|(id, ledger)| LedgerMeta {
                id: id.clone(),
                name: ledger.name.clone(),
                currency: ledger.currency.clone(),
            })
            .collect(),
    }))
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ListLedgerResponse {
    pub ledgers: Vec<LedgerMeta>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LedgerMeta {
    pub name: String,
    pub id: String,
    pub currency: String,
}
