use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};

use crate::{
    error::AppError,
    state::{PortfolioAdapter, PortfolioState},
};

#[debug_handler]
pub async fn handler(
    State((adapter, state)): State<(PortfolioAdapter, PortfolioState)>,
    Path(id): Path<String>,
) -> Result<Json<()>, AppError> {
    adapter.delete_ledger(state, &id).await?;

    Ok(Json(()))
}
