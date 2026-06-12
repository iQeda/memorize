/** iframe 内の本文テキストを抽出して、macOS の `say` に渡して読み上げる。
 *  設定オン時の自動発火 (新カード Question 表示) と、Speak ボタン /
 *  k キーによる手動発火の両方で使う。osascript + Apple Events 方式は
 *  本番ビルド (ad-hoc + Hardened Runtime) で entitlement が無く動かない
 *  ため、子プロセス起動だけで完結する `say` 経由に統一している。 */

import { invoke } from "$lib/ipc";
import { speech } from "$lib/stores/speech.svelte";
import { extractCardText, whenFrameReady } from "./frame-text";

export function speakFrameText(frame: HTMLIFrameElement): void {
  whenFrameReady(frame, () => {
    const doc = frame.contentDocument;
    if (!doc) return;
    const text = extractCardText(doc);
    if (!text) return;
    void invoke<void>("start_speak_text", {
      text,
      rate: speech.speechRate,
      sentencePauseMs: speech.sentencePauseMs,
      volume: speech.volume,
    }).catch((e) => {
      console.error("start_speak_text failed", e);
    });
  });
}
