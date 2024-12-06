use std::collections::HashMap;

use axum::{
    debug_handler,
    extract::{Query, State},
    Json,
};
use chrono::NaiveDate;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{error::AppError, state::PortfolioState};

#[debug_handler]
pub async fn handler(
    date_range: Query<DateRange>,
    State(state): State<PortfolioState>,
) -> Result<Json<LedgerSummary>, AppError> {
    let guard = state.lock().await;
    let mut spending = HashMap::new();
    for (id, account) in &guard.accounts {
        let categories = account.records.clone();

        let categories = if let Some(from) = &date_range.from {
            let from =
                NaiveDate::from_num_days_from_ce_opt((*from / 1000 / 60 / 60 / 24) as i32).unwrap();
            categories.into_iter().filter(|v| v.date >= from).collect()
        } else {
            categories
        };

        let categories = if let Some(to) = &date_range.to {
            let to =
                NaiveDate::from_num_days_from_ce_opt((*to / 1000 / 60 / 60 / 24) as i32).unwrap();
            categories.into_iter().filter(|v| v.date <= to).collect()
        } else {
            categories
        };

        let categories = categories
            .into_iter()
            .group_by(|v| v.category.clone())
            .into_iter()
            .map(|(k, v)| (k.clone(), v.into_iter().map(|v| v.amount).sum()))
            .collect();

        spending.insert(id.clone(), SpendingSummary { categories });
    }

    Ok(Json(LedgerSummary { spending }))
}

#[derive(Deserialize, Debug)]
pub struct DateRange {
    from: Option<u64>,
    to: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LedgerSummary {
    spending: HashMap<String, SpendingSummary>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SpendingSummary {
    categories: HashMap<String, f64>,
}
