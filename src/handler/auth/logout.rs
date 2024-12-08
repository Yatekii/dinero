use anyhow::Context;
use async_session::{MemoryStore, SessionStore};
use axum::{extract::State, response::Redirect};
use axum_extra::{headers, TypedHeader};

use crate::{error::AppError, state::FrontendUrl};

use super::COOKIE_NAME;

pub async fn logout(
    State(store): State<MemoryStore>,
    State(FrontendUrl(frontend_url)): State<FrontendUrl>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
) -> Result<Redirect, AppError> {
    let cookie = cookies
        .get(COOKIE_NAME)
        .context("unexpected error getting cookie name")?;

    let session = match store
        .load_session(cookie.to_string())
        .await
        .context("failed to load session")?
    {
        Some(s) => s,
        // No session active, just redirect
        None => return Ok(Redirect::to(&frontend_url)),
    };

    store
        .destroy_session(session)
        .await
        .context("failed to destroy session")?;

    Ok(Redirect::to(&frontend_url))
}
