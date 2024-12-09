use axum::{debug_handler, extract::State, Json};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{error::AppError, fx::Currency, handler::auth::user::User, state::PortfolioAdapter};

#[debug_handler(state = crate::state::AppState)]
pub async fn handler(
    State(adapter): State<PortfolioAdapter>,
    user: User,
) -> Result<Json<ListLedgerResponse>, AppError> {
    let portfolio = user.portfolio(adapter)?;
    Ok(Json(ListLedgerResponse {
        ledgers: portfolio
            .accounts
            .iter()
            .map(|(id, ledger)| LedgerMeta {
                id: id.clone(),
                name: ledger.name.clone(),
                currency: ledger.currency,
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
    pub currency: Currency,
}
