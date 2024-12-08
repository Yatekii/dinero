use anyhow::Context;
use async_session::{MemoryStore, Session, SessionStore};
use axum::{extract::State, http::HeaderMap, response::Redirect};
use oauth2::{basic::BasicClient, CsrfToken, Scope};
use reqwest::header::SET_COOKIE;

use crate::error::AppError;

use super::{COOKIE_NAME, CSRF_TOKEN};

pub async fn oidc_auth(
    State(client): State<BasicClient>,
    State(store): State<MemoryStore>,
) -> Result<(HeaderMap, Redirect), AppError> {
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scopes(
            ["openid", "profile"]
                .into_iter()
                .map(|s| Scope::new(s.into())),
        )
        .url();

    // Create session to store csrf_token
    let mut session = Session::new();
    session
        .insert(CSRF_TOKEN, &csrf_token)
        .context("failed in inserting CSRF token into session")?;

    // Store the session in MemoryStore and retrieve the session cookie
    let cookie = store
        .store_session(session)
        .await
        .context("failed to store CSRF token session")?
        .context("unexpected error retrieving CSRF cookie value")?;

    // Attach the session cookie to the response header
    let cookie = format!("{COOKIE_NAME}={cookie}; SameSite=Lax; HttpOnly; Secure; Path=/");
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        cookie.parse().context("failed to parse cookie")?,
    );

    Ok((headers, Redirect::to(auth_url.as_ref())))
}
