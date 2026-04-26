use crate::error::{AppError, AppResult};
use crate::state::AppState;
use anki::collection::CollectionBuilder;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::{Manager, State};

/// Export the currently-open collection as a .colpkg file at `out_path`.
/// Re-opens the same collection afterwards so the app continues working.
async fn export_to(state: &State<'_, AppState>, out_path: &Path, include_media: bool) -> AppResult<()> {
    let path = state
        .col_path
        .lock()
        .await
        .clone()
        .ok_or(AppError::CollectionNotOpen)?;

    // export_colpkg consumes the Collection (it closes it internally).
    let col = state
        .col
        .lock()
        .await
        .take()
        .ok_or(AppError::CollectionNotOpen)?;

    let result = col.export_colpkg(out_path, include_media, /* legacy */ false);

    // Re-open regardless of export success so the app stays usable.
    let reopened = CollectionBuilder::new(&path).build()?;
    *state.col.lock().await = Some(reopened);

    result?;
    Ok(())
}

#[tauri::command]
pub async fn export_colpkg(
    out_path: String,
    include_media: bool,
    state: State<'_, AppState>,
) -> AppResult<()> {
    export_to(&state, Path::new(&out_path), include_media).await
}

#[derive(Serialize, Debug)]
pub struct AutoBackupResult {
    pub path: String,
}

/// Save a timestamped backup into the default app backups dir.
/// Used both via manual button and (later) automatically before sync.
#[tauri::command]
pub async fn auto_backup(
    include_media: bool,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> AppResult<AutoBackupResult> {
    let backup_dir: PathBuf = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("app_data_dir: {e}")))?
        .join("backups");
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("mkdir backups: {e}")))?;

    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let out_path = backup_dir.join(format!("memorize-{stamp}.colpkg"));

    export_to(&state, &out_path, include_media).await?;
    Ok(AutoBackupResult {
        path: out_path.to_string_lossy().to_string(),
    })
}
