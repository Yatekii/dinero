use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    cli::BankFormat,
    error::AppError,
    fx::Currency,
    state::{PortfolioAdapter, PortfolioState},
};

#[debug_handler]
pub async fn handler(
    State((adapter, state)): State<(PortfolioAdapter, PortfolioState)>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateLedgerRequest>,
) -> Result<Json<UpdateLedgerResponse>, AppError> {
    let id = adapter.update_ledger(state, id, payload).await?;

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
    pub currency: Currency,
    pub spending: bool,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateLedgerResponse {
    id: String,
}
