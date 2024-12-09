use anyhow::{anyhow, Context};
use async_session::{MemoryStore, Session, SessionStore};
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect},
};
use axum_extra::{headers, TypedHeader};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthorizationCode, CsrfToken, TokenResponse,
};
use reqwest::header::SET_COOKIE;
use serde::Deserialize;

use crate::{error::AppError, state::FrontendUrl};

use super::{user::User, COOKIE_NAME, CSRF_TOKEN};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AuthRequest {
    code: String,
    state: String,
}

async fn csrf_token_validation_workflow(
    auth_request: &AuthRequest,
    cookies: &headers::Cookie,
    store: &MemoryStore,
) -> Result<(), AppError> {
    // Extract the cookie from the request
    let cookie = cookies
        .get(COOKIE_NAME)
        .context("unexpected error getting cookie name")?
        .to_string();

    // Load the session
    let session = match store
        .load_session(cookie)
        .await
        .context("failed to load session")?
    {
        Some(session) => session,
        None => return Err(anyhow!("Session not found").into()),
    };

    // Extract the CSRF token from the session
    let stored_csrf_token = session
        .get::<CsrfToken>(CSRF_TOKEN)
        .context("CSRF token not found in session")?
        .to_owned();

    // Cleanup the CSRF token session
    store
        .destroy_session(session)
        .await
        .context("Failed to destroy old session")?;

    // Validate CSRF token is the same as the one in the auth request
    if *stored_csrf_token.secret() != auth_request.state {
        return Err(anyhow!("CSRF token mismatch").into());
    }

    Ok(())
}

pub async fn login_authorized(
    Query(query): Query<AuthRequest>,
    State(store): State<MemoryStore>,
    State(oauth_client): State<BasicClient>,
    State(FrontendUrl(frontend_url)): State<FrontendUrl>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
) -> Result<impl IntoResponse, AppError> {
    csrf_token_validation_workflow(&query, &cookies, &store).await?;

    // Get an auth token
    let token = oauth_client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .request_async(async_http_client)
        .await
        .context("failed in sending request request to authorization server")?;

    // Fetch user data from discord
    let client = reqwest::Client::new();
    let user_data: User = client
        .get("https://zitadel.huesser.dev/oidc/v1/userinfo")
        .bearer_auth(token.access_token().secret())
        .send()
        .await
        .context("failed in sending request to target Url")?
        .json::<User>()
        .await
        .context("failed to deserialize response as JSON")?;

    // Create a new session filled with user data
    let mut session = Session::new();
    session
        .insert("user", &user_data)
        .context("failed in inserting serialized value into session")?;

    // Store session and get corresponding cookie
    let cookie = store
        .store_session(session)
        .await
        .context("failed to store session")?
        .context("unexpected error retrieving cookie value")?;

    // Build the cookie
    let cookie = format!("{COOKIE_NAME}={cookie}; SameSite=Lax; HttpOnly; Secure; Path=/");

    // Set cookie
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        cookie.parse().context("failed to parse cookie")?,
    );

    Ok((
        headers,
        Redirect::to(&format!(
            "{}?userid={}&username={}",
            frontend_url, &*user_data.sub, user_data.name
        )),
    ))
}
