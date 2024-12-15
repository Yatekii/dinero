use anyhow::anyhow;
use axum::{
    debug_handler,
    extract::{Path, Query, State},
    Json,
};
use chrono::NaiveDate;

use crate::{
    error::AppError, handler::auth::user::User, realms::portfolio::state::Account,
    state::PortfolioAdapter,
};

#[debug_handler(state = crate::state::AppState)]
pub async fn handler(
    State(adapter): State<PortfolioAdapter>,
    Path(id): Path<String>,
    Query(filter): Query<Filter>,
    user: User,
) -> Result<Json<Account>, AppError> {
    let portfolio = user.portfolio(adapter)?;
    let account = &portfolio.accounts.get(&id);
    let Some(account) = account else {
        return Err(anyhow!("{id} was not found"))?;
    };

    if account.owner != user.sub {
        return Err(anyhow!("Not authorized!"))?;
    }

    let mut ledgers = account.ledgers.clone();
    for ledger in &mut ledgers {
        if let Some(from) = &filter.from {
            let from = NaiveDate::parse_from_str(from, "%Y-%m-%d")?;
            ledger.records.retain(|v| v.date >= from);
        }

        if let Some(to) = &filter.to {
            let to = NaiveDate::parse_from_str(to, "%Y-%m-%d")?;
            ledger.records.retain(|v| v.date <= to);
        }
    }

    let account = Account {
        id: account.id.clone(),
        owner: user.sub,
        name: account.name.clone(),
        currency: account.currency,
        format: account.format,
        ledgers,
        initial_balance: account.initial_balance,
        initial_date: account.initial_date,
        spending: account.spending,
    };

    Ok(Json(account))
}

#[derive(serde::Deserialize)]
pub struct Filter {
    from: Option<String>,
    to: Option<String>,
}
