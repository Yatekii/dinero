use anyhow::Context;
use async_session::{MemoryStore, SessionStore};
use axum::{extract::State, response::Json};
use axum_extra::{headers, TypedHeader};
use serde::Serialize;
use ts_rs::TS;

use crate::error::AppError;

use super::COOKIE_NAME;

#[derive(Serialize, TS)]
#[ts(export)]
pub struct LogoutResponse {
    pub message: String,
}

pub async fn logout(
    State(store): State<MemoryStore>,
    cookies: Option<TypedHeader<headers::Cookie>>,
) -> Result<Json<LogoutResponse>, AppError> {
    // Handle case where no cookies are present
    let cookies = match cookies {
        Some(c) => c,
        None => {
            return Ok(Json(LogoutResponse {
                message: "No active session to logout".to_string(),
            }));
        }
    };

    // Try to get the session cookie
    let cookie = match cookies.get(COOKIE_NAME) {
        Some(c) => c,
        None => {
            return Ok(Json(LogoutResponse {
                message: "No active session to logout".to_string(),
            }));
        }
    };

    // Load and destroy session if it exists
    match store
        .load_session(cookie.to_string())
        .await
        .context("failed to load session")?
    {
        Some(session) => {
            store
                .destroy_session(session)
                .await
                .context("failed to destroy session")?;
            
            Ok(Json(LogoutResponse {
                message: "Successfully logged out".to_string(),
            }))
        }
        None => Ok(Json(LogoutResponse {
            message: "No active session found".to_string(),
        })),
    }
}
