use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(about = "Serve the frontend")]
    Serve(Serve),
}

#[derive(Parser, Debug)]
pub struct Serve {}

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize, TS, Copy)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum BankFormat {
    Ubs,
    Neon,
    Ibkr,
}

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize, TS, Copy)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum BrokerFormat {
    Ibkr,
}
