use anyhow::anyhow;
use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use itertools::Itertools;

use crate::{error::AppError, state::PortfolioAdapter};

use super::get::{LedgerFile, LedgerFiles};

#[debug_handler]
pub async fn handler(
    State(adapter): State<PortfolioAdapter>,
    Path((id, name)): Path<(String, String)>,
) -> Result<Json<LedgerFiles>, AppError> {
    adapter.delete_file(&id, &name)?;

    let files = adapter.list_files()?;
    let Some(paths) = files.get(&id) else {
        return Err(anyhow!("{id} was not found"))?;
    };
    let files = paths
        .iter()
        .map(|path| {
            let entries = adapter.load_file(&id, path);
            LedgerFile {
                filename: path.display().to_string(),
                number_of_entries: entries.as_ref().ok().map(|e| e.len()),
                error: entries.err().map(|e| e.chain().join("\n")),
            }
        })
        .collect();

    Ok(Json(LedgerFiles { id, files }))
}
