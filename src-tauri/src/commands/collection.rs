use crate::error::AppResult;
use crate::state::AppState;
use anki::collection::CollectionBuilder;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn open_collection(
    path: String,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let mut guard = state.col.lock().await;
    if guard.is_some() {
        if let Some(col) = guard.take() {
            let _ = col.close(None);
        }
    }
    let path_buf = PathBuf::from(&path);
    let col = CollectionBuilder::new(&path_buf)
        .set_shared_progress_state(state.progress.clone())
        .build()?;
    *guard = Some(col);
    *state.col_path.lock().await = Some(path_buf);
    Ok(())
}

#[tauri::command]
pub async fn close_collection(state: State<'_, AppState>) -> AppResult<()> {
    let mut guard = state.col.lock().await;
    if let Some(col) = guard.take() {
        let _ = col.close(None);
    }
    *state.col_path.lock().await = None;
    Ok(())
}

#[tauri::command]
pub async fn is_open(state: State<'_, AppState>) -> AppResult<bool> {
    let guard = state.col.lock().await;
    Ok(guard.is_some())
}

#[derive(serde::Serialize, Debug)]
pub struct CollectionInfo {
    pub current_path: Option<String>,
    pub anki_desktop_path: Option<String>,
}

/// Return both the currently-open collection path and the standard
/// Anki Desktop collection location (so the UI can offer "switch to
/// the collection that's already syncing with AnkiWeb").
#[tauri::command]
pub async fn collection_info(state: State<'_, AppState>) -> AppResult<CollectionInfo> {
    let current_path = state
        .col_path
        .lock()
        .await
        .clone()
        .map(|p| p.to_string_lossy().to_string());

    // Anki Desktop default profile location per platform.
    let home = dirs::home_dir();
    let anki_desktop_candidate = home.map(|h| {
        if cfg!(target_os = "macos") {
            h.join("Library/Application Support/Anki2/User 1/collection.anki2")
        } else if cfg!(target_os = "windows") {
            h.join("AppData/Roaming/Anki2/User 1/collection.anki2")
        } else {
            h.join(".local/share/Anki2/User 1/collection.anki2")
        }
    });
    let anki_desktop_path = anki_desktop_candidate
        .filter(|p| p.exists())
        .map(|p| p.to_string_lossy().to_string());

    Ok(CollectionInfo {
        current_path,
        anki_desktop_path,
    })
}
