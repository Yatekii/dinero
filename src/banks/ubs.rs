use std::io::Cursor;

use polars::{
    datatypes::DataType,
    io::SerReader,
    lazy::{
        dsl::{col, lit},
        frame::{IntoLazy, LazyFrame},
    },
    prelude::*,
};

use super::Parser;

pub struct Ubs {}

impl Parser for Ubs {
    fn parse(content: String) -> anyhow::Result<LazyFrame> {
        let df = CsvReadOptions::default()
            .with_parse_options(
                CsvParseOptions::default()
                    .with_separator(b';')
                    .with_try_parse_dates(true)
                    .with_truncate_ragged_lines(true),
            )
            .with_has_header(true)
            .into_reader_with_file_handle(Cursor::new(&content))
            .finish()?;

        #[allow(clippy::possible_missing_comma)]
        let first = df
            .clone()
            .lazy()
            .select([
                col("Balance").alias("Credit").last()
                    - col("Debit")
                        .fill_null(col("Credit"))
                        .str()
                        .replace(lit("'"), lit(""), true)
                        .cast(DataType::Float64)
                        .last(),
                lit(NULL).cast(DataType::Float64).alias("Debit"),
                col("Booking date").last(),
                col("Description1").last(),
            ])
            .collect()
            .unwrap();

        let df = concat([first.lazy(), df.lazy().reverse()], UnionArgs::default())
            .unwrap()
            .select(&[
                col("Booking date").alias("Date"),
                col("Debit")
                    .fill_null(col("Credit"))
                    .alias("Amount")
                    .str()
                    .replace(lit("'"), lit(""), true)
                    .cast(DataType::Float64),
                col("Description1").alias("Description"),
            ])
            .with_column(lit("").alias("Category"));

        Ok(df)
    }
}

#[cfg(test)]
mod tests {
    use crate::banks::Parser;

    const TRANSACTIONS: &str = r#"Trade date;Trade time;Booking date;Value date;Currency;Debit;Credit;Individual amount;Balance;Transaction no.;Description1;Description2;Description3;Footnotes;
2021-01-29;;2021-01-29;2021-01-31;CHF;-10.00;;;22113.15;BL01529HJ0125142;"Balance closing of service prices";;"Transaction no. BL01529HJ0125142";;
2021-01-27;;2021-01-28;2021-01-28;CHF;-503.50;;;22123.15;9906527KH9626550;"UBS Switzerland AG,c/o UBS Card Center";"VIS1W WIDERSPRUCH AN UBS INNERT 30 TAGEN, direct debit";"Reference no. 70 03130 00000 00326 20405 08188, Account no. IBAN: CH25 0023 0230 0129 5305 U, Costs: LSV direct debit, Transaction no. 9906527KH9626550";;
2021-01-27;;2021-01-27;2021-01-27;CHF;-62.75;;;22626.65;9906026TO5927425;"CAISSE DES MEDECINS,1211 GENEVE";"e-banking payment order";"Reference no. 05 62945 01846 09123 50099 10841, Account no. 01-037005-8, Costs: E-Banking domestic, Transaction no. 9906026TO5927425";;
2021-01-22;;2021-01-22;2021-01-22;CHF;;9.59;;22689.40;9999022ZC8003633;"STRIPE PAYMENTS UK LTD,9TH FLOOR, 107 CHEAPSIDE GB - LONDO, N EC2V 6DN";"credit";"Reason for payment: GITHUB SPONSORS R8N0A2, Costs: Incoming SIC-payment, Transaction no. 9999022ZC8003633";;
2021-01-22;;2021-01-22;2021-01-22;CHF;;3688.00;;22679.81;9999022ZC7962684;"TECHNOKRAT GMBH,UNTERROHRSTRASSE 5, 8952 SCHLIEREN, CH";"credit";"Reason for payment: LOHN, Costs: Incoming SIC-payment, Transaction no. 9999022ZC7962684";;
2021-01-18;;2021-01-18;2021-01-18;CHF;-482.80;;;18991.81;BF21018DJ2743561;"CSS KRANKEN-VERSICHERUNG,AG, LUZERN";"EBILL-RECHNUNG, PayNet Order";"Reference no. 00 00002 64029 44001 71051 49861, Account no. 01-070393-3, Costs: E-Banking domestic, Transaction no. BF21018DJ2743561";;
2021-01-13;;2021-01-13;2021-01-13;CHF;-820.00;;;19474.61;9906012TI0715972;"Stichting DEGIRO,NL";"e-banking payment order";"Reason for payment: 88391-243702-5FFD9A1F-8BB5, Account no. IBAN: CH82 0871 0039 1145 1200 2, Costs: E-Banking domestic, Transaction no. 9906012TI0715972";;
2021-01-13;;2021-01-13;2021-01-13;CHF;-20.00;;;20294.61;9906013GK1225794;"ROTH, JONAS, Debit UBS TWINT";;"Reason for payment: +41798181674, TWINT-Acc.:+41799607130, Transaction no. 9906013GK1225794";;
2021-01-04;;2021-01-04;2021-01-04;CHF;-350.00;;;20314.61;9906501KH9243834;"GENERALI PERSONENVERSICHERUNGEN AG,8134 ADLISWIL";"FORT1 WIDERSPRUCH AN UBS INNERT 30 TAGEN, direct debit";"Reference no. 91 34065 09797 16210 10157 11525, Costs: LSV direct debit, Transaction no. 9906501KH9243834";;
2021-01-04;;2021-01-04;2021-01-04;CHF;-281.35;;;20664.61;BH21004DJ2411724;"CSS KRANKEN-VERSICHERUNG,AG, LUZERN";"EBILL-RECHNUNG, PayNet Order";"Reference no. 00 00002 64029 44002 90851 13989, Account no. 01-070393-3, Costs: E-Banking domestic, Transaction no. BH21004DJ2411724";;"#;

    #[test]
    fn parse() {
        let _df = super::Ubs::parse(TRANSACTIONS.into()).unwrap();
    }
}
