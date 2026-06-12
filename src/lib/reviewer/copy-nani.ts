/** Nani lookup フロー: iframe 本文を全選択 → クリップボードへコピー →
 *  Nani.app を発火する。 */

import { invoke } from "$lib/ipc";
import { extractCardText } from "./frame-text";

export type CopyOutcome =
  | { kind: "copied"; text: string }
  | { kind: "nothing" }
  | { kind: "error"; message: string };

export async function copyCardTextForNani(
  frame: HTMLIFrameElement | undefined,
): Promise<CopyOutcome> {
  const win = frame?.contentWindow;
  const doc = frame?.contentDocument;
  if (!win || !doc) return { kind: "nothing" };
  const host = doc.querySelector(".memorize-card-host");
  if (!host) return { kind: "nothing" };

  // iframe 内の本文 div を全範囲選択。removeAllRanges → addRange で
  // 既存の選択を上書きするだけで、その後の操作で解除はしない。
  const range = doc.createRange();
  range.selectNodeContents(host);
  const sel = win.getSelection();
  sel?.removeAllRanges();
  sel?.addRange(range);

  // iframe にフォーカスを移し selection を AX 的に「active」にする。
  // ユーザーが手動で Cmd+J を押した際に Nani.app が現在選択中のテキストを
  // 読み取れるのは、フォーカスが当たっている要素の selection だけなので
  // 必須。後続の c/1/2/3/4/Space などは srcdoc 内の key bridge が
  // parent window に再ディスパッチするため引き続き反応する。
  win.focus();

  const text = extractCardText(doc);
  if (!text) return { kind: "nothing" };

  let outcome: CopyOutcome;
  try {
    await navigator.clipboard.writeText(text);
    outcome = { kind: "copied", text };
  } catch (e) {
    console.error("clipboard write failed", e);
    outcome = { kind: "error", message: e instanceof Error ? e.message : String(e) };
  }
  // Nani.app (辞書 / 翻訳ランチャー) を発火する。Rust 側は
  // `naniapp://translate?source=<word>` deep link を `open` に渡すだけ
  // (commands/nani.rs 参照) なので、アクセシビリティ権限や Apple Events
  // entitlement は不要で本番ビルド (Hardened Runtime + ad-hoc 署名) でも動く。
  // Nani 未インストールなら open が失敗して console に出るだけで害はない。
  // コピー失敗時も従来どおり発火する。
  void invoke<void>("start_nani_lookup", { word: text }).catch((e) => {
    console.error("start_nani_lookup failed", e);
  });
  return outcome;
}
