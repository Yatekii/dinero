pub mod create;
pub mod get;
pub mod list;
pub mod summary;
pub mod update;

// #[cfg(test)]
// mod tests {
//     use std::sync::Arc;

//     use axum::{
//         extract::{Path, State},
//         Json,
//     };
//     use chrono::NaiveDate;
//     use float_cmp::assert_approx_eq;
//     use tokio::sync::Mutex;

//     use crate::{
//         handler::ledger::{create::CreateLedgerRequest, update::UpdateLedgerRequest},
//         realms::portfolio::state::Portfolio,
//     };

//     #[tokio::test]
//     async fn create() {
//         let name = "Neon";
//         let currency = "CHF";
//         let Json(response) = crate::handler::ledger::create::handler(
//             State(Arc::new(Mutex::new(Portfolio::default()))),
//             Json(CreateLedgerRequest {
//                 transactions_data: TEST_TRANSACTION_DATA_FULL.into(),
//                 format: crate::cli::BankFormat::Neon,
//                 initial_balance: Some(42.),
//                 initial_date: Some(NaiveDate::from_ymd_opt(2019, 1, 1).unwrap()),
//                 name: name.into(),
//                 currency: "CHF".into(),
//                 spending: true,
//             }),
//         )
//         .await
//         .unwrap();

//         assert_eq!(
//             slug::slugify(format!("{}-{}", &name, &currency)),
//             response.account.id
//         );
//         assert_eq!(name, response.account.name);
//         assert_eq!(currency, response.account.currency);

//         let sum = response
//             .account
//             .ledgers
//             .column("balance")
//             .unwrap()
//             .f64()
//             .unwrap()
//             .last()
//             .unwrap();
//         assert_approx_eq!(f64, 1071.1, sum);
//     }

//     #[tokio::test]
//     async fn update() {
//         let name = "Neon";
//         let currency = "CHF";
//         let state = Arc::new(Mutex::new(Portfolio::default()));
//         let Json(response) = crate::handler::ledger::create::handler(
//             State(state.clone()),
//             Json(CreateLedgerRequest {
//                 transactions_data: TEST_TRANSACTION_DATA_HALF.into(),
//                 format: crate::cli::BankFormat::Neon,
//                 initial_balance: Some(42.),
//                 initial_date: Some(NaiveDate::from_ymd_opt(2019, 1, 1).unwrap()),
//                 name: name.into(),
//                 currency: "CHF".into(),
//                 spending: false,
//             }),
//         )
//         .await
//         .unwrap();

//         let Json(response) = crate::handler::ledger::update::handler(
//             State(state.clone()),
//             Path(response.account.id),
//             Json(UpdateLedgerRequest {
//                 transactions_data: TEST_TRANSACTION_DATA_DELETE_RANDOM.into(),
//             }),
//         )
//         .await
//         .unwrap();

//         assert_eq!(
//             slug::slugify(format!("{}-{}", &name, &currency)),
//             response.ledger.id
//         );
//         assert_eq!(name, response.ledger.name);
//         assert_eq!(currency, response.ledger.currency);

//         let sum = response
//             .ledger
//             .ledgers
//             .column("balance")
//             .unwrap()
//             .f64()
//             .unwrap()
//             .last()
//             .unwrap();
//         assert_approx_eq!(f64, 1211.1, sum);

//         let Json(response) = crate::handler::ledger::update::handler(
//             State(state),
//             Path(response.ledger.id),
//             Json(UpdateLedgerRequest {
//                 transactions_data: TEST_TRANSACTION_DATA_MINUS_ONE.into(),
//             }),
//         )
//         .await
//         .unwrap();

//         let sum = response
//             .ledger
//             .ledgers
//             .column("balance")
//             .unwrap()
//             .f64()
//             .unwrap()
//             .last()
//             .unwrap();
//         assert_approx_eq!(f64, 1071.1, sum);
//     }

//     const TEST_TRANSACTION_DATA_MINUS_ONE: &str = r#""Date";"Amount";"Original amount";"Original currency";"Exchange rate";"Description";"Subject";"Category";"Tags";"Wise";"Spaces"
// "2019-09-17";"-1009.00";"";"";"";"Urech Optik";;"health";"";"no";"no"
// "2019-08-02";"-200.00";"";"";"";"ZKB ZH HB SIHLQUAI 2";;"cash";"";"no";"no"
// "2019-07-22";"-150.00";"";"";"";"Hanspeter Schoop";"ANZAHLUNG";"uncategorized";"";"no";"no"
// "2019-07-22";"-439.70";"";"";"";"Generali";"000000002198972910070138387";"finances";"";"no";"no"
// "2019-07-15";"-92.20";"";"";"";"Urbach Optik";"000000000000000041017313152";"health";"";"no";"no"
// "2019-06-03";"-140.00";"";"";"";"Regionalpolizei Lenzburg";"000000000000000003478000113";"finances";"";"no";"no"
// "2019-05-10";"30.00";"";"";"";"neon Switzerland AG";;"income";"";"no";"no"
// "2019-05-10";"30.00";"";"";"";"neon Switzerland AG";;"income";"";"no";"no"
// "#;

//     const TEST_TRANSACTION_DATA_DELETE_RANDOM: &str = r#""Date";"Amount";"Original amount";"Original currency";"Exchange rate";"Description";"Subject";"Category";"Tags";"Wise";"Spaces"
// "2019-09-17";"-1009.00";"";"";"";"Urech Optik";;"health";"";"no";"no"
// "2019-08-02";"-200.00";"";"";"";"ZKB ZH HB SIHLQUAI 2";;"cash";"";"no";"no"
// "2019-07-22";"-150.00";"";"";"";"Hanspeter Schoop";"ANZAHLUNG";"uncategorized";"";"no";"no"
// "2019-07-22";"-439.70";"";"";"";"Generali";"000000002198972910070138387";"finances";"";"no";"no"
// "2019-07-15";"-92.20";"";"";"";"Urbach Optik";"000000000000000041017313152";"health";"";"no";"no"
// "2019-05-10";"30.00";"";"";"";"neon Switzerland AG";;"income";"";"no";"no"
// "2019-05-10";"30.00";"";"";"";"neon Switzerland AG";;"income";"";"no";"no"
// "2019-05-09";"3000.00";"";"";"";"Technokrat GmbH";"Lohn";"income_salary";"income";"no";"no"
// "#;

//     const TEST_TRANSACTION_DATA_HALF: &str = r#""Date";"Amount";"Original amount";"Original currency";"Exchange rate";"Description";"Subject";"Category";"Tags";"Wise";"Spaces"
// "2019-09-17";"-1009.00";"";"";"";"Urech Optik";;"health";"";"no";"no"
// "2019-08-02";"-200.00";"";"";"";"ZKB ZH HB SIHLQUAI 2";;"cash";"";"no";"no"
// "2019-07-22";"-150.00";"";"";"";"Hanspeter Schoop";"ANZAHLUNG";"uncategorized";"";"no";"no"
// "2019-07-22";"-439.70";"";"";"";"Generali";"000000002198972910070138387";"finances";"";"no";"no"
// "#;

//     const TEST_TRANSACTION_DATA_FULL: &str = r#""Date";"Amount";"Original amount";"Original currency";"Exchange rate";"Description";"Subject";"Category";"Tags";"Wise";"Spaces"
// "2019-09-17";"-1009.00";"";"";"";"Urech Optik";;"health";"";"no";"no"
// "2019-08-02";"-200.00";"";"";"";"ZKB ZH HB SIHLQUAI 2";;"cash";"";"no";"no"
// "2019-07-22";"-150.00";"";"";"";"Hanspeter Schoop";"ANZAHLUNG";"uncategorized";"";"no";"no"
// "2019-07-22";"-439.70";"";"";"";"Generali";"000000002198972910070138387";"finances";"";"no";"no"
// "2019-07-15";"-92.20";"";"";"";"Urbach Optik";"000000000000000041017313152";"health";"";"no";"no"
// "2019-06-03";"-140.00";"";"";"";"Regionalpolizei Lenzburg";"000000000000000003478000113";"finances";"";"no";"no"
// "2019-05-10";"30.00";"";"";"";"neon Switzerland AG";;"income";"";"no";"no"
// "2019-05-10";"30.00";"";"";"";"neon Switzerland AG";;"income";"";"no";"no"
// "2019-05-09";"3000.00";"";"";"";"Technokrat GmbH";"Lohn";"income_salary";"income";"no";"no"
// "#;
// }
