use std::collections::HashMap;

use axum::{
    debug_handler,
    extract::{Query, State},
    Json,
};
use polars::lazy::frame::IntoLazy;
use polars_plan::{dsl::col, logical_plan::lit};
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
        let categories = account.transactions.clone().lazy();

        let categories = if let Some(from) = &date_range.from {
            categories.filter(col("Date").gt_eq(lit(*from / 1000 / 60 / 60 / 24)))
        } else {
            categories
        };

        let categories = if let Some(to) = &date_range.to {
            categories.filter(col("Date").lt_eq(lit(*to / 1000 / 60 / 60 / 24)))
        } else {
            categories
        };

        let categories = categories
            .group_by([col("category")])
            .agg([col("Amount").sum().alias("total")])
            .collect()
            .unwrap();

        let columns = categories.columns(["category", "total"])?;

        let categories: HashMap<_, _> = columns[0]
            .str()
            .unwrap()
            .into_iter()
            .zip(columns[1].f64().unwrap().into_iter())
            .map(|(c, t)| (c.unwrap().to_string(), t.unwrap()))
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
pub struct SpendingSummary {
    categories: HashMap<String, f64>,
}
