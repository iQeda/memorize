/** Sync 完了後の残数バッジ調整。`list_decks` は全件カウントを返すのに対し、
 *  ヘッダーの totals は「現在カードを除く残り」なので 1 枚分を引いて差を
 *  吸収する。現在カードの種類はフロントに渡っていないため、前回 totals に
 *  存在し、かつ今回の値が 0 でないカテゴリを 1 減算して推定する。 */

import type { Counts } from "./types";

export function adjustRemainingAfterSync(prev: Counts, next: Counts): Counts {
  const adjusted = { ...next };
  if (prev.new > 0 && adjusted.new > 0) adjusted.new -= 1;
  else if (prev.learning > 0 && adjusted.learning > 0) adjusted.learning -= 1;
  else if (prev.review > 0 && adjusted.review > 0) adjusted.review -= 1;
  return adjusted;
}
