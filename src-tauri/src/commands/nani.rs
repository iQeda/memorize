use crate::error::AppResult;

/// Nani.app (Cmd+J 起動の辞書 / 翻訳ランチャー) で `word` を引く。
///
/// `naniapp://translate?source=<URL-encoded word>` deep link を `open`
/// に渡すだけ。Nani の Info.plist 内 `CFBundleURLSchemes` に "naniapp"
/// が登録されており、deep link を開くと Nani が起動して `searchParams.get("source")`
/// を `trimmedText` として翻訳画面に流し込む (Nani app.asar 解析で確認)。
///
/// この方式の利点:
/// - CGEvent / Cmd+J 合成は不要 → アクセシビリティ権限プロンプトが出ない
/// - osascript / Apple Events は不要 → entitlement / Hardened Runtime 制約なし
/// - pbcopy も不要 → Nani が「現在の selection」を読まない実装でも確実に
///   検索対象を渡せる (前バージョンで pbcopy + open -a Nani 方式を試した
///   ところ、Nani は selection だけを読み pasteboard を見ない仕様だった)
#[tauri::command]
pub async fn start_nani_lookup(word: String) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        let trimmed = word.trim();
        if trimmed.is_empty() {
            return Ok(());
        }
        let url = format!("naniapp://translate?source={}", url_encode(trimmed));
        let status = std::process::Command::new("/usr/bin/open")
            .arg(&url)
            .status()
            .map_err(|e| anyhow::anyhow!("spawn open {url:?}: {e}"))?;
        if !status.success() {
            return Err(anyhow::anyhow!(
                "open {url:?} exited with {status} (Nani.app installed?)"
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

/// URL の query 値として安全な percent-encoding。RFC 3986 unreserved 以外を
/// すべて %HH に変換する。`urlencoding` crate を入れるほどの規模でもない
/// ので自前実装。
#[cfg(target_os = "macos")]
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}
