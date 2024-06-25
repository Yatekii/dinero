use std::collections::HashMap;

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
            spend_per_month: SpendPerMonth {
                months: HashMap::new(),
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

        max_date = max_date.max(max);
        min_date = min_date.min(min);
    }

    let mut df = DataFrame::default();
    df.with_column(
        Series::from_vec("Date", (min_date..max_date + 1).collect::<Vec<_>>())
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

    let mut data = HashMap::new();
    for ledger in portfolio.accounts.values().filter(|a| a.spending) {
        let transactions = ledger.transactions.clone().lazy().select(&[
            col("Date").sort(SortOptions::default().with_maintain_order(true)),
            col("Amount"),
            col("Category"),
        ]);

        let transactions = if ledger.currency != "CHF" {
            transactions
                .left_join(
                    fetch_rate(&state, ledger)
                        .await?
                        .select([col("Date"), col("Close")])
                        .lazy(),
                    col("Date"),
                    col("Date"),
                )
                .select([
                    col("*").exclude(["Amount", "Close"]),
                    col("Amount") * col("Close"),
                ])
        } else {
            transactions.select([col("Amount"), col("Date"), col("Category")])
        }
        .filter(col("Amount").lt(0))
        .group_by([
            col("Date").dt().year().alias("year"),
            col("Date").dt().month().alias("month"),
            col("Category"),
        ])
        .agg([col("Amount").sum()])
        .collect()?;

        let year = transactions.column("year")?.i32()?;
        let month = transactions.column("month")?.i8()?;
        let amount = transactions.column("Amount")?.f64()?;
        let category = transactions.column("Category")?.str()?;

        for (((year, month), amount), category) in year
            .iter()
            .zip(month.iter())
            .zip(amount.iter())
            .zip(category.iter())
        {
            let amount = -amount.unwrap();
            let months = data.entry(month.unwrap()).or_insert(HashMap::new());
            let categories = months.entry(year.unwrap()).or_insert(HashMap::new());
            let total = categories
                .entry(category.unwrap().to_string())
                .or_insert(0.0);
            *total += amount;
        }
    }

    Ok(Json(PortfolioSummaryResponse {
        total_balance: PortfolioLedgersData {
            balances,
            timestamps,
        },
        spend_per_month: SpendPerMonth { months: data },
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
pub struct SpendPerMonth {
    pub months: HashMap<i8, HashMap<i32, HashMap<String, f64>>>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PortfolioSummaryResponse {
    pub total_balance: PortfolioLedgersData,
    pub spend_per_month: SpendPerMonth,
}

async fn fetch_rate(state: &AppState, ledger: &Ledger) -> Result<LazyFrame, AppError> {
    let mut cache = state.cache.lock().await;
    let rate = cache.get(&ledger.currency, "CHF").await?;
    Ok(rate.rates.clone().lazy())
}
