use chrono::NaiveDate;

use crate::banks::{ExtendedLedgerRecord, LedgerRecord};

pub fn process(
    data: Vec<LedgerRecord>,
    initial_balance: Option<f64>,
    initial_date: Option<NaiveDate>,
) -> anyhow::Result<Vec<ExtendedLedgerRecord>> {
    let mut incoming = data
        .into_iter()
        .map(|v| ExtendedLedgerRecord {
            date: v.date,
            amount: v.amount,
            description: v.description.clone(),
            original_description: v.description,
            category: v.category.clone(),
            original_category: v.category,
            symbol: "".to_string(),
            comments: "".to_string(),
            checked: false,
        })
        .collect::<Vec<_>>();

    let records =
        if let (Some(initial_balance), Some(initial_date)) = (initial_balance, initial_date) {
            let initial_description = "Initial Balance";
            let initial_category = "initial";
            let initial = ExtendedLedgerRecord {
                date: initial_date,
                amount: initial_balance,
                description: initial_description.to_string(),
                original_description: initial_description.to_string(),
                category: initial_category.to_string(),
                original_category: initial_category.to_string(),
                symbol: "".to_string(),
                comments: "".to_string(),
                checked: false,
            };

            incoming.push(initial);
            incoming
        } else {
            incoming
        };

    Ok(records)
}
