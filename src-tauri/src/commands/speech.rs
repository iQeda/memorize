use crate::error::AppResult;

/// macOS の "選択項目を読み上げる" を起動する。System Events に Option+Esc
/// (key code 53 + option) のキーストロークを合成させることで、OS 標準ホット
/// キーを押下したのと同じ効果を得る。System Settings →アクセシビリティ→
/// 読み上げコンテンツでこの機能が有効化されており、かつアプリにアクセシビ
/// リティ権限が付与されていることが前提。他プラットフォームでは no-op。
#[tauri::command]
pub async fn start_speak_selection() -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        let script =
            r#"tell application "System Events" to key code 53 using {option down}"#;
        let status = std::process::Command::new("/usr/bin/osascript")
            .arg("-e")
            .arg(script)
            .status()
            .map_err(|e| anyhow::anyhow!("spawn osascript failed: {e}"))?;
        if !status.success() {
            return Err(anyhow::anyhow!("osascript exited with {status}").into());
        }
    }
    Ok(())
}
