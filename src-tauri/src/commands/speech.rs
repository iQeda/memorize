use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

/// macOS のシステム音声合成 (`/usr/bin/say`) で `text` を読み上げる。
/// 直前に起動した `say` プロセスがあれば先に kill して、ボタン連打や
/// 自動オン時のカード切替で再生が重なるのを防ぐ。
///
/// osascript 経由 (`tell System Events to key code 53 using {option down}`)
/// で macOS の "選択項目を読み上げる" を起動する方式は、本番 DMG の
/// ad-hoc 署名 + Hardened Runtime 構成だと Apple Events 送信に
/// `com.apple.security.automation.apple-events` entitlement が必要で
/// 一切音が出なかった。`say` は子プロセス起動なので entitlement も
/// アクセシビリティ権限も不要で、システム音声 / 速度設定はそのまま使う。
#[tauri::command]
pub async fn start_speak_text(
    text: String,
    state: State<'_, AppState>,
) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        let mut guard = state.speech_proc.lock().await;
        if let Some(mut prev) = guard.take() {
            let _ = prev.kill();
            let _ = prev.wait();
        }
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok(());
        }
        let child = std::process::Command::new("/usr/bin/say")
            .arg(trimmed)
            .spawn()
            .map_err(|e| anyhow::anyhow!("spawn say failed: {e}"))?;
        *guard = Some(child);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (text, state);
    }
    Ok(())
}
