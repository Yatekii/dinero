mod neon;
mod ubs;
use std::path::Path;

use anyhow::{Context, Result};
use polars::{lazy::frame::LazyFrame, prelude::*};

use crate::cli::Format;

pub fn parse(content: String, format: Format) -> Result<LazyFrame> {
    match format {
        Format::Neon => neon::Neon::parse(content),
        Format::Ubs => ubs::Ubs::parse(content),
    }
}

pub fn load(path: impl AsRef<Path>, format: Format) -> anyhow::Result<LazyFrame> {
    match format {
        Format::Neon => load_inner::<neon::Neon>(path),
        Format::Ubs => load_inner::<ubs::Ubs>(path),
    }
}

fn load_inner<T: Parser>(path: impl AsRef<Path>) -> anyhow::Result<LazyFrame> {
    let mut df = None;
    for dir_entry in std::fs::read_dir(&path).with_context(|| {
        format!(
            "could not read ledger directory {}",
            path.as_ref().display()
        )
    })? {
        let dir_entry = dir_entry?;

        let path = dir_entry.path();
        let loaded = T::parse(
            std::fs::read_to_string(&path)
                .with_context(|| format!("could not read ledger CSV {}", path.display()))?,
        )?;

        df = Some(if let Some(df) = df {
            concat([df, loaded], UnionArgs::default())?
        } else {
            loaded
        });
    }
    Ok(df.expect("a dataframe"))
}

pub trait Parser {
    fn parse(content: String) -> Result<LazyFrame>;
}
