use std::{io::Cursor, str::FromStr};

use chrono::{
    format::{parse, Fixed, Item, Numeric, Pad, Parsed},
    NaiveDate,
};
use csv::ReaderBuilder;
use serde::{de::Visitor, Deserializer};

use crate::fx::Currency;

use super::{Ledger, LedgerKind, LedgerRecord, ParsedAccount, Parser};

pub struct Wise {}

impl Parser for Wise {
    fn parse(name: &str, content: String) -> anyhow::Result<ParsedAccount> {
        let mut currency = Currency::CHF;
        let records = ReaderBuilder::new()
            .delimiter(b',')
            .from_reader(Cursor::new(&content))
            .deserialize::<Record>()
            .filter_map(|v| {
                v.map(|v| {
                    // Set currency from the record
                    currency = Currency::from_str(&v.currency).unwrap_or(Currency::CHF);

                    let description = v.description
                        .or_else(|| v.payer_name.clone())
                        .or_else(|| v.payee_name.clone())
                        .unwrap_or_else(|| "no description".into());

                    v.date.map(|date| {
                        LedgerRecord {
                            date,
                            amount: v.amount,
                            description,
                            category: "WISE".into(),
                        }
                    })
                })
                .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ParsedAccount {
            ledgers: vec![Ledger {
                name: name.to_string(),
                records,
                symbol: crate::fx::Symbol::Currency(currency),
                kind: LedgerKind::Bank,
            }],
        })
    }
}

#[derive(Debug, serde::Deserialize)]
struct Record {
    #[serde(rename = "Date Time", deserialize_with = "parse_date_with_time")]
    date: Option<NaiveDate>,
    #[serde(rename = "Amount")]
    amount: f64,
    #[serde(rename = "Currency")]
    currency: String,
    #[serde(rename = "Description")]
    description: Option<String>,
    #[serde(rename = "Payer Name")]
    payer_name: Option<String>,
    #[serde(rename = "Payee Name")]
    payee_name: Option<String>,
}

fn parse_date_with_time<'de, D: Deserializer<'de>>(de: D) -> Result<Option<NaiveDate>, D::Error> {
    const ITEMS: &[Item<'static>] = &[
        Item::Numeric(Numeric::Day, Pad::Zero),
        Item::Space(""),
        Item::Literal("-"),
        Item::Numeric(Numeric::Month, Pad::Zero),
        Item::Space(""),
        Item::Literal("-"),
        Item::Numeric(Numeric::Year, Pad::Zero),
        Item::Space(""),
        Item::Numeric(Numeric::Hour, Pad::Zero),
        Item::Space(""),
        Item::Literal(":"),
        Item::Numeric(Numeric::Minute, Pad::Zero),
        Item::Space(""),
        Item::Literal(":"),
        Item::Numeric(Numeric::Second, Pad::Zero),
        Item::Fixed(Fixed::Nanosecond),
        Item::Space(""),
    ];

    struct DateVisitor;
    struct MaybeDateVisitor;

    impl Visitor<'_> for DateVisitor {
        type Value = NaiveDate;

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let mut parsed = Parsed::new();
            parse(&mut parsed, v, ITEMS.iter()).map_err(E::custom)?;
            parsed.to_naive_date().map_err(E::custom)
        }

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("expected a date string of the format DD-MM-YYYY hh:mm::ss")
        }
    }

    impl<'de> Visitor<'de> for MaybeDateVisitor {
        type Value = Option<NaiveDate>;

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_some<D: Deserializer<'de>>(self, de: D) -> Result<Self::Value, D::Error> {
            Ok(Some(de.deserialize_str(DateVisitor)?))
        }

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter
                .write_str("expected a date string of the format DD-MM-YYYY hh:mm::ss or no value")
        }
    }

    de.deserialize_option(MaybeDateVisitor)
}

#[cfg(test)]
pub mod tests {
    use crate::banks::Parser;

    use super::Wise;

    #[test]
    pub fn parse_export() {
        const TRANSACTIONS: &str = r#""TransferWise ID",Date,"Date Time",Amount,Currency,Description,"Payment Reference","Running Balance","Exchange From","Exchange To","Exchange Rate","Payer Name","Payee Name","Payee Account Number",Merchant,"Card Last Four Digits","Card Holder Full Name",Attachment,Note,"Total fees","Exchange To Amount"
BALANCE-3783203164,29-07-2025,"29-07-2025 17:09:27.735",-53197.44,CHF,"Converted 53,197.44 CHF to 65,792.47 USD for USD balance",,0.00,CHF,USD,1.23908,,,,,,,,,99.60,65792.47
TRANSFER-1648398637,29-07-2025,"29-07-2025 09:04:55.522",52996.45,CHF,"Received money from Karolina Elzbieta Jartych with reference ",,53197.44,,,,"Karolina Elzbieta Jartych",,,,,,,,3.55,
TRANSFER-1646666581,28-07-2025,"28-07-2025 09:05:09.270",96.45,CHF,"Received money from Karolina Elzbieta Jartych with reference ",,200.99,,,,"Karolina Elzbieta Jartych",,,,,,,,3.55,
TRANSFER-1364555736,06-01-2025,"06-01-2025 12:36:08.427",104.54,CHF,"Received money from FORGED GMBH with reference ",,104.54,,,,"FORGED GMBH",,,,,,,,0.00,
"#;
        insta::assert_debug_snapshot!(Wise::parse("Wise", TRANSACTIONS.to_string()).unwrap());
    }

    #[test]
    pub fn test_wise_balance_calculation() {
        test_wise_balance_with_target(1036.74);
    }

    #[tokio::test]
    async fn test_wise_balance_api() {
        use crate::banks::test_utils::test_account_balance_api;
        
        let current_balance = test_account_balance_api("wise", "portfolio", "254110332425928707").await;
        
        // Test with the expected balance
        test_wise_balance_with_target_tolerance(current_balance, 1036.74, 1.0);
    }

    #[tokio::test]
    async fn test_wise_balance_api_test_data() {
        use crate::banks::test_utils::test_account_balance_api;
        
        let current_balance = test_account_balance_api("wise", "portfolio-test", "123456789").await;
        
        // Test exact balance - updated after first run
        let expected_balance = 2747.40;
        assert!((current_balance - expected_balance).abs() < 0.01, 
                "Expected Wise balance {:.2}, got {:.2}", expected_balance, current_balance);
    }

    fn test_wise_balance_with_target(target_balance: f64) {
        test_wise_balance_with_target_tolerance(target_balance, target_balance, 0.0);
    }

    fn test_wise_balance_with_target_tolerance(actual_balance: f64, target_balance: f64, tolerance_percent: f64) {
        let diff = (actual_balance - target_balance).abs();
        let percentage_diff = if target_balance != 0.0 {
            (diff / target_balance.abs()) * 100.0
        } else {
            0.0
        };
        
        println!("Actual balance: {:.2} CHF", actual_balance);
        println!("Target balance: {:.2} CHF", target_balance);
        println!("Difference: {:.2} CHF ({:.2}%)", diff, percentage_diff);
        
        if tolerance_percent == 0.0 {
            assert_eq!(
                actual_balance, target_balance,
                "Balance should be exactly {:.2} CHF, but got {:.2} CHF", 
                target_balance, actual_balance
            );
        } else {
            assert!(
                percentage_diff <= tolerance_percent,
                "Balance {:.2} CHF is not within {:.2}% of target {:.2} CHF (difference: {:.2}%)",
                actual_balance, tolerance_percent, target_balance, percentage_diff
            );
            println!("✅ Balance is within {:.2}% of target!", tolerance_percent);
        }
    }
}
