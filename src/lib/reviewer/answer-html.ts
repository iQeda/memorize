/** Anki テンプレートの answer 側は通常 `{{FrontSide}}<hr id=answer>{{Back}}`
 *  構造で、answer_html に質問部分が含まれる。フリップ後は答えだけ見せたいので
 *  hr#answer を境に前半 (質問) を削除した HTML を返す。hr#answer が無い
 *  カスタムテンプレートはそのまま返す (破壊しない)。 */
export function stripQuestionFromAnswer(html: string): string {
  const doc = new DOMParser().parseFromString(html, "text/html");
  const hr = doc.querySelector('hr#answer, hr[id="answer"]');
  if (!hr || !hr.parentElement) return html;
  const parent = hr.parentElement;
  while (parent.firstChild && parent.firstChild !== hr) {
    parent.removeChild(parent.firstChild);
  }
  parent.removeChild(hr);
  return doc.body.innerHTML;
}
