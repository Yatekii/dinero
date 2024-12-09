use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};

use crate::{error::AppError, handler::auth::user::User, state::PortfolioAdapter};

#[debug_handler(state = crate::state::AppState)]
pub async fn handler(
    State(adapter): State<PortfolioAdapter>,
    Path(id): Path<String>,
    user: User,
) -> Result<Json<()>, AppError> {
    let portfolio = user.portfolio(adapter.clone())?;
    adapter.delete_ledger(portfolio, &id).await?;

    Ok(Json(()))
}
