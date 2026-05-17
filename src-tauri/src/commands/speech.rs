use crate::error::AppResult;
use crate::state::AppState;
use tauri::{AppHandle, Emitter, State};

/// Event name emitted when the most recent `say` process finishes naturally
/// (i.e. wasn't killed by a follow-up `start_speak_text` invocation). The
/// frontend listens for this to drive optional repeat playback.
pub const SPEECH_FINISHED_EVENT: &str = "memorize://speech-finished";

/// macOS のシステム音声合成 (`/usr/bin/say`) で `text` を読み上げる。
/// 直前に起動した `say` プロセスがあれば先にキャンセル通知を送って kill し、
/// ボタン連打や自動オン時のカード切替で再生が重なるのを防ぐ。
///
/// osascript 経由 (`tell System Events to key code 53 using {option down}`)
/// で macOS の "選択項目を読み上げる" を起動する方式は、本番 DMG の
/// ad-hoc 署名 + Hardened Runtime 構成だと Apple Events 送信に
/// `com.apple.security.automation.apple-events` entitlement が必要で
/// 一切音が出なかった。`say` は子プロセス起動なので entitlement も
/// アクセシビリティ権限も不要で、システム音声 / 速度設定はそのまま使う。
///
/// 自然終了時には `memorize://speech-finished` を emit する。キャンセル
/// (= 後続の start_speak_text に上書きされた) の場合は emit しないので、
/// フロント側のリピート再生ロジックが多重ループに陥らない。
#[tauri::command]
pub async fn start_speak_text(
    text: String,
    rate: Option<u32>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        let trimmed = text.trim().to_owned();
        if trimmed.is_empty() {
            return Ok(());
        }

        // 旧再生があればキャンセル通知。Sender::send(()) は失敗しても問題ない
        // (Receiver が既に drop 済み = タスクは終了済みのケース)。
        let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();
        {
            let mut guard = state.speech_cancel.lock().await;
            if let Some(prev_tx) = guard.take() {
                let _ = prev_tx.send(());
            }
            *guard = Some(cancel_tx);
        }

        // 読み上げ速度 (wpm)。フロントが渡さなければ say の voice 既定値を使う。
        let mut cmd = tokio::process::Command::new("/usr/bin/say");
        for a in say_args(rate, &trimmed) {
            cmd.arg(a);
        }
        let mut child = cmd
            .spawn()
            .map_err(|e| anyhow::anyhow!("spawn say failed: {e}"))?;

        let app_handle = app.clone();
        tokio::spawn(async move {
            tokio::select! {
                // 自然終了 → finished イベントを送る
                _ = child.wait() => {
                    let _ = app_handle.emit(SPEECH_FINISHED_EVENT, ());
                }
                // 後続の start_speak_text に上書きされた → kill して wait のみ
                _ = cancel_rx => {
                    let _ = child.start_kill();
                    let _ = child.wait().await;
                }
            }
        });
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (text, rate, state, app);
    }
    Ok(())
}

/// `say` の引数配列を構築するピュア関数。`-r <wpm>` は 80-400 で clamp し、
/// rate が None なら省略 (voice の組み込み既定値が使われる)。
fn say_args(rate: Option<u32>, text: &str) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    if let Some(r) = rate {
        let clamped = r.clamp(100, 400);
        args.push("-r".to_string());
        args.push(clamped.to_string());
    }
    args.push(text.to_string());
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn say_args_without_rate_only_passes_text() {
        assert_eq!(say_args(None, "hello"), vec!["hello"]);
    }

    #[test]
    fn say_args_with_rate_inserts_r_flag_before_text() {
        assert_eq!(
            say_args(Some(200), "hello"),
            vec!["-r", "200", "hello"],
        );
    }

    #[test]
    fn say_args_clamps_rate_to_supported_range() {
        assert_eq!(say_args(Some(50), "x"), vec!["-r", "100", "x"]);
        assert_eq!(say_args(Some(9999), "x"), vec!["-r", "400", "x"]);
        assert_eq!(say_args(Some(100), "x"), vec!["-r", "100", "x"]);
        assert_eq!(say_args(Some(400), "x"), vec!["-r", "400", "x"]);
    }
}
