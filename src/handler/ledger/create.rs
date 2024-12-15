use axum::{debug_handler, extract::State, Json};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::handler::auth::user::User;
use crate::state::PortfolioAdapter;
use crate::{cli::BankFormat, error::AppError};

#[debug_handler(state = crate::state::AppState)]
pub async fn handler(
    State(adapter): State<PortfolioAdapter>,
    user: User,
    Json(payload): Json<CreateLedgerRequest>,
) -> Result<Json<CreateLedgerResponse>, AppError> {
    let portfolio = user.portfolio(adapter.clone())?;
    let id = adapter.create_ledger(portfolio, payload).await?;
    Ok(Json(CreateLedgerResponse { id }))
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CreateLedgerRequest {
    pub format: BankFormat,
    pub initial_balance: Option<f64>,
    #[ts(type = "number")]
    pub initial_date: Option<NaiveDate>,
    pub name: String,
    pub spending: bool,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateLedgerResponse {
    id: String,
}
