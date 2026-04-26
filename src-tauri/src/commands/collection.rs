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
    let col = CollectionBuilder::new(&path_buf).build()?;
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
