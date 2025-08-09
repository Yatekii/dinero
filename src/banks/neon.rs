use std::io::Cursor;

use chrono::NaiveDate;
use csv::ReaderBuilder;

use super::{Ledger, LedgerKind, LedgerRecord, ParsedAccount, Parser};

pub struct Neon {}

impl Parser for Neon {
    fn parse(name: &str, content: String) -> anyhow::Result<ParsedAccount> {
        let records = ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(Cursor::new(&content))
            .deserialize::<Record>()
            .filter(|v| (v.as_ref()).map_or("", |v| &v.spaces) == "no")
            .map(|v| {
                v.map(|v| LedgerRecord {
                    date: v.date,
                    amount: v.amount,
                    description: v.description,
                    category: v.category,
                })
            })
            .collect::<Result<_, _>>()?;

        Ok(ParsedAccount {
            ledgers: vec![Ledger {
                name: name.to_string(),
                records,
                symbol: crate::fx::Symbol::Currency(crate::fx::Currency::CHF),
                kind: LedgerKind::Bank,
            }],
        })
    }
}

#[derive(Debug, serde::Deserialize)]
struct Record {
    #[serde(rename = "Date")]
    date: NaiveDate,
    #[serde(rename = "Amount")]
    amount: f64,
    #[serde(rename = "Original amount")]
    _original_amount: Option<f64>,
    #[serde(rename = "Original currency")]
    _original_currency: Option<String>,
    #[serde(rename = "Exchange rate")]
    _exchange_rate: Option<f64>,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Subject")]
    _subject: String,
    #[serde(rename = "Category")]
    category: String,
    #[serde(rename = "Tags")]
    _tags: String,
    #[serde(rename = "Wise")]
    _wise: String,
    #[serde(rename = "Spaces")]
    spaces: String,
}

#[cfg(test)]
pub mod tests {
    use crate::banks::Parser;

    use super::Neon;

    #[test]
    pub fn parse_export() {
        const TRANSACTIONS: &str = r#""Date";"Amount";"Original amount";"Original currency";"Exchange rate";"Description";"Subject";"Category";"Tags";"Wise";"Spaces"
"2019-09-17";"-1009.00";"";"";"";"Urech Optik";;"health";"";"no";"no"
"2019-08-02";"-200.00";"";"";"";"ZKB ZH HB SIHLQUAI 2";;"cash";"";"no";"no"
"2019-07-22";"-150.00";"";"";"";"Hanspeter Schoop";"ANZAHLUNG";"uncategorized";"";"no";"no"
"2019-07-22";"-439.70";"";"";"";"Generali";"000000002198972910070138387";"finances";"";"no";"no"
"2019-07-15";"-92.20";"";"";"";"Urbach Optik";"000000000000000041017313152";"health";"";"no";"no"
"2019-06-03";"-140.00";"";"";"";"Regionalpolizei Lenzburg";"000000000000000003478000113";"finances";"";"no";"no"
"2019-05-10";"30.00";"";"";"";"neon Switzerland AG";;"income";"";"no";"no"
"2019-05-10";"30.00";"";"";"";"neon Switzerland AG";;"income";"";"no";"no"
"2019-05-09";"3000.00";"";"";"";"Technokrat GmbH";"Lohn";"income_salary";"income";"no";"no"
"#;
        insta::assert_debug_snapshot!(Neon::parse("Neon", TRANSACTIONS.to_string()).unwrap());
    }

    #[tokio::test]
    async fn test_neon_balance_api_test_data() {
        use crate::banks::test_utils::test_account_balance_api;
        
        let current_balance = test_account_balance_api("neon", "portfolio-test", "123456789").await;
        
        // Test exact balance - updated after first run
        let expected_balance = 746.82;
        assert!((current_balance - expected_balance).abs() < 0.01, 
                "Expected Neon balance {:.2}, got {:.2}", expected_balance, current_balance);
    }
}
