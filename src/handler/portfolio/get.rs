use std::collections::{BTreeMap, HashMap};

use anyhow::bail;
use axum::{debug_handler, extract::State, Json};
use chrono::{Datelike, Days, NaiveDate, NaiveTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    banks::LedgerKind,
    error::AppError,
    fx::{Currency, Symbol},
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
        return Ok(Json(PortfolioSummaryResponse::new(portfolio.base_currency)));
    }

    let dates = get_date_series(&portfolio.accounts);
    const NUM_SAMPLES: usize = 3 * 365;
    let dates_len = dates.len();
    let samples_to_skip = dates_len.saturating_sub(NUM_SAMPLES);

    let mut accounts = HashMap::new();
    for account in portfolio.accounts.values() {
        let mut ledger_balances = HashMap::<Symbol, Vec<_>>::new();

        for ledger in &account.ledgers {
            let mut ledger_worth_on_date = 0.0;
            // If Some this contains the base currency rates against the ticker.
            // This is None if the ticker is traded against the base currency.
            let mut ticker_to_base = None;
            // We need to get the ticker_to_base_rates.
            let needs_ticker_to_base_transform =
                ledger.kind == LedgerKind::Stock && account.currency != portfolio.base_currency;
            let rates = if ledger.symbol != portfolio.base_currency {
                if needs_ticker_to_base_transform {
                    ticker_to_base = Some(
                        fetch_rate(
                            cache.clone(),
                            &Symbol::Currency(account.currency),
                            portfolio.base_currency,
                        )
                        .await?,
                    );
                }
                Some(fetch_rate(cache.clone(), &ledger.symbol, portfolio.base_currency).await?)
            } else {
                None
            };
            for date in &dates {
                let sum_on_date = ledger
                    .records
                    .iter()
                    .filter(|v| &v.date == date)
                    .map(|v| v.amount)
                    .sum::<f64>();

                let rate_currency = ticker_to_base
                    .as_ref()
                    .map_or(1.0, |ttb| rate_for_date(ttb, date));
                let rate = if let Some(rates) = &rates {
                    rate_for_date(rates, date)
                } else {
                    1.0
                };

                ledger_worth_on_date += sum_on_date;
                let amount = ledger_worth_on_date * rate * rate_currency;

                ledger_balances
                    .entry(ledger.symbol.clone())
                    .or_default()
                    .push(amount);
            }
        }

        let mut account_balances = vec![0.0; dates.len()];
        for b in ledger_balances.into_values() {
            for (t, b) in account_balances.iter_mut().zip(b) {
                *t += b;
            }
        }

        accounts.insert(
            account.id.clone(),
            (account.name.clone(), account.currency, account_balances),
        );
    }

    let mut balances = Vec::new();
    let mut total = vec![0.0; dates_len];
    for (id, (name, currency, mut transactions)) in accounts.into_iter() {
        for (total, b) in total.iter_mut().zip(transactions.iter()) {
            *total += b;
        }

        // Take 3 years worth of data.
        balances.push(PortfolioLedgerData {
            id,
            name,
            currency,
            series: transactions.drain(samples_to_skip..).collect(),
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
    for account in portfolio.accounts.values().filter(|a| a.spending) {
        for ledger in &account.ledgers {
            let mut transactions = ledger.records.clone();
            for transaction in &mut transactions {
                let rate = if ledger.symbol != Symbol::Currency(portfolio.base_currency) {
                    let rates =
                        fetch_rate(cache.clone(), &ledger.symbol, portfolio.base_currency).await?;
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
    }

    Ok(Json(PortfolioSummaryResponse {
        total_balance: PortfolioLedgersData {
            balances,
            timestamps: dates
                .into_iter()
                .skip(samples_to_skip)
                .map(|v| v.and_time(NaiveTime::default()).and_utc().timestamp())
                .collect(),
        },
        total_prediction,
        spend_per_month: SpendPerMonth { months: data },
        base_currency: portfolio.base_currency,
    }))
}

fn rate_for_date(rates: &BTreeMap<NaiveDate, f64>, date: &NaiveDate) -> f64 {
    let mut result = None;
    let mut days = 0;
    while result.is_none() {
        let date = date.checked_sub_days(Days::new(days)).unwrap();
        days += 1;
        result = rates.get(&date).copied()
    }
    result.unwrap_or(1.0)
}

/// Get all the dates from the oldest found transaction to today.
fn get_date_series(accounts: &HashMap<String, Account>) -> Vec<NaiveDate> {
    let max_date = Utc::now().naive_utc().date();
    let mut min_date = NaiveDate::MAX;
    for account in accounts.values() {
        for ledger in &account.ledgers {
            let min = ledger
                .records
                .iter()
                .map(|v| v.date)
                .min()
                .unwrap_or(NaiveDate::MAX);

            min_date = min_date.min(min);
        }
    }

    (min_date.iter_days().take_while(|d| d <= &max_date)).collect::<Vec<_>>()
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
    pub base_currency: Currency,
}

impl PortfolioSummaryResponse {
    pub fn new(currency: Currency) -> Self {
        Self {
            total_balance: PortfolioLedgersData {
                balances: vec![],
                timestamps: vec![],
            },
            total_prediction: PortfolioLedgerData {
                id: "total-prediction".to_string(),
                name: "Prediction of the total".to_string(),
                currency,
                series: vec![],
            },
            spend_per_month: SpendPerMonth {
                months: HashMap::new(),
            },
            base_currency: Currency::CHF,
        }
    }
}

async fn fetch_rate(
    cache: CacheState,
    symbol: &Symbol,
    base_currency: Currency,
) -> Result<BTreeMap<NaiveDate, f64>, AppError> {
    let mut cache = cache.lock().await;
    let rate = cache.get(symbol, &Symbol::Currency(base_currency)).await?;
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
