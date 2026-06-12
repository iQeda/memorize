/** deck_graph_stats の応答をチャートコンポーネントの入力形へ変換する
 *  純関数群。day は 0 が今日、負が過去 (rslib の慣習)。 */

import type { Bucket, ReviewsBucket } from "./types";

export type SeriesDef = { label: string; color: string; values: number[] };

/** answer_millis 用の人間可読な所要時間。 */
export function formatDuration(ms: number): string {
  if (ms < 60_000) return `${Math.round(ms / 1000)}s`;
  const mins = Math.round(ms / 60_000);
  if (mins < 60) return `${mins}m`;
  const hrs = Math.floor(mins / 60);
  return `${hrs}h ${mins % 60}m`;
}

/** reviews を StackedBarChart の列形式へ。最小 day〜0 の連続列に展開し、
 *  欠損日は 0 で埋める。 */
export function buildReviewsCols(reviews: ReviewsBucket[]): {
  cols: number;
  series: SeriesDef[];
  minDay: number;
} {
  const minDay = Math.min(0, ...reviews.map((r) => r.day));
  const cols = -minDay + 1;
  const series: SeriesDef[] = [
    { label: "Learn", color: "#f5a623", values: Array(cols).fill(0) },
    { label: "Relearn", color: "#e26d6d", values: Array(cols).fill(0) },
    { label: "Young", color: "#7ed87a", values: Array(cols).fill(0) },
    { label: "Mature", color: "#3aa050", values: Array(cols).fill(0) },
    { label: "Filtered", color: "#7c8aff", values: Array(cols).fill(0) },
  ];
  for (const r of reviews) {
    const i = r.day - minDay;
    if (i < 0 || i >= cols) continue;
    series[0].values[i] = r.learn;
    series[1].values[i] = r.relearn;
    series[2].values[i] = r.young;
    series[3].values[i] = r.mature;
    series[4].values[i] = r.filtered;
  }
  return { cols, series, minDay };
}

/** reviews を CalendarHeatmap の {day, total} 形式へ。 */
export function buildCalendarPerDay(
  reviews: ReviewsBucket[],
): { day: number; total: number }[] {
  return reviews.map((r) => ({
    day: r.day,
    total: r.learn + r.relearn + r.young + r.mature + r.filtered,
  }));
}

/** added バケットを StackedBarChart 用の連続列へ (欠損日 0 埋め)。 */
export function buildAddedCols(added: Bucket<number>[]): {
  cols: number;
  values: number[];
  minDay: number;
} {
  const minDay = Math.min(0, ...added.map((b) => b.key));
  const cols = -minDay + 1;
  const values = Array(cols).fill(0);
  for (const b of added) {
    const i = b.key - minDay;
    if (i >= 0 && i < cols) values[i] = b.value;
  }
  return { cols, values, minDay };
}
