use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(about = "Add new data from a file")]
    Import(Import),
    #[command(about = "Serve the frontend")]
    Serve(Serve),
}

#[derive(Parser, Debug)]
pub struct Import {
    /// ID of the ledger
    #[arg(short, long)]
    pub id: String,
    /// Name of the ledger
    #[arg(short, long)]
    pub name: String,
    /// Path of the ledger
    #[arg(short, long)]
    pub path: Vec<PathBuf>,
    /// Currency of the ledger
    #[arg(short, long)]
    pub currency: String,
    /// The export format
    #[arg(short, long)]
    pub format: Format,
    /// The initial balance
    #[arg(short, long)]
    pub initial_balance: Option<f64>,
}

#[derive(Parser, Debug)]
pub struct Serve {}

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Ubs,
    Neon,
}
