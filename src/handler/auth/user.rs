use anyhow::Result;
use async_session::{MemoryStore, SessionStore};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    response::{IntoResponse, Redirect, Response},
    RequestPartsExt,
};
use axum_extra::{headers, typed_header::TypedHeaderRejectionReason, TypedHeader};
use reqwest::header;
use serde::{Deserialize, Serialize};

use crate::{
    realms::portfolio::state::{Owner, Portfolio},
    state::PortfolioAdapter,
};

use super::COOKIE_NAME;

// The user data we'll get back from OIDC.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub sub: Owner,
    pub name: String,
}

impl User {
    pub fn portfolio(&self, adapter: PortfolioAdapter) -> Result<Portfolio> {
        adapter.load(self.sub.clone())
    }
}

pub struct AuthRedirect;

impl IntoResponse for AuthRedirect {
    fn into_response(self) -> Response {
        Redirect::temporary("/api/auth/oidc").into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    MemoryStore: FromRef<S>,
    S: Send + Sync,
{
    // If anything goes wrong or no session is found, redirect to the auth page
    type Rejection = AuthRedirect;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Ok(user_id) = std::env::var("USER_ID") {
            return Ok(User {
                sub: Owner::new(user_id),
                name: "Test".into(),
            });
        }

        let store = MemoryStore::from_ref(state);

        let cookies = parts
            .extract::<TypedHeader<headers::Cookie>>()
            .await
            .map_err(|e| match *e.name() {
                header::COOKIE => match e.reason() {
                    TypedHeaderRejectionReason::Missing => AuthRedirect,
                    _ => panic!("unexpected error getting Cookie header(s): {e}"),
                },
                _ => panic!("unexpected error getting cookies: {e}"),
            })?;
        let session_cookie = cookies.get(COOKIE_NAME).ok_or(AuthRedirect)?;

        let session = store
            .load_session(session_cookie.to_string())
            .await
            .unwrap()
            .ok_or(AuthRedirect)?;

        let user = session.get::<User>("user").ok_or(AuthRedirect)?;

        Ok(user)
    }
}
