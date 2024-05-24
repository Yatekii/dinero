use axum::{debug_handler, extract::State, Json};
use polars::{
    chunked_array::ops::SortOptions,
    lazy::frame::{IntoLazy, LazyFrame},
};
use polars_core::{
    chunked_array::ops::FillNullLimit, frame::DataFrame, prelude::NamedFromOwned, series::Series,
};
use polars_ops::frame::DataFrameJoinOps;
use polars_plan::dsl::col;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{error::AppError, realms::portfolio::state::Ledger, state::AppState};

#[debug_handler]
pub async fn handler(
    State(state): State<AppState>,
) -> Result<Json<PortfolioSummaryResponse>, AppError> {
    let portfolio = state.portfolio.lock().await;

    if portfolio.accounts.is_empty() {
        return Ok(Json(PortfolioSummaryResponse {
            total_balance: PortfolioLedgersData {
                balances: vec![],
                timestamps: vec![],
            },
        }));
    }

    let mut max_date = 0u64;
    let mut min_date = u64::MAX;
    for ledger in portfolio.accounts.values() {
        let max = ledger
            .transactions
            .clone()
            .lazy()
            .select([col("Date").max()])
            .collect()
            .unwrap()
            .column("Date")
            .unwrap()
            .max::<u64>()
            .unwrap()
            .unwrap();
        let min = ledger
            .transactions
            .clone()
            .lazy()
            .select([col("Date").min()])
            .collect()
            .unwrap()
            .column("Date")
            .unwrap()
            .min::<u64>()
            .unwrap()
            .unwrap();

        dbg!((&ledger.id, min));

        max_date = max_date.max(max);
        min_date = min_date.min(min);
    }

    dbg!((min_date, max_date));

    let mut df = DataFrame::default();
    df.with_column(
        Series::from_vec("Date", (min_date..max_date).collect::<Vec<_>>())
            .cast(&polars_core::datatypes::DataType::Date)
            .unwrap(),
    )
    .unwrap();
    let mut df = df.lazy().select([col("Date")]).collect().unwrap();

    for ledger in portfolio.accounts.values() {
        let transactions = ledger.transactions.clone().lazy().select(&[
            col("Date").sort(SortOptions::default().with_maintain_order(true)),
            col("balance"),
        ]);

        let transactions = if ledger.currency != "CHF" {
            transactions
                .left_join(
                    fetch_rate(&state, ledger)
                        .await?
                        .clone()
                        .select([col("Date"), col("Close")])
                        .lazy(),
                    col("Date"),
                    col("Date"),
                )
                .select([
                    col("*").exclude(["balance", "Close"]),
                    col("balance").alias(&format!("{} [{}]", ledger.name, ledger.currency))
                        * col("Close"),
                ])
        } else {
            transactions.select([
                col("*").exclude(["balance"]),
                col("balance").alias(&format!("{} [{}]", ledger.name, ledger.currency)),
            ])
        }
        .collect()?;

        df = df.left_join(&transactions, ["Date"], ["Date"])?;
    }
    df = df
        .lazy()
        .select(&[col("*").forward_fill(FillNullLimit::None)])
        .collect()?;

    let timestamps = df.column("Date")?.date()?.to_vec();
    let mut balances = Vec::new();
    for (transactions, (id, ledger)) in df
        .get_columns()
        .iter()
        .skip(1)
        .zip(portfolio.accounts.iter())
    {
        balances.push(PortfolioLedgerData {
            id: id.clone(),
            name: ledger.name.clone(),
            currency: "CHF".into(),
            series: transactions.f64()?.to_vec(),
        });
    }

    Ok(Json(PortfolioSummaryResponse {
        total_balance: PortfolioLedgersData {
            balances,
            timestamps,
        },
    }))
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PortfolioLedgerData {
    pub id: String,
    pub name: String,
    pub currency: String,
    pub series: Vec<Option<f64>>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PortfolioLedgersData {
    pub balances: Vec<PortfolioLedgerData>,
    pub timestamps: Vec<Option<i32>>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PortfolioSummaryResponse {
    pub total_balance: PortfolioLedgersData,
}

async fn fetch_rate(state: &AppState, ledger: &Ledger) -> Result<LazyFrame, AppError> {
    let mut cache = state.cache.lock().await;
    let rate = cache.get(&ledger.currency, "CHF").await?;
    Ok(rate.rates.clone().lazy())
}
