/** グローバルショートカット用のキーボードイベント述語。
 *
 *  注意: review ページ (`src/routes/review/[deckId]/+page.svelte`) の inline
 *  チェックは意図的にこれより狭い (input/textarea のみ + `editing` フラグ)
 *  ので、ここの広いバリアントに置き換えないこと。 */

/** フォーカス先がテキスト入力か (input / textarea / select / contentEditable)。 */
export function isTextFieldTarget(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null;
  if (!el) return false;
  const tag = el.tagName;
  return (
    tag === "INPUT" ||
    tag === "TEXTAREA" ||
    tag === "SELECT" ||
    el.isContentEditable === true
  );
}

/** meta / ctrl / alt のいずれかが押下されているか。 */
export function hasModifier(e: KeyboardEvent): boolean {
  return e.metaKey || e.ctrlKey || e.altKey;
}
