use std::io::Cursor;

use polars::{
    io::{csv::CsvReader, SerReader},
    lazy::frame::{IntoLazy, LazyFrame},
};
use polars_core::chunked_array::ops::SortOptions;

pub struct Neon {}

impl Neon {
    pub fn parse(content: String) -> anyhow::Result<LazyFrame> {
        let df = CsvReader::new(Cursor::new(&content))
            .has_header(true)
            .with_try_parse_dates(true)
            .with_separator(b';')
            .finish()?
            .lazy()
            .sort(
                "Date",
                SortOptions {
                    descending: true,
                    nulls_last: false,
                    multithreaded: false,
                    maintain_order: false,
                },
            )
            .reverse()
            .drop_columns([
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
