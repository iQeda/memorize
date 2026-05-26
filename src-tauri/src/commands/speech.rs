use crate::error::AppResult;
use crate::state::AppState;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{AppHandle, Emitter, State};

/// Event name emitted when the most recent speech playback finishes naturally
/// (i.e. wasn't killed by a follow-up `start_speak_text` invocation). The
/// frontend listens for this to drive optional repeat playback.
pub const SPEECH_FINISHED_EVENT: &str = "memorize://speech-finished";

/// 1 サイクルあたり連番を振って `say -o` の出力 AIFF が衝突しないようにする。
/// 旧再生が cancel された直後に新再生が来た場合、旧 task が start_kill した
/// say -o がまだ AIFF を書き出している最中に新 task の say -o が同名ファイル
/// に上書き → afplay が中途半端な音を流す、を防ぐためファイル名を毎回変える。
static SPEECH_TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_temp_aiff_path() -> PathBuf {
    let n = SPEECH_TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("memorize-speech-{n}.aiff"))
}

/// macOS のシステム音声合成 (`/usr/bin/say`) で `text` を読み上げる。
/// 直前に起動した再生があれば先にキャンセル通知を送って kill し、
/// ボタン連打や自動オン時のカード切替で再生が重なるのを防ぐ。
///
/// osascript 経由 (`tell System Events to key code 53 using {option down}`)
/// で macOS の "選択項目を読み上げる" を起動する方式は、本番 DMG の
/// ad-hoc 署名 + Hardened Runtime 構成だと Apple Events 送信に
/// `com.apple.security.automation.apple-events` entitlement が必要で
/// 一切音が出なかった。`say` は子プロセス起動なので entitlement も
/// アクセシビリティ権限も不要で、システム音声 / 速度設定はそのまま使う。
///
/// `volume` の扱いは 2 系統に分かれる:
///   - 0..=100: `say` の `[[volm 0.0..1.0]]` を直接埋め込む (低レイテンシ)
///   - 101..=200: `say -o file.aiff` で一旦レンダーし、`afplay -v <gain>`
///     で増幅再生 (gain = volume/100、1.01..2.00 の範囲)。`[[volm]]` は
///     仕様上 1.0 が上限で 100% を超える増幅ができないため。
///
/// 自然終了時には `memorize://speech-finished` を emit する。キャンセル
/// (= 後続の start_speak_text に上書きされた) の場合は emit しないので、
/// フロント側のリピート再生ロジックが多重ループに陥らない。
#[tauri::command]
pub async fn start_speak_text(
    text: String,
    rate: Option<u32>,
    sentence_pause_ms: Option<u32>,
    volume: Option<u32>,
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

        let app_handle = app.clone();
        let route = pick_route(volume);
        match route {
            PlaybackRoute::Direct => {
                let mut cmd = tokio::process::Command::new("/usr/bin/say");
                for a in say_args(rate, volume, &processed) {
                    cmd.arg(a);
                }
                let mut child = cmd
                    .spawn()
                    .map_err(|e| anyhow::anyhow!("spawn say failed: {e}"))?;
                tokio::spawn(async move {
                    tokio::select! {
                        _ = child.wait() => {
                            let _ = app_handle.emit(SPEECH_FINISHED_EVENT, ());
                        }
                        _ = cancel_rx => {
                            let _ = child.start_kill();
                            let _ = child.wait().await;
                        }
                    }
                });
            }
            PlaybackRoute::Amplified { gain } => {
                let aiff_path = next_temp_aiff_path();
                // Stage 1: say -o file (テキストを AIFF にレンダー)。volume は
                // 渡さない (= フル音量レンダー)、後段の afplay -v で増幅する。
                let mut render_cmd = tokio::process::Command::new("/usr/bin/say");
                render_cmd.arg("-o").arg(&aiff_path);
                if let Some(r) = rate {
                    let clamped = r.clamp(100, 400);
                    render_cmd.arg("-r").arg(clamped.to_string());
                }
                render_cmd.arg(&processed);
                let mut render_child = render_cmd
                    .spawn()
                    .map_err(|e| anyhow::anyhow!("spawn say -o failed: {e}"))?;
                tokio::spawn(async move {
                    let mut cancel_rx = cancel_rx;
                    // Stage 1: レンダー完了 or キャンセル待ち
                    let render_ok = tokio::select! {
                        res = render_child.wait() => {
                            matches!(res, Ok(s) if s.success())
                        }
                        _ = &mut cancel_rx => {
                            let _ = render_child.start_kill();
                            let _ = render_child.wait().await;
                            let _ = tokio::fs::remove_file(&aiff_path).await;
                            return;
                        }
                    };
                    if !render_ok {
                        let _ = tokio::fs::remove_file(&aiff_path).await;
                        return;
                    }
                    // Stage 2: afplay -v gain file
                    let mut play_cmd = tokio::process::Command::new("/usr/bin/afplay");
                    play_cmd.arg("-v").arg(format!("{gain:.2}")).arg(&aiff_path);
                    let mut play_child = match play_cmd.spawn() {
                        Ok(c) => c,
                        Err(_) => {
                            let _ = tokio::fs::remove_file(&aiff_path).await;
                            return;
                        }
                    };
                    tokio::select! {
                        _ = play_child.wait() => {
                            let _ = app_handle.emit(SPEECH_FINISHED_EVENT, ());
                        }
                        _ = cancel_rx => {
                            let _ = play_child.start_kill();
                            let _ = play_child.wait().await;
                        }
                    }
                    let _ = tokio::fs::remove_file(&aiff_path).await;
                });
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (text, rate, sentence_pause_ms, volume, state, app);
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum PlaybackRoute {
    /// `say` で直接再生。volume が None または 0..=100 のとき。
    Direct,
    /// `say -o aiff` → `afplay -v gain` で増幅再生。volume が 101..=200。
    /// `gain` は 1.01..=2.00 の f32。
    Amplified { gain: f32 },
}

/// volume を見て再生ルートを決める。201 以上は 200 に clamp (afplay が
/// 高ゲインで割れた音を出すのを避けるため上限を設けている)。
fn pick_route(volume: Option<u32>) -> PlaybackRoute {
    match volume {
        Some(v) if v > 100 => {
            let clamped = v.min(200);
            PlaybackRoute::Amplified {
                gain: clamped as f32 / 100.0,
            }
        }
        _ => PlaybackRoute::Direct,
    }
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
///
/// `volume` (0-100) は `say` の埋め込みコマンド `[[volm <0.0..1.0>]]` を
/// テキスト先頭に差し込むことで実現する。`say` には `--volume` フラグが無く、
/// この埋め込みが公式に文書化された唯一の音量制御方法。`volume` が None
/// または 100 以上 (= フル音量) のときは何も挿入しない。100 超は
/// `[[volm]]` 仕様 (1.0 上限) で実現できないため Direct ルートでは
/// プレフィックスを省略するに留め、増幅は呼び出し側 (`pick_route`) が
/// Amplified ルートを選んで `afplay -v` 側でカバーする。
fn say_args(rate: Option<u32>, volume: Option<u32>, text: &str) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    if let Some(r) = rate {
        let clamped = r.clamp(100, 400);
        args.push("-r".to_string());
        args.push(clamped.to_string());
    }
    args.push(with_volume_prefix(volume, text));
    args
}

fn with_volume_prefix(volume: Option<u32>, text: &str) -> String {
    match volume {
        None => text.to_string(),
        Some(v) => {
            if v >= 100 {
                text.to_string()
            } else {
                let fraction = v as f32 / 100.0;
                format!("[[volm {fraction:.2}]] {text}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn say_args_without_rate_only_passes_text() {
        assert_eq!(say_args(None, None, "hello"), vec!["hello"]);
    }

    #[test]
    fn say_args_with_rate_inserts_r_flag_before_text() {
        assert_eq!(
            say_args(Some(200), None, "hello"),
            vec!["-r", "200", "hello"],
        );
    }

    #[test]
    fn say_args_clamps_rate_to_supported_range() {
        assert_eq!(say_args(Some(50), None, "x"), vec!["-r", "100", "x"]);
        assert_eq!(say_args(Some(9999), None, "x"), vec!["-r", "400", "x"]);
        assert_eq!(say_args(Some(100), None, "x"), vec!["-r", "100", "x"]);
        assert_eq!(say_args(Some(400), None, "x"), vec!["-r", "400", "x"]);
    }

    #[test]
    fn say_args_without_volume_passes_text_unchanged() {
        // None / 100 はいずれも voice 既定 (フル音量) と同じ挙動なので volm を埋め込まない。
        assert_eq!(say_args(None, None, "hello"), vec!["hello"]);
        assert_eq!(say_args(None, Some(100), "hello"), vec!["hello"]);
    }

    #[test]
    fn say_args_with_volume_prepends_volm_command() {
        assert_eq!(
            say_args(None, Some(50), "hello"),
            vec!["[[volm 0.50]] hello"],
        );
        assert_eq!(
            say_args(None, Some(0), "hello"),
            vec!["[[volm 0.00]] hello"],
        );
        assert_eq!(
            say_args(None, Some(25), "hello"),
            vec!["[[volm 0.25]] hello"],
        );
    }

    #[test]
    fn say_args_does_not_embed_volm_for_amplified_range() {
        // 100 超は Amplified ルートで afplay 側が増幅する役割なので、
        // Direct ルート用の say_args は volm を一切埋め込まない。
        assert_eq!(say_args(None, Some(101), "x"), vec!["x"]);
        assert_eq!(say_args(None, Some(150), "x"), vec!["x"]);
        assert_eq!(say_args(None, Some(200), "x"), vec!["x"]);
        assert_eq!(say_args(None, Some(u32::MAX), "x"), vec!["x"]);
    }

    #[test]
    fn say_args_with_volume_and_rate_combines_both() {
        // -r フラグと [[volm]] は両立する。volm はテキストに埋め込まれるだけなので
        // -r の後に来る text 引数の中身として現れる。
        assert_eq!(
            say_args(Some(180), Some(70), "hello"),
            vec!["-r", "180", "[[volm 0.70]] hello"],
        );
    }

    #[test]
    fn pick_route_returns_direct_for_none_and_le_100() {
        assert_eq!(pick_route(None), PlaybackRoute::Direct);
        assert_eq!(pick_route(Some(0)), PlaybackRoute::Direct);
        assert_eq!(pick_route(Some(50)), PlaybackRoute::Direct);
        assert_eq!(pick_route(Some(100)), PlaybackRoute::Direct);
    }

    #[test]
    fn pick_route_returns_amplified_with_gain_for_over_100() {
        match pick_route(Some(150)) {
            PlaybackRoute::Amplified { gain } => assert!((gain - 1.5).abs() < 1e-6),
            other => panic!("expected Amplified, got {other:?}"),
        }
        match pick_route(Some(101)) {
            PlaybackRoute::Amplified { gain } => assert!((gain - 1.01).abs() < 1e-6),
            other => panic!("expected Amplified, got {other:?}"),
        }
        match pick_route(Some(200)) {
            PlaybackRoute::Amplified { gain } => assert!((gain - 2.0).abs() < 1e-6),
            other => panic!("expected Amplified, got {other:?}"),
        }
    }

    #[test]
    fn pick_route_clamps_amplified_gain_to_2_0() {
        // 200 超は 200 に clamp。afplay の高ゲインは音が割れて実用性低なので上限固定。
        match pick_route(Some(500)) {
            PlaybackRoute::Amplified { gain } => assert!((gain - 2.0).abs() < 1e-6),
            other => panic!("expected Amplified, got {other:?}"),
        }
        match pick_route(Some(u32::MAX)) {
            PlaybackRoute::Amplified { gain } => assert!((gain - 2.0).abs() < 1e-6),
            other => panic!("expected Amplified, got {other:?}"),
        }
    }

    #[test]
    fn next_temp_aiff_path_returns_unique_paths() {
        // 同じプロセス内で次々呼ぶたびに別ファイル名 (連番末尾) を返す。
        let p1 = next_temp_aiff_path();
        let p2 = next_temp_aiff_path();
        assert_ne!(p1, p2);
        assert!(p1.file_name().unwrap().to_string_lossy().starts_with("memorize-speech-"));
        assert!(p1.extension().unwrap() == "aiff");
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
