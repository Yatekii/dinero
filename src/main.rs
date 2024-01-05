mod app;
mod cli;
mod error;
mod fx_store;
mod parser;
mod portfolio;

use axum::{routing::get, Router};
use clap::Parser;
use cli::Import;
use polars::lazy::{dsl::col, frame::IntoLazy};
use polars::prelude::*;
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::app::{data_handler, index_handler};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let provider = yahoo::YahooConnector::new();
    // let start = datetime!(2020-1-1 0:00:00.00 UTC);
    // let end = datetime!(2020-1-31 23:59:59.99 UTC);
    // // returns historic quotes with daily interval
    // let resp = provider
    //     .get_quote_history("AAPL", start, end)
    //     .await
    //     .unwrap();
    // let quotes = resp.quotes().unwrap();
    // println!("Apple's quotes in January: {:?}", quotes);

    match cli::Args::parse().command {
        cli::Command::Import(args) => import(args).await?,
        cli::Command::Serve(_args) => serve().await?,
    }

    Ok(())
}

async fn serve() -> anyhow::Result<()> {
    let portfolio = portfolio::Portfolio::from_file("portfolio/portfolio.yaml")?;

    // build our application with a route
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/data", get(data_handler))
        .with_state(Arc::new(portfolio));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn import(
    Import {
        name,
        path,
        currency,
        format,
    }: Import,
) -> anyhow::Result<()> {
    let mut portfolio = portfolio::Portfolio::from_file("portfolio/portfolio.yaml")?;

    for path in path {
        let mut incoming = parser::parse(name.clone(), path, currency.clone(), format.clone())?;

        if let Some(account) = portfolio
            .accounts
            .iter_mut()
            .find(|a| a.name == incoming.name && a.currency == incoming.currency)
        {
            account.transactions.df = concat(
                [
                    account.transactions.df.clone().lazy(),
                    incoming.transactions.df.clone().lazy().reverse().select(&[
                        col("Date"),
                        col("Amount"),
                        col("Amount").alias("Balance"),
                    ]),
                ],
                UnionArgs::default(),
            )?
            .select(&[
                col("Date"),
                col("Amount"),
                col("Amount").cum_sum(false).alias("Balance"),
            ])
            .collect()?;
        } else {
            incoming.transactions.df = incoming
                .transactions
                .df
                .lazy()
                .reverse()
                .select(&[
                    col("Date"),
                    col("Amount"),
                    col("Amount").cum_sum(false).alias("Balance"),
                ])
                .collect()?;
            incoming.transactions.path = format!(
                "portfolio/{}.parquet",
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()
            );
            portfolio.accounts.push(incoming);
        }
    }
    serde_yaml::to_writer(
        std::fs::File::create("portfolio/portfolio.yaml")?,
        &portfolio,
    )?;

    Ok(())
}
