use chrono::NaiveDate;
use polars::{lazy::frame::LazyFrame, prelude::*};

pub fn process(
    data: LazyFrame,
    initial_balance: Option<f64>,
    initial_date: Option<NaiveDate>,
) -> anyhow::Result<DataFrame> {
    let incoming = data.clone().with_columns([
        col("Description").alias("description"),
        col("Category").alias("category"),
        lit("").alias("comments"),
        lit(false).alias("checked"),
    ]);

    let df = if let (Some(initial_balance), Some(initial_date)) = (initial_balance, initial_date) {
        let initial_description = "Initial Balance";
        let initial_category = "initial";
        let initial = df!(
            "Date" => [initial_date],
            "Amount" => [initial_balance],
            "Description" => [initial_description],
            "Category" => [initial_category],
            "Symbol" => [""]
        )?
        .lazy()
        .select(&[
            col("Date").cast(DataType::Date),
            col("Amount"),
            col("Description"),
            col("Category"),
            col("Symbol"),
        ]);

        concat([initial, incoming], UnionArgs::default())?
    } else {
        incoming
    }
    .group_by([col("Date"), col("Description"), col("Category")])
    .agg([
        col("Amount").sum(),
        col("*").exclude(["Amount"]),
        col("Date").count().alias("transactions"),
    ])
    .sort(
        ["Date"],
        SortMultipleOptions {
            descending: vec![false],
            nulls_last: false,
            multithreaded: true,
            maintain_order: true,
        },
    )
    .select(&[
        col("Date"),
        col("Amount"),
        col("Amount").cum_sum(false).alias("balance"),
        col("Description"),
        col("Category"),
        col("Description").alias("description"),
        col("Category").alias("category"),
        lit("").alias("comments"),
        lit(false).alias("checked"),
        col("transactions"),
        col("Symbol"),
    ])
    .collect()?;

    Ok(df)
}
