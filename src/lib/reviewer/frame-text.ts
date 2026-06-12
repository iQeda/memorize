/** CardFrame iframe との読み書きで 3 箇所に重複していた
 *  readyState / .memorize-card-host チェックとテキスト抽出の統合。 */

/** iframe document からカード本文テキストを抽出 (空白正規化済み)。
 *  host が無い・本文が空なら null。 */
export function extractCardText(doc: Document): string | null {
  const host = doc.querySelector(".memorize-card-host");
  if (!host) return null;
  const text = (host.textContent ?? "").trim().replace(/\s+/g, " ");
  return text || null;
}

/** iframe のロード完了 (= .memorize-card-host が存在) を待って run を実行。
 *  既にロード済みなら同期実行、未ロードなら load イベントに once で乗せる。 */
export function whenFrameReady(frame: HTMLIFrameElement, run: () => void): void {
  if (
    frame.contentDocument?.readyState === "complete" &&
    frame.contentDocument.querySelector(".memorize-card-host")
  ) {
    run();
  } else {
    frame.addEventListener("load", run, { once: true });
  }
}
