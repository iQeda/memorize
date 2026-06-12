/** 非表示モード: questionFrame の body class とオーバーレイラベルを直接
 *  付け外しする。CardFrame の srcdoc は theme/html/css の derived で、新 prop
 *  を入れると iframe がフルリロードされ speech が中断するため、prop は介さず
 *  contentDocument を直接操作する。ラベルは CSS ::after に頼ると iframe
 *  srcdoc が HMR や user CSS で古いまま/上書きされる懸念があるので、
 *  inline-style 付き <div> を body に挿入/撤去する。 */

const HIDDEN_LABEL_ID = "memorize-hidden-label";
const HIDDEN_LABEL_STYLE =
  "position:fixed; inset:0; display:flex; flex-direction:column; align-items:center; justify-content:center; gap:6px; " +
  "color:rgba(140,140,140,0.85); pointer-events:none; z-index:2147483647; visibility:visible;";

/** hidden 状態を doc に反映する。冪等 — 同じ状態で何度呼んでもラベルは 1 枚。 */
export function setHiddenOverlay(doc: Document, hidden: boolean, hintText: string): void {
  if (!doc.body) return;
  doc.body.classList.toggle("memorize-hidden", hidden);
  const existing = doc.getElementById(HIDDEN_LABEL_ID);
  if (hidden) {
    if (!existing) {
      const el = doc.createElement("div");
      el.id = HIDDEN_LABEL_ID;
      el.setAttribute("style", HIDDEN_LABEL_STYLE);
      const main = doc.createElement("div");
      main.textContent = "[hidden mode]";
      main.setAttribute("style", "font-size:0.95rem; letter-spacing:0.05em;");
      const hint = doc.createElement("div");
      hint.textContent = hintText;
      hint.setAttribute("style", "font-size:0.75rem; opacity:0.7;");
      el.appendChild(main);
      el.appendChild(hint);
      doc.body.appendChild(el);
    }
  } else if (existing) {
    existing.remove();
  }
}
