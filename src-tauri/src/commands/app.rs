use crate::error::AppResult;
use crate::state::AppState;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, State};

/// Called by the frontend after it has run any pre-quit work (auto sync,
/// flush state). Flips the AppState latch so the next ExitRequested fires
/// straight through, then triggers the actual exit.
#[tauri::command]
pub async fn confirm_exit(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    state.allow_exit.store(true, Ordering::SeqCst);
    app.exit(0);
    Ok(())
}
