use anyhow::anyhow;
use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{error::AppError, handler::auth::user::User, state::PortfolioAdapter};

#[debug_handler(state = crate::state::AppState)]
pub async fn handler(
    State(adapter): State<PortfolioAdapter>,
    Path(id): Path<String>,
    user: User,
) -> Result<Json<LedgerFiles>, AppError> {
    let files = adapter.list_files(&user.sub)?;
    let Some(paths) = files.get(&id) else {
        return Err(anyhow!("{id} was not found"))?;
    };

    let files = paths
        .iter()
        .map(|path| {
            let entries = adapter.load_file(&user.sub, &id);
            LedgerFile {
                filename: path.display().to_string(),
                number_of_entries: entries.as_ref().ok().map(|e| e.len()),
                error: entries.err().map(|e| e.chain().join("\n")),
            }
        })
        .collect();

    Ok(Json(LedgerFiles { id, files }))
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct LedgerFiles {
    pub id: String,
    pub files: Vec<LedgerFile>,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct LedgerFile {
    pub filename: String,
    pub number_of_entries: Option<usize>,
    pub error: Option<String>,
}
