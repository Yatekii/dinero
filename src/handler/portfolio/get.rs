use std::collections::{BTreeMap, HashMap};

use axum::{debug_handler, extract::State, Json};
use chrono::{Datelike, Days, NaiveDate, NaiveTime};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{error::AppError, fx::Currency, realms::portfolio::state::Account, state::AppState};

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

    let mut max_date = NaiveDate::MIN;
    let mut min_date = NaiveDate::MAX;
    for ledger in portfolio.accounts.values() {
        let max = ledger.records.iter().map(|v| v.date).max().unwrap();
        let min = ledger.records.iter().map(|v| v.date).min().unwrap();

        max_date = max_date.max(max);
        min_date = min_date.min(min);
    }

    let dates = (min_date.iter_days().take_while(|d| d <= &max_date)).collect::<Vec<_>>();

    let mut ledgers = HashMap::new();
    for ledger in portfolio.accounts.values() {
        let rates = if ledger.currency != portfolio.base_currency {
            fetch_rate(&state, ledger, portfolio.base_currency).await?
        } else {
            BTreeMap::new()
        };
        let mut balances = Vec::with_capacity(ledger.records.len());
        let mut total = 0.0;
        for date in &dates {
            let sum = ledger
                .records
                .iter()
                .filter(|v| &v.date == date)
                .map(|v| v.amount)
                .sum::<f64>();

            let rate = if ledger.currency != portfolio.base_currency {
                if sum > 0.0 {
                    *rates
                        .get(date)
                        .or_else(|| rates.get(&date.checked_sub_days(Days::new(1)).unwrap()))
                        .or_else(|| rates.get(&date.checked_sub_days(Days::new(2)).unwrap()))
                        .unwrap()
                } else {
                    1.0
                }
            } else {
                1.0
            };

            total += sum;
            let amount = total * rate;

            balances.push(amount);
        }

        ledgers.insert(
            ledger.id.clone(),
            (ledger.name.clone(), ledger.currency, balances),
        );
    }

    let mut balances = Vec::new();
    let mut total = ledgers
        .iter()
        .next()
        .map(|v| v.1 .2.clone())
        .unwrap_or_default();
    for (i, (id, (name, currency, transactions))) in ledgers.into_iter().enumerate() {
        if i != 0 {
            total = total
                .iter()
                .zip(transactions.iter())
                .map(|(a, b)| a + b)
                .collect();
        }
        balances.push(PortfolioLedgerData {
            id,
            name,
            currency,
            series: transactions,
        });
    }

    const TAKE: usize = 30;
    let xs = std::iter::repeat(())
        .take(TAKE)
        .enumerate()
        .map(|(i, _)| i as f64)
        .collect::<Vec<_>>();
    let ys = total
        .iter()
        .copied()
        .rev()
        .take(TAKE)
        .rev()
        .collect::<Vec<_>>();
    let trend_of_total = linear_regression(&xs, &ys);

    let trend_of_total = 0.0;

    let mut data = HashMap::new();
    for ledger in portfolio.accounts.values().filter(|a| a.spending) {
        let mut transactions = ledger.records.clone();
        for transaction in &mut transactions {
            let rate = if ledger.currency != portfolio.base_currency {
                let rates = fetch_rate(&state, ledger, portfolio.base_currency).await?;
                rates[&transaction.date]
            } else {
                1.0
            };
            transaction.amount *= rate;
        }

        let categories = transactions
            .iter()
            .filter(|v| v.amount < 0.0)
            .sorted_by_key(|v| (v.date.year(), v.date.month(), v.category.clone()))
            .group_by(|v| (v.date.year(), v.date.month(), v.category.clone()))
            .into_iter()
            .map(|(g, v)| (g.clone(), v.into_iter().map(|v| v.amount).sum::<f64>()))
            .collect::<HashMap<_, _>>();

        for ((year, month, category), amount) in categories {
            let amount = -amount;
            let months = data.entry(month).or_insert(HashMap::new());
            let categories = months.entry(year).or_insert(HashMap::new());
            let total = categories.entry(category.clone()).or_insert(0.0);
            *total += amount;
        }
    }

    Ok(Json(PortfolioSummaryResponse {
        total_balance: PortfolioLedgersData {
            balances,
            timestamps: dates
                .into_iter()
                .map(|v| v.and_time(NaiveTime::default()).and_utc().timestamp())
                .collect(),
        },
        spend_per_month: SpendPerMonth { months: data },
    }))
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PortfolioLedgerData {
    pub id: String,
    pub name: String,
    pub currency: Currency,
    pub series: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PortfolioLedgersData {
    pub balances: Vec<PortfolioLedgerData>,
    #[ts(type = "number[]")]
    pub timestamps: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SpendPerMonth {
    pub months: HashMap<u32, HashMap<i32, HashMap<String, f64>>>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PortfolioSummaryResponse {
    pub total_balance: PortfolioLedgersData,
    pub spend_per_month: SpendPerMonth,
}

async fn fetch_rate(
    state: &AppState,
    ledger: &Account,
    base_currency: Currency,
) -> Result<BTreeMap<NaiveDate, f64>, AppError> {
    let mut cache = state.cache.lock().await;
    let rate = cache.get(ledger.currency, base_currency).await?;
    Ok(rate.rates.clone())
}
