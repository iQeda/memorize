use crate::error::AppResult;

/// Nani.app (Cmd+J 起動の辞書ランチャー) でカード本文の単語を引く。
///
/// 流れ:
/// 1. `word` を pbcopy で pasteboard に置く (Nani が selection ではなく
///    pasteboard を読みに来るケースの保険)
/// 2. macOS の Quartz Event Services で Cmd+J を合成して post し、Nani の
///    グローバルホットキーを発火する
///
/// osascript + `tell System Events` 方式は Hardened Runtime + ad-hoc 署名 +
/// entitlement 空の本番 DMG では Apple Events 送信がブロックされて何も
/// 起きなかったため、CGEvent で直接キーストロークを post する方式に
/// 統一する。代わりに Memorize が「コンピュータの制御」(アクセシビリティ
/// 権限) を要求するプロンプトが初回に出るので、ユーザーは System Settings →
/// プライバシーとセキュリティ → アクセシビリティ で Memorize を許可する
/// 必要がある。
#[tauri::command]
pub async fn start_nani_lookup(word: String) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        let trimmed = word.trim();
        if trimmed.is_empty() {
            return Ok(());
        }
        copy_to_pasteboard(trimmed)?;
        post_cmd_j()?;
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

#[cfg(target_os = "macos")]
fn post_cmd_j() -> AppResult<()> {
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    // virtual keycode 38 = J on US keyboards. The mapping is
    // layout-independent: 38 means the physical key labeled "J" on a US
    // layout, which is what Nani's default Cmd+J binding listens for.
    const KEYCODE_J: core_graphics::event::CGKeyCode = 38;

    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| anyhow::anyhow!("CGEventSource::new failed"))?;
    let down = CGEvent::new_keyboard_event(source.clone(), KEYCODE_J, true)
        .map_err(|_| anyhow::anyhow!("CGEvent down failed"))?;
    down.set_flags(CGEventFlags::CGEventFlagCommand);
    down.post(CGEventTapLocation::HID);
    let up = CGEvent::new_keyboard_event(source, KEYCODE_J, false)
        .map_err(|_| anyhow::anyhow!("CGEvent up failed"))?;
    up.set_flags(CGEventFlags::CGEventFlagCommand);
    up.post(CGEventTapLocation::HID);
    Ok(())
}
