use crate::error::AppResult;

/// Nani.app (Cmd+J 起動の辞書ランチャー) で `word` を引く。
///
/// 流れ:
/// 1. `word` を pbcopy で pasteboard に置く
/// 2. `/usr/bin/open -a Nani` で Nani.app を起動
///
/// 当初は CGEvent (CGEventCreateKeyboardEvent + CGEventPost) で Cmd+J を
/// 合成する方針だったが、Memorize がフォーカスを持ったまま CGEvent を
/// post すると Nani のグローバルホットキーが拾わず Memorize ウィンドウに
/// 直接 keystroke が送られて何も起きないケースがあった。さらに CGEvent
/// post は本来アクセシビリティ権限が必要だが、macOS が prompt を出さず
/// silent fail することがあり実用的でない。
///
/// `open -a` は子プロセス起動だけで完結するので権限プロンプトも entitlement
/// も不要。Nani.app が「起動時に pasteboard を読み込んで自動検索する」仕様
/// に乗ることで、Cmd+J 経由と同等の体験を得る。Nani が未インストールなら
/// open がエラーを返すのでフロントの catch でログる。
#[tauri::command]
pub async fn start_nani_lookup(word: String) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        let trimmed = word.trim();
        if trimmed.is_empty() {
            return Ok(());
        }
        copy_to_pasteboard(trimmed)?;
        let status = std::process::Command::new("/usr/bin/open")
            .arg("-a")
            .arg("Nani")
            .status()
            .map_err(|e| anyhow::anyhow!("spawn open -a Nani: {e}"))?;
        if !status.success() {
            return Err(anyhow::anyhow!(
                "open -a Nani exited with {status} (Nani.app installed?)"
            )
            .into());
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = word;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn copy_to_pasteboard(text: &str) -> AppResult<()> {
    use std::io::Write;
    let mut child = std::process::Command::new("/usr/bin/pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("spawn pbcopy: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| anyhow::anyhow!("write pbcopy stdin: {e}"))?;
    }
    let status = child
        .wait()
        .map_err(|e| anyhow::anyhow!("wait pbcopy: {e}"))?;
    if !status.success() {
        return Err(anyhow::anyhow!("pbcopy exited with {status}").into());
    }
    Ok(())
}
