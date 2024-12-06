use axum::{debug_handler, extract::State, Json};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::fx::Currency;
use crate::state::{PortfolioAdapter, PortfolioState};
use crate::{cli::BankFormat, error::AppError};

#[debug_handler]
pub async fn handler(
    State((adapter, state)): State<(PortfolioAdapter, PortfolioState)>,
    Json(payload): Json<CreateLedgerRequest>,
) -> Result<Json<CreateLedgerResponse>, AppError> {
    let id = adapter.create_ledger(state, payload).await?;
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
    pub currency: Currency,
    pub spending: bool,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateLedgerResponse {
    id: String,
}
