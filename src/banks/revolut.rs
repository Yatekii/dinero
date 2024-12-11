use std::io::Cursor;

use chrono::{
    format::{parse, Fixed, Item, Numeric, Pad, Parsed},
    NaiveDate,
};
use csv::ReaderBuilder;
use serde::{de::Visitor, Deserializer};

use super::{LedgerRecord, ParsedAccount, ParsedLedger, Parser};

pub struct Revolut {}

impl Parser for Revolut {
    fn parse(name: &str, content: String) -> anyhow::Result<ParsedAccount> {
        let records = ReaderBuilder::new()
            .delimiter(b',')
            .from_reader(Cursor::new(&content))
            .deserialize::<Record>()
            .filter(|v| (v.as_ref()).map_or("", |v| &v.state) == "COMPLETED")
            .filter_map(|v| {
                v.map(|v| {
                    v.date.map(|date| LedgerRecord {
                        date,
                        amount: v.amount,
                        description: v.description,
                        category: v.category,
                        symbol: v.currency,
                    })
                })
                .transpose()
            })
            .collect::<Result<_, _>>()?;

        Ok(ParsedAccount {
            ledgers: vec![ParsedLedger {
                name: name.to_string(),
                records,
            }],
        })
    }
}

#[derive(Debug, serde::Deserialize)]
struct Record {
    #[serde(rename = "Completed Date", deserialize_with = "parse_date_with_time")]
    date: Option<NaiveDate>,
    #[serde(rename = "Amount")]
    amount: f64,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Type")]
    category: String,
    #[serde(rename = "Currency")]
    currency: String,
    #[serde(rename = "State")]
    state: String,
}

fn parse_date_with_time<'de, D: Deserializer<'de>>(de: D) -> Result<Option<NaiveDate>, D::Error> {
    const ITEMS: &[Item<'static>] = &[
        Item::Numeric(Numeric::Year, Pad::Zero),
        Item::Space(""),
        Item::Literal("-"),
        Item::Numeric(Numeric::Month, Pad::Zero),
        Item::Space(""),
        Item::Literal("-"),
        Item::Numeric(Numeric::Day, Pad::Zero),
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
            formatter.write_str("expected a date string of the format YYYY-MM-DD hh:mm::ss")
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
                .write_str("expected a date string of the format YYYY-MM-DD hh:mm::ss or no value")
        }
    }

    de.deserialize_option(MaybeDateVisitor)
}

#[cfg(test)]
pub mod tests {
    use crate::banks::Parser;

    use super::Revolut;

    #[test]
    pub fn parse_export() {
        const TRANSACTIONS: &str = r#"Type,Product,Started Date,Completed Date,Description,Amount,Fee,Currency,State,Balance
CARD_PAYMENT,Current,2023-01-02 03:54:18,2023-01-02 13:11:10,migrolino,-7.50,0.00,CHF,COMPLETED,91.65
TOPUP,Current,2023-01-05 09:14:08,2023-01-05 09:14:30,Auto Top-Up by *0382,75.00,0.00,CHF,COMPLETED,166.65
CARD_PAYMENT,Current,2023-01-05 09:14:00,2023-01-06 23:25:07,CCBill,-19.74,0.00,CHF,COMPLETED,146.91
CARD_PAYMENT,Current,2023-01-08 03:42:39,2023-01-08 11:34:11,GitHub,-7.44,0.07,CHF,COMPLETED,139.40
CARD_PAYMENT,Current,2023-01-12 13:55:31,2023-01-18 21:20:55,Contabo,-6.48,0.00,CHF,COMPLETED,132.92
CARD_PAYMENT,Current,2023-01-22 14:06:23,2023-01-28 22:57:06,Contabo,-11.32,0.11,CHF,COMPLETED,121.49
CARD_PAYMENT,Current,2023-02-04 09:14:03,2023-02-06 22:47:13,CCBill,-20.02,0.20,CHF,COMPLETED,101.27
CARD_PAYMENT,Current,2023-02-08 03:56:44,2023-02-08 11:32:47,GitHub,-18.14,0.00,CHF,COMPLETED,83.13
CARD_PAYMENT,Current,2023-02-12 13:52:03,2023-02-18 22:24:06,Contabo,-6.38,0.06,CHF,COMPLETED,76.69
TOPUP,Current,2023-02-22 13:54:55,2023-02-22 13:54:57,Auto Top-Up by *0382,75.00,0.00,CHF,COMPLETED,151.69
CARD_PAYMENT,Current,2023-02-22 13:54:54,2023-02-28 21:56:47,Contabo,-11.18,0.00,CHF,COMPLETED,140.51
CARD_PAYMENT,Current,2023-03-01 21:55:56,2023-03-02 11:41:17,Google Cloud,-7.38,0.00,CHF,COMPLETED,133.13
CARD_PAYMENT,Current,2023-03-06 09:14:09,2023-03-07 23:30:03,CCBill,-19.95,0.00,CHF,COMPLETED,113.18
CARD_PAYMENT,Current,2023-03-08 03:06:33,2023-03-08 18:34:22,GitHub,-7.54,0.00,CHF,COMPLETED,105.64
CARD_PAYMENT,Current,2023-03-12 13:48:50,,Contabo,-6.34,0.06,CHF,REVERTED,
CARD_PAYMENT,Current,2023-03-18 09:05:30,2023-03-21 11:32:07,Contabo,-6.44,0.00,CHF,COMPLETED,99.20
TRANSFER,Current,2023-03-25 20:17:05,2023-03-25 20:17:10,To GARY PETER BYRNE,-30.00,0.00,CHF,COMPLETED,69.20
TOPUP,Current,2023-03-25 20:17:05,2023-03-25 20:17:15,Auto Top-Up by *0382,75.00,0.00,CHF,COMPLETED,144.20
CARD_PAYMENT,Current,2023-03-22 13:52:30,2023-03-28 20:28:02,Contabo,-11.25,0.00,CHF,COMPLETED,132.95
CARD_PAYMENT,Current,2023-04-01 19:41:19,2023-04-02 10:48:11,Google Cloud,-12.50,0.00,CHF,COMPLETED,120.45
CARD_PAYMENT,Current,2023-04-05 09:13:54,2023-04-06 23:16:17,CCBill,-19.88,0.00,CHF,COMPLETED,100.57
CARD_PAYMENT,Current,2023-04-12 12:45:25,2023-04-18 21:18:30,Contabo,-6.37,0.00,CHF,COMPLETED,94.20
CARD_PAYMENT,Current,2023-05-05 09:13:52,2023-05-07 00:58:36,CCBill,-19.54,0.00,CHF,COMPLETED,74.66
CARD_PAYMENT,Current,2023-06-04 09:13:44,2023-06-06 01:37:35,CCBill,-19.50,0.20,CHF,COMPLETED,54.96
CARD_PAYMENT,Current,2023-06-12 22:52:27,,Migros,-1.90,0.00,CHF,REVERTED,
CARD_PAYMENT,Current,2023-07-08 08:06:21,2023-07-08 20:53:22,Sky,-14.90,0.00,CHF,COMPLETED,40.06
CARD_PAYMENT,Current,2023-08-01 19:23:16,2023-08-02 10:43:45,Google Cloud,-7.00,0.00,CHF,COMPLETED,33.06
EXCHANGE,Current,2023-08-22 20:58:34,2023-08-22 20:58:34,Exchanged to USD,-8.80,0.00,CHF,COMPLETED,24.26
CARD_PAYMENT,Current,2023-09-01 21:12:54,2023-09-02 11:19:08,Google Cloud,-7.75,0.00,CHF,COMPLETED,16.51
CARD_PAYMENT,Current,2023-09-11 23:01:47,2023-09-12 13:55:10,Gst,-7.45,0.00,CHF,COMPLETED,9.06
CARD_PAYMENT,Current,2023-09-12 00:57:32,2023-09-12 21:52:25,Confiteria Antojos,-4.94,0.00,CHF,COMPLETED,4.12
CARD_PAYMENT,Current,2023-10-23 17:56:02,2023-10-24 13:33:24,Coop,-1.45,0.00,CHF,COMPLETED,2.67
"#;
        insta::assert_debug_snapshot!(Revolut::parse("Neon", TRANSACTIONS.to_string()).unwrap());
    }
}
