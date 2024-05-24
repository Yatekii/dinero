use std::sync::Arc;

use anyhow::Result;
use axum::extract::FromRef;
use tokio::sync::Mutex;

use crate::{
    fx::HistoryCache,
    realms::portfolio::{self, adapter::Adapter, state::Portfolio},
};

// the application state
#[derive(Clone)]
pub struct AppState {
    pub cache: CacheState,
    pub portfolio_adapter: PortfolioAdapter,
    pub portfolio: PortfolioState,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let adapter = portfolio::adapter::Production::new("portfolio/".into());
        let portfolio = adapter.load()?;
        Ok(Self {
            cache: Arc::new(Mutex::new(HistoryCache::load().unwrap())),
            portfolio_adapter: Arc::new(adapter),
            portfolio: Arc::new(Mutex::new(portfolio)),
        })
    }
}

pub type CacheState = Arc<Mutex<HistoryCache>>;

impl FromRef<AppState> for CacheState {
    fn from_ref(app_state: &AppState) -> CacheState {
        app_state.cache.clone()
    }
}

pub type PortfolioState = Arc<Mutex<Portfolio>>;

impl FromRef<AppState> for PortfolioState {
    fn from_ref(app_state: &AppState) -> PortfolioState {
        app_state.portfolio.clone()
    }
}

pub type PortfolioAdapter = Arc<dyn portfolio::adapter::Adapter>;

impl FromRef<AppState> for PortfolioAdapter {
    fn from_ref(app_state: &AppState) -> PortfolioAdapter {
        app_state.portfolio_adapter.clone()
    }
}

impl FromRef<AppState> for (PortfolioAdapter, PortfolioState) {
    fn from_ref(app_state: &AppState) -> (PortfolioAdapter, PortfolioState) {
        (
            app_state.portfolio_adapter.clone(),
            app_state.portfolio.clone(),
        )
    }
}
