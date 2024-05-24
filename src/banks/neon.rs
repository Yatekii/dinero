use std::io::Cursor;

use polars::{
    chunked_array::ops::SortMultipleOptions,
    io::{
        csv::read::{CsvParseOptions, CsvReadOptions},
        SerReader,
    },
    lazy::frame::{IntoLazy, LazyFrame},
};
use polars_plan::dsl::{col, lit};

pub struct Neon {}

impl Neon {
    pub fn parse(content: String) -> anyhow::Result<LazyFrame> {
        let df = CsvReadOptions::default()
            .with_parse_options(
                CsvParseOptions::default()
                    .with_separator(b';')
                    .with_try_parse_dates(true),
            )
            .with_has_header(true)
            .into_reader_with_file_handle(Cursor::new(&content))
            .finish()?
            .lazy()
            .filter(col("Spaces").eq(lit("no")))
            .sort(
                ["Date"],
                SortMultipleOptions {
                    descending: vec![true],
                    nulls_last: false,
                    multithreaded: false,
                    maintain_order: false,
                },
            )
            .reverse()
            .drop([
                "Original amount",
                "Original currency",
                "Exchange rate",
                "Exchange rate date",
                "Subject",
                "Tags",
                "Wise",
                "Spaces",
            ]);

        Ok(df)
    }
}
