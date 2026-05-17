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
    sentence_pause_ms: Option<u32>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        let trimmed = text.trim().to_owned();
        if trimmed.is_empty() {
            return Ok(());
        }
        let pause = sentence_pause_ms.unwrap_or(0).min(5000);
        let processed = add_sentence_pauses(&trimmed, pause);

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
        for a in say_args(rate, &processed) {
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
        let _ = (text, rate, sentence_pause_ms, state, app);
    }
    Ok(())
}

/// テキスト中の文末記号 (`.` `!` `?` `。` `！` `？`) の後に
/// `[[slnc <ms>]]` の埋め込み無音コマンドを差し込む。`pause_ms == 0` は
/// 何もせず原文を返す (say の既定挙動)。`say` は `[[slnc N]]` を理解する。
fn add_sentence_pauses(text: &str, pause_ms: u32) -> String {
    if pause_ms == 0 {
        return text.to_string();
    }
    let marker = format!("[[slnc {pause_ms}]] ");
    let mut out = String::with_capacity(text.len() + 16);
    for ch in text.chars() {
        out.push(ch);
        if matches!(ch, '.' | '!' | '?' | '。' | '！' | '？') {
            out.push_str(&marker);
        }
    }
    out
}

/// `say` の引数配列を構築するピュア関数。`-r <wpm>` は 100-400 で clamp し、
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

    #[test]
    fn add_sentence_pauses_returns_input_unchanged_when_zero() {
        assert_eq!(
            add_sentence_pauses("Hello world. How are you?", 0),
            "Hello world. How are you?",
        );
    }

    #[test]
    fn add_sentence_pauses_inserts_slnc_after_english_terminators() {
        // Each of . ! ? should be followed by an embedded silence command.
        let out = add_sentence_pauses("One. Two! Three?", 500);
        assert_eq!(out, "One.[[slnc 500]]  Two![[slnc 500]]  Three?[[slnc 500]] ");
    }

    #[test]
    fn add_sentence_pauses_inserts_slnc_after_japanese_terminators() {
        let out = add_sentence_pauses("一。二！三？", 300);
        assert_eq!(out, "一。[[slnc 300]] 二！[[slnc 300]] 三？[[slnc 300]] ");
    }

    #[test]
    fn add_sentence_pauses_leaves_non_terminator_chars_alone() {
        assert_eq!(
            add_sentence_pauses("apple, banana, cherry", 200),
            "apple, banana, cherry",
        );
    }
}
