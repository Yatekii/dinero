use axum::{debug_handler, extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{error::AppError, state::AppState};

#[debug_handler]
pub async fn handler(State(state): State<AppState>) -> Result<Json<LedgersData>, AppError> {
    let guard = state.portfolio.lock().await;
    Ok(Json(LedgersData {
        ledgers: guard
            .accounts
            .iter()
            .map(|v| Ledger {
                id: v.id.clone(),
                name: v.name.clone(),
                currency: v.currency.clone(),
            })
            .collect(),
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LedgersData {
    pub ledgers: Vec<Ledger>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ledger {
    pub name: String,
    pub id: String,
    pub currency: String,
}
