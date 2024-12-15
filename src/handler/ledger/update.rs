use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{cli::BankFormat, error::AppError, handler::auth::user::User, state::PortfolioAdapter};

#[debug_handler(state = crate::state::AppState)]
pub async fn handler(
    State(adapter): State<PortfolioAdapter>,
    Path(id): Path<String>,
    user: User,
    Json(payload): Json<UpdateLedgerRequest>,
) -> Result<Json<UpdateLedgerResponse>, AppError> {
    let portfolio = user.portfolio(adapter.clone())?;
    let id = adapter.update_ledger(portfolio, id, payload).await?;

    Ok(Json(UpdateLedgerResponse { id }))
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateLedgerRequest {
    pub format: BankFormat,
    pub initial_balance: Option<f64>,
    #[ts(type = "number")]
    pub initial_date: Option<NaiveDate>,
    pub name: String,
    pub spending: bool,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateLedgerResponse {
    id: String,
}
