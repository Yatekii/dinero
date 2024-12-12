use std::collections::{BTreeMap, HashMap};

use anyhow::bail;
use axum::{debug_handler, extract::State, Json};
use chrono::{Datelike, Days, NaiveDate, NaiveTime};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    error::AppError,
    fx::Currency,
    handler::auth::user::User,
    realms::portfolio::state::Account,
    state::{CacheState, PortfolioAdapter},
};

#[debug_handler(state = crate::state::AppState)]
pub async fn handler(
    State(adapter): State<PortfolioAdapter>,
    State(cache): State<CacheState>,
    user: User,
) -> Result<Json<PortfolioSummaryResponse>, AppError> {
    let portfolio = user.portfolio(adapter)?;

    if portfolio.accounts.is_empty() {
        return Ok(Json(PortfolioSummaryResponse {
            total_balance: PortfolioLedgersData {
                balances: vec![],
                timestamps: vec![],
            },
            total_prediction: PortfolioLedgerData {
                id: "total-prediction".to_string(),
                name: "Prediction of the total".to_string(),
                currency: portfolio.base_currency,
                series: vec![],
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
            fetch_rate(cache.clone(), ledger, portfolio.base_currency).await?
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
                if sum != 0.0 {
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

    const NUM_SAMPLES: usize = 3 * 365;

    let mut balances = Vec::new();
    let mut total = ledgers
        .iter()
        .next()
        .map(|v| v.1 .2.clone())
        .unwrap_or_default();
    for (i, (id, (name, currency, mut transactions))) in ledgers.into_iter().enumerate() {
        if i != 0 {
            total = total
                .iter()
                .zip(transactions.iter())
                .map(|(a, b)| a + b)
                .collect();
        }
        // Take 3 years worth of data.

        balances.push(PortfolioLedgerData {
            id,
            name,
            currency,
            series: transactions
                .drain(transactions.len() - NUM_SAMPLES..)
                .collect(),
        });
    }

    const TAKE: usize = 300;
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
    let (m, q) = linear_regression(&xs, &ys)?;

    let total_prediction = PortfolioLedgerData {
        id: "total-prediction".to_string(),
        name: "Prediction of the total".to_string(),
        currency: portfolio.base_currency,
        series: (0..365).map(|x| m * ((x + TAKE) as f64) + q).collect(),
    };

    let mut data = HashMap::new();
    for ledger in portfolio.accounts.values().filter(|a| a.spending) {
        let mut transactions = ledger.records.clone();
        for transaction in &mut transactions {
            let rate = if ledger.currency != portfolio.base_currency {
                let rates = fetch_rate(cache.clone(), ledger, portfolio.base_currency).await?;
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

    let dates_len = dates.len();
    Ok(Json(PortfolioSummaryResponse {
        total_balance: PortfolioLedgersData {
            balances,
            timestamps: dates
                .into_iter()
                .skip(dates_len - NUM_SAMPLES)
                .map(|v| v.and_time(NaiveTime::default()).and_utc().timestamp())
                .collect(),
        },
        total_prediction,
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
    pub total_prediction: PortfolioLedgerData,
    pub spend_per_month: SpendPerMonth,
}

async fn fetch_rate(
    cache: CacheState,
    ledger: &Account,
    base_currency: Currency,
) -> Result<BTreeMap<NaiveDate, f64>, AppError> {
    let mut cache = cache.lock().await;
    let rate = cache.get(ledger.currency, base_currency).await?;
    Ok(rate.rates.clone())
}

/// Calculates a linear regression with a known mean.
///
/// Lower-level linear regression function. Assumes that `x_mean` and `y_mean`
/// have already been calculated. Returns `Error::DivByZero` if
///
/// * the slope is too steep to represent, approaching infinity.
///
/// Since there is a mean, this function assumes that `xs` and `ys` are both non-empty.
///
/// Returns `Ok((slope, intercept))` of the regression line.
pub fn lin_reg<I>(xys: I, x_mean: f64, y_mean: f64) -> anyhow::Result<(f64, f64)>
where
    I: Iterator<Item = (f64, f64)>,
{
    // SUM (x-mean(x))^2
    let mut xxm2 = 0.0;

    // SUM (x-mean(x)) (y-mean(y))
    let mut xmym2 = 0.0;

    for (x, y) in xys {
        xxm2 += (x - x_mean) * (x - x_mean);
        xmym2 += (x - x_mean) * (y - y_mean);
    }

    let slope = xmym2 / xxm2;

    // we check for divide-by-zero after the fact
    if slope.is_nan() {
        bail!("The slope is too steep to represent (approaching infinity)");
    }

    let intercept = y_mean - slope * x_mean;

    Ok((slope, intercept))
}

/// Two-pass simple linear regression from slices.
///
/// Calculates the linear regression from two slices, one for x- and one for y-values, by
/// calculating the mean and then calling `lin_reg`.
///
/// Returns `Ok(slope, intercept)` of the regression line.
///
/// # Errors
///
/// Returns an error if
///
/// * `xs` and `ys` differ in length
/// * `xs` or `ys` are empty
/// * the slope is too steep to represent, approaching infinity
/// * the number of elements cannot be represented as an `F`
///
pub fn linear_regression(xs: &[f64], ys: &[f64]) -> anyhow::Result<(f64, f64)> {
    if xs.len() != ys.len() {
        bail!("Input vector lengths do not match");
    }

    if xs.is_empty() {
        bail!("Input set is empty. Cannot calculate mean");
    }
    let x_sum: f64 = xs.iter().cloned().sum();
    let n = xs.len() as f64;
    let x_mean = x_sum / n;
    let y_sum: f64 = ys.iter().cloned().sum();
    let y_mean = y_sum / n;

    lin_reg(xs.iter().copied().zip(ys.iter().copied()), x_mean, y_mean)
}
