use std::io::Cursor;

use anyhow::{bail, Error};
use chrono::NaiveDate;
use csv::{Reader, ReaderBuilder};

use super::{LedgerRecord, ParsedAccount, Parser};

pub struct Ibkr {}

impl Parser for Ibkr {
    fn parse(name: String, content: String) -> anyhow::Result<ParsedAccount> {
        let mut ledgers = vec![];
        let lines = content.lines().collect::<Vec<_>>();
        let mut end = lines.len();
        let mut header_found = false;
        for (index, line) in lines.iter().enumerate().rev() {
            if line.starts_with("\"HEADER\"") {
                header_found = true;
                let content = &lines[index..end].join("\n");

                let reader = ReaderBuilder::new()
                    .delimiter(b',')
                    .from_reader(Cursor::new(&content));

                let records = if line.contains("\"CTRN\"") {
                    parse_cash_transactions(reader)?
                } else if line.contains("\"TRNT\"") {
                    parse_stock_transactions(reader)?
                } else {
                    bail!("Unknown export")
                };

                ledgers.push(super::ParsedLedger {
                    name: name.clone(),
                    records,
                });

                end = index;
            }
        }

        if !header_found {
            bail!("The data seems to not be in IBKR format as no HEADER lines were found")
        }

        Ok(ParsedAccount { ledgers })
    }
}

fn parse_stock_transactions(
    mut reader: Reader<Cursor<&&String>>,
) -> Result<Vec<LedgerRecord>, Error> {
    let records = reader
        .deserialize::<StockRecord>()
        .map(|v| {
            v.map(|v| LedgerRecord {
                date: v.date,
                amount: v.amount,
                description: v.description,
                category: "Broker".to_string(),
                symbol: v.symbol,
            })
        })
        .collect::<Result<_, _>>()?;

    Ok(records)
}

#[derive(Debug, serde::Deserialize)]
struct StockRecord {
    #[serde(rename = "TradeDate")]
    date: NaiveDate,
    #[serde(rename = "Quantity")]
    amount: f64,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Symbol")]
    symbol: String,
}

fn parse_cash_transactions(
    mut reader: Reader<Cursor<&&String>>,
) -> Result<Vec<LedgerRecord>, Error> {
    let records = reader
        .deserialize::<CashRecord>()
        .map(|v| {
            v.map(|v| LedgerRecord {
                date: v.date,
                amount: v.amount,
                description: v.description,
                category: "Broker".to_string(),
                symbol: v.symbol,
            })
        })
        .collect::<Result<_, _>>()?;

    Ok(records)
}

#[derive(Debug, serde::Deserialize)]
struct CashRecord {
    #[serde(rename = "SettleDate")]
    date: NaiveDate,
    #[serde(rename = "Amount")]
    amount: f64,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Symbol")]
    symbol: String,
}

#[cfg(test)]
mod tests {
    use crate::banks::Parser;

    const TRANSACTIONS_BAD: &str = r#"Trade date;Trade time;Booking date;Value date;Currency;Debit;Credit;Individual amount;Balance;Transaction no.;Description1;Description2;Description3;Footnotes;"#;
    const TRANSACTIONS: &str = r#""HEADER","TRNT","Symbol","Description","ISIN","CurrencyPrimary","Quantity","TradePrice","TradeDate"
"DATA","TRNT","VT","VANGUARD TOT WORLD STK ETF","US9220427424","USD","100","113","2024-06-24"
"HEADER","CTRN","Symbol","Description","ISIN","Amount","Type","SettleDate"
"DATA","CTRN","AAPL","AAPL(US0378331005) CASH DIVIDEND USD 0.24 PER SHARE - US TAX","US0378331005","-0.72","Withholding Tax","2024-02-15"
"DATA","CTRN","VT","VT(US9220427424) CASH DIVIDEND USD 0.4212 PER SHARE - US TAX","US9220427424","-16.03","Withholding Tax","2024-03-20"
"DATA","CTRN","AAPL","AAPL(US0378331005) CASH DIVIDEND USD 0.25 PER SHARE - US TAX","US0378331005","-0.75","Withholding Tax","2024-05-16"
"DATA","CTRN","GOOGL","GOOGL(US02079K3059) CASH DIVIDEND USD 0.20 PER SHARE - US TAX","US02079K3059","-0.6","Withholding Tax","2024-06-17"
"DATA","CTRN","VT","VT(US9220427424) CASH DIVIDEND USD 0.5779 PER SHARE - US TAX","US9220427424","-22","Withholding Tax","2024-06-25"
"DATA","CTRN","","CASH RECEIPTS / ELECTRONIC FUND TRANSFERS","","69980","Deposits/Withdrawals","2024-06-20"
"#;

    #[test]
    fn parse() {
        insta::assert_debug_snapshot!(
            super::Ibkr::parse("IBKR".to_string(), TRANSACTIONS.into()).unwrap()
        );
    }

    #[test]
    #[should_panic(
        expected = "The data seems to not be in IBKR format as no HEADER lines were found"
    )]
    fn parse_fail() {
        insta::assert_debug_snapshot!(super::Ibkr::parse(
            "IBKR".to_string(),
            TRANSACTIONS_BAD.into()
        )
        .unwrap());
    }
}
