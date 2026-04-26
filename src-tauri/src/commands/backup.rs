use crate::error::{AppError, AppResult};
use crate::progress::ProgressEmitter;
use crate::state::AppState;
use anki::collection::CollectionBuilder;
use anki::import_export::package::import_colpkg as rslib_import_colpkg;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};

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
    let reopened = CollectionBuilder::new(&path)
        .set_shared_progress_state(state.progress.clone())
        .build()?;
    *state.col.lock().await = Some(reopened);

    result?;
    Ok(())
}

#[tauri::command]
pub async fn export_colpkg(
    out_path: String,
    include_media: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let _emitter = ProgressEmitter::start(app, state.progress.clone());
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
    app: AppHandle,
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

    let _emitter = ProgressEmitter::start(app, state.progress.clone());
    export_to(&state, &out_path, include_media).await?;
    Ok(AutoBackupResult {
        path: out_path.to_string_lossy().to_string(),
    })
}

/// Restore the currently-open collection from a .colpkg backup.
/// This OVERWRITES the local collection at its current path. The frontend
/// MUST present a destructive-action confirm dialog before invoking this.
#[tauri::command]
pub async fn import_colpkg(
    in_path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let _emitter = ProgressEmitter::start(app, state.progress.clone());
    let col_path = state
        .col_path
        .lock()
        .await
        .clone()
        .ok_or(AppError::CollectionNotOpen)?;
    let col_path_str = col_path
        .to_str()
        .ok_or_else(|| AppError::Anyhow(anyhow::anyhow!("non-utf8 collection path")))?
        .to_string();
    let media_folder = col_path.with_extension("media");
    let media_db = col_path.with_extension("mdb");

    // Take Collection out, grab a progress handle, then close to release file lock.
    let col = state
        .col
        .lock()
        .await
        .take()
        .ok_or(AppError::CollectionNotOpen)?;
    let progress = col.new_progress_handler();
    let _ = col.close(None);

    let import_result = rslib_import_colpkg(
        &in_path,
        &col_path_str,
        &media_folder,
        &media_db,
        progress,
    );

    // Re-open at the same path regardless of import success so the app
    // doesn't end up in a "collection closed" state with no recovery path.
    let reopened = CollectionBuilder::new(&col_path).build()?;
    *state.col.lock().await = Some(reopened);

    import_result?;
    Ok(())
}
