use polars::{
    datatypes::DataType,
    io::{csv::CsvReader, SerReader},
    lazy::{
        dsl::{col, lit},
        frame::{IntoLazy, LazyFrame},
    },
};

pub struct Ubs {}

impl Ubs {
    pub fn parse(content: String) -> anyhow::Result<LazyFrame> {
        let df = CsvReader::new(std::io::Cursor::new(&content))
            .has_header(true)
            .with_try_parse_dates(true)
            .with_separator(b';')
            .truncate_ragged_lines(true)
            .finish()?
            .lazy()
            .reverse()
            .select(&[
                col("Booking date").alias("Date"),
                col("Debit")
                    .fill_null(col("Credit"))
                    .alias("Amount")
                    .str()
                    .replace(lit("'"), lit(""), true)
                    .cast(DataType::Float64),
                col("Balance")
                    .str()
                    .replace(lit("'"), lit(""), true)
                    .cast(DataType::Float64),
                col("Description1").alias("Description"),
            ])
            .with_column(lit("").alias("Category"));

        Ok(df)
    }
}
