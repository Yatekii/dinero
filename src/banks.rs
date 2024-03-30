mod neon;
mod ubs;
use anyhow::Result;
use polars::lazy::frame::LazyFrame;

use crate::cli::Format;

pub fn parse(content: String, format: Format) -> Result<LazyFrame> {
    match format {
        Format::Neon => neon::Neon::parse(content),
        Format::Ubs => ubs::Ubs::parse(content),
    }
}
