use crate::error::{AppError, AppResult};
use std::process::{Command, Stdio};

/// Send Cmd+J to the system. Nani (configured to a global Cmd+J hotkey)
/// will pick it up and look up the currently selected text.
#[tauri::command]
pub async fn nani_lookup(word: String) -> AppResult<()> {
    // Put the word on the system pasteboard so Nani can read it from there
    // even if it doesn't reach Nani via the active text selection.
    let mut pbcopy = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("pbcopy spawn failed: {e}")))?;
    if let Some(stdin) = pbcopy.stdin.as_mut() {
        use std::io::Write;
        stdin
            .write_all(word.as_bytes())
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("pbcopy write failed: {e}")))?;
    }
    let _ = pbcopy.wait();

    // Synthesize Cmd+J at the OS level via AppleScript / System Events.
    let status = Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to keystroke "j" using command down"#)
        .status()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("osascript failed: {e}")))?;
    if !status.success() {
        return Err(AppError::Anyhow(anyhow::anyhow!(
            "osascript exited with {status}"
        )));
    }
    Ok(())
}
