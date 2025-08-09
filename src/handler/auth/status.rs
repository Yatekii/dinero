use axum::{extract::State, response::Json};
use serde::Serialize;
use ts_rs::TS;

use crate::{error::AppError, state::PortfolioAdapter};

use super::user::User;

#[derive(Serialize, TS)]
#[ts(export)]
pub struct AuthStatus {
    pub authenticated: bool,
    pub user_id: Option<String>,
    pub username: Option<String>,
}

pub async fn auth_status(
    State(_portfolio_adapter): State<PortfolioAdapter>,
    user: Result<User, super::user::AuthRedirect>,
) -> Result<Json<AuthStatus>, AppError> {
    match user {
        Ok(user) => Ok(Json(AuthStatus {
            authenticated: true,
            user_id: Some(user.sub.to_string()),
            username: Some(user.name),
        })),
        Err(_) => Ok(Json(AuthStatus {
            authenticated: false,
            user_id: None,
            username: None,
        })),
    }
}