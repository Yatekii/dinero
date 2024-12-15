use std::{collections::HashMap, io::Cursor};

use anyhow::{bail, Error};
use chrono::NaiveDate;
use csv::{Reader, ReaderBuilder};

use crate::fx::{Currency, Symbol};

use super::{Ledger, LedgerRecord, ParsedAccount, Parser, StockLedgerRecord};

pub struct Ibkr {}

impl Parser for Ibkr {
    fn parse(name: &str, content: String) -> anyhow::Result<ParsedAccount> {
        let mut ledgers: Vec<Ledger> = vec![];
        let mut currency_ledger: Option<Ledger> = None;
        let lines = content.lines().collect::<Vec<_>>();
        let mut end = lines.len();
        let mut header_found = false;

        let currency = Currency::USD;

        // We scan the entire file from the back until we find a HEADER row that tells us what kind of
        // transactions the transactions following the header are.
        for (index, line) in lines.iter().enumerate().rev() {
            if line.starts_with("\"HEADER\"") {
                // Acknowledge that we found and entry, so this seems to be a valid IBKR format.
                header_found = true;
                let content = &lines[index..end].join("\n");
                let reader = ReaderBuilder::new()
                    .delimiter(b',')
                    .from_reader(Cursor::new(&content));

                // Extract the cash transactions.
                if line.contains("\"CTRN\"") {
                    let records = parse_cash_transactions(reader)?;
                    if let Some(ledger) = &mut currency_ledger {
                        ledger.records.extend(records);
                    } else {
                        currency_ledger = Some(super::Ledger {
                            name: name.to_string(),
                            records,
                            symbol: Symbol::Currency(currency),
                            kind: super::LedgerKind::Bank,
                        });
                    }
                }
                // Extract the trades.
                else if line.contains("\"TRNT\"") {
                    let data = parse_stock_transactions(reader)?;
                    for (symbol, records) in data {
                        // For each stock transaction we have an entry on the individual symbols ledger
                        // but also on the main currency ledger because it does not export the stock transactions.
                        // We need to deduce the amount of {main_currency} we paid for said stock.
                        let currency_records = records.iter().map(|r| LedgerRecord {
                            amount: r.amount * -1.0 * r.price,
                            date: r.date,
                            description: r.description.clone(),
                            category: r.category.clone(),
                        });
                        if let Some(ledger) = &mut currency_ledger {
                            ledger.records.extend(currency_records);
                        } else {
                            currency_ledger = Some(super::Ledger {
                                name: name.to_string(),
                                records: currency_records.collect(),
                                symbol: Symbol::Currency(currency),
                                kind: super::LedgerKind::Bank,
                            });
                        }

                        // We also need to add each record to the individual stock ledgers of course.
                        // But here we just add the number of shares, not the price we paid in {main_currency}.
                        let records = records.into_iter().map(From::from).collect();
                        if let Some(ledger) = ledgers.iter_mut().find(|l| l.symbol == symbol) {
                            ledger.records.extend(records);
                        } else {
                            ledgers.push(super::Ledger {
                                name: name.to_string(),
                                records,
                                symbol,
                                kind: super::LedgerKind::Stock,
                            });
                        }
                    }
                } else {
                    bail!("Unknown export")
                };

                end = index;
            }
        }

        if !header_found {
            bail!("The data seems to not be in IBKR format as no HEADER lines were found")
        }

        if let Some(ledeger) = currency_ledger {
            ledgers.push(ledeger);
        }

        Ok(ParsedAccount { ledgers })
    }
}

/// Gets all the stock purchases in the given reader.
///
/// Contains everything a regular transaction contains but also a stock price.
fn parse_stock_transactions(
    mut reader: Reader<Cursor<&&String>>,
) -> Result<HashMap<Symbol, Vec<StockLedgerRecord>>, Error> {
    let mut records = HashMap::new();
    let data = reader.deserialize::<StockRecord>();

    for record in data.flatten() {
        let symbol = Symbol::from(record.symbol);
        let entry = records.entry(symbol).or_insert(vec![]);
        entry.push(StockLedgerRecord {
            date: record.date,
            amount: record.amount,
            price: record.price,
            description: record.description,
            category: "Broker".to_string(),
        })
    }
    Ok(records)
}

#[derive(Debug, serde::Deserialize)]
struct StockRecord {
    #[serde(rename = "TradeDate")]
    date: NaiveDate,
    #[serde(rename = "Quantity")]
    amount: f64,
    #[serde(rename = "TradePrice")]
    price: f64,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Symbol")]
    symbol: String,
}

/// Parse all the cash transactions in the given reader.
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
    #[allow(unused)]
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
        insta::assert_debug_snapshot!(super::Ibkr::parse("IBKR", TRANSACTIONS.into()).unwrap());
    }

    #[test]
    #[should_panic(
        expected = "The data seems to not be in IBKR format as no HEADER lines were found"
    )]
    fn parse_fail() {
        super::Ibkr::parse("IBKR", TRANSACTIONS_BAD.into()).unwrap();
    }
}
