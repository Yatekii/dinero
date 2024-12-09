use std::{env, sync::Arc};

use anyhow::{Context, Result};
use async_session::MemoryStore;
use axum::extract::FromRef;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use tokio::sync::Mutex;

use crate::{fx::HistoryCache, realms::portfolio};

// the application state
#[derive(Clone)]
pub struct AppState {
    pub cache: CacheState,
    pub portfolio_adapter: PortfolioAdapter,
    pub oauth_client: BasicClient,
    pub session_store: MemoryStore,
    pub frontend_url: String,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let adapter = portfolio::adapter::Production::new("portfolio/".into());
        let frontend_url = env::var("FRONTEND_URL").context("Missing FRONTEND_URL!")?;
        Ok(Self {
            cache: Arc::new(Mutex::new(HistoryCache::load().unwrap())),
            portfolio_adapter: Arc::new(adapter),
            oauth_client: Self::oauth_client()?,
            session_store: MemoryStore::new(),
            frontend_url,
        })
    }

    fn oauth_client() -> Result<BasicClient> {
        let client_id = env::var("CLIENT_ID").context("Missing CLIENT_ID!")?;
        let client_secret = env::var("CLIENT_SECRET").context("Missing CLIENT_SECRET!")?;
        let redirect_url = env::var("REDIRECT_URL").context("Missing REDIRECT_URL!")?;

        let auth_url = env::var("AUTH_URL").context("Missing AUTH_URL!")?;
        let token_url = env::var("TOKEN_URL").context("Missing TOKEN_URL!")?;

        Ok(BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new(auth_url).context("failed to create new authorization server URL")?,
            Some(TokenUrl::new(token_url).context("failed to create new token endpoint URL")?),
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_url).context("failed to create new redirection URL")?,
        ))
    }
}

pub type CacheState = Arc<Mutex<HistoryCache>>;

impl FromRef<AppState> for CacheState {
    fn from_ref(app_state: &AppState) -> CacheState {
        app_state.cache.clone()
    }
}

pub type PortfolioAdapter = Arc<dyn portfolio::adapter::Adapter>;

impl FromRef<AppState> for PortfolioAdapter {
    fn from_ref(app_state: &AppState) -> PortfolioAdapter {
        app_state.portfolio_adapter.clone()
    }
}

impl FromRef<AppState> for MemoryStore {
    fn from_ref(state: &AppState) -> Self {
        state.session_store.clone()
    }
}

impl FromRef<AppState> for BasicClient {
    fn from_ref(state: &AppState) -> Self {
        state.oauth_client.clone()
    }
}

pub struct FrontendUrl(pub String);
impl FromRef<AppState> for FrontendUrl {
    fn from_ref(state: &AppState) -> Self {
        Self(state.frontend_url.clone())
    }
}
