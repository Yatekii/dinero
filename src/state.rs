use std::sync::Arc;

use axum::extract::FromRef;
use tokio::sync::Mutex;

use crate::{
    fx::HistoryCache,
    portfolio::{Portfolio, StoredDataFrame},
};

// the application state
#[derive(Clone)]
pub struct AppState {
    pub portfolio: PortfolioState,
    pub cache: Arc<Mutex<HistoryCache>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            portfolio: Arc::new(Mutex::new(
                Portfolio::from_file("portfolio/portfolio.yaml").unwrap(),
            )),
            cache: Arc::new(Mutex::new(HistoryCache::load().unwrap())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

pub type PortfolioState = Arc<Mutex<Portfolio<StoredDataFrame>>>;

impl FromRef<AppState> for PortfolioState {
    fn from_ref(app_state: &AppState) -> PortfolioState {
        app_state.portfolio.clone()
    }
}
