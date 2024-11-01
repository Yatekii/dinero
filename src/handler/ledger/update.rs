use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::realms::portfolio::state::Account;

// #[debug_handler]
// pub async fn handler(
//     State((adapter, state)): State<(PortfolioAdapter, PortfolioState)>,
//     Path(id): Path<String>,
//     Json(payload): Json<UpdateLedgerRequest>,
// ) -> Result<Json<UpdateLedgerResponse>, AppError> {
//     let mut guard = state.lock().await;

//     if let Some(ledger) = guard.accounts.get_mut(&id) {
//         let incoming = banks::parse(payload.transactions_data, ledger.format)?
//             .group_by([col("Date"), col("Description"), col("Category")])
//             .agg([
//                 col("Amount").sum(),
//                 col("*").exclude(["Amount"]),
//                 col("Date").count().alias("transactions"),
//             ])
//             .with_columns([
//                 col("Description").alias("description"),
//                 col("Category").alias("category"),
//                 lit("").alias("comments"),
//                 lit(false).alias("checked"),
//             ]);

//         let df = concat(
//             [ledger.ledgers.clone().lazy(), incoming],
//             UnionArgs::default(),
//         )?
//         .unique(
//             Some(vec![
//                 "Date".to_string(),
//                 "Amount".to_string(),
//                 "Category".to_string(),
//                 "Description".to_string(),
//             ]),
//             UniqueKeepStrategy::First,
//         )
//         .sort(
//             ["Date"],
//             SortMultipleOptions {
//                 descending: vec![false],
//                 nulls_last: false,
//                 multithreaded: true,
//                 maintain_order: true,
//             },
//         )
//         .select(&[
//             col("Date"),
//             col("Amount"),
//             col("Amount").cum_sum(false).alias("balance"),
//             col("Description"),
//             col("Category"),
//             col("description"),
//             col("category"),
//             col("comments"),
//             col("checked"),
//             col("transactions"),
//         ]);

//         ledger.ledgers = df.collect()?;
//     }

//     adapter.store(&guard)?;
//     let account = guard.accounts.get(&id).unwrap();

//     Ok(Json(UpdateLedgerResponse {
//         ledger: Account {
//             id: account.id.clone(),
//             name: account.name.clone(),
//             currency: account.currency.clone(),
//             format: account.format,
//             ledgers: account.ledgers.clone(),
//             initial_balance: account.initial_balance,
//             initial_date: account.initial_date,
//             spending: account.spending,
//         },
//     }))
// }

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateLedgerRequest {
    pub transactions_data: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateLedgerResponse {
    pub ledger: Account,
}
