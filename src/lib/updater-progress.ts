/** アップデータのダウンロード進捗 (%) を計算する純関数。
 *  total が未知 (0 以下) のあいだは null を返し、UI 側は不定表示にする。 */
export function downloadPercent(received: number, total: number): number | null {
  if (total <= 0) return null;
  return Math.min(100, Math.round((received / total) * 100));
}
