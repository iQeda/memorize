use crate::error::{AppError, AppResult};

/// Send Cmd+J to the system. Nani (configured to a global Cmd+J hotkey)
/// will pick it up and look up the currently selected text.
///
/// macOS-only: relies on `pbcopy` and `osascript`. On any other platform
/// the command returns a clear error so callers can surface it to the
/// user instead of silently failing.
#[tauri::command]
pub async fn nani_lookup(word: String) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        impl_nani_lookup_macos(word)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = word;
        Err(AppError::Anyhow(anyhow::anyhow!(
            "Nani lookup is only supported on macOS"
        )))
    }
}

#[cfg(target_os = "macos")]
fn impl_nani_lookup_macos(word: String) -> AppResult<()> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    // Put the word on the system pasteboard so Nani can read it from there
    // even if it doesn't reach Nani via the active text selection.
    let mut pbcopy = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("pbcopy spawn failed: {e}")))?;
    pbcopy
        .stdin
        .as_mut()
        .ok_or_else(|| AppError::Anyhow(anyhow::anyhow!("pbcopy stdin unavailable")))?
        .write_all(word.as_bytes())
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("pbcopy write failed: {e}")))?;
    let pb_status = pbcopy
        .wait()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("pbcopy wait failed: {e}")))?;
    if !pb_status.success() {
        // Don't fire Cmd+J if the clipboard wasn't actually updated —
        // otherwise Nani looks up whatever was last on the pasteboard.
        return Err(AppError::Anyhow(anyhow::anyhow!(
            "pbcopy exited with {pb_status}"
        )));
    }

    // Synthesize Cmd+J at the OS level via AppleScript / System Events.
    // Use .output() (not .status()) so we can surface stderr — without it,
    // the most common failure ("not authorized to send keystrokes …
    // Accessibility permission missing") is invisible to the user.
    let output = Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to keystroke "j" using command down"#)
        .output()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("osascript failed: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Anyhow(anyhow::anyhow!(
            "osascript exited with {}: {}",
            output.status,
            stderr.trim()
        )));
    }
    Ok(())
}
