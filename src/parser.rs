mod neon;
mod ubs;

use std::path::Path;

use anyhow::Result;

use crate::{cli::Format, portfolio::Account};

pub fn parse(
    name: String,
    path: impl AsRef<Path>,
    currency: String,
    format: Format,
) -> Result<Account> {
    let path = path.as_ref().to_path_buf();
    match format {
        Format::Neon => neon::Parser::new(path, name.clone(), currency.clone()).parse(),
        Format::Ubs => ubs::Parser::new(path, name.clone(), currency.clone()).parse(),
    }
}
