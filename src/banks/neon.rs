use std::io::Cursor;

use chrono::NaiveDate;
use csv::ReaderBuilder;

use super::{LedgerRecord, ParsedAccount, ParsedLedger, Parser};

pub struct Neon {}

impl Parser for Neon {
    fn parse(name: String, content: String) -> anyhow::Result<ParsedAccount> {
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
                    symbol: "CHF".to_string(),
                })
            })
            .collect::<Result<_, _>>()?;

        Ok(ParsedAccount {
            ledgers: vec![ParsedLedger { name, records }],
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
        insta::assert_debug_snapshot!(
            Neon::parse("Neon".to_string(), TRANSACTIONS.to_string()).unwrap()
        );
    }
}
