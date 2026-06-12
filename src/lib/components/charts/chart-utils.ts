/** ホームの統計パネル用チャートの共有ジオメトリと y 軸 tick 計算。
 *
 *  注意: 定数は完全一致ではない — ButtonsChart は pad.r=6 / pad.b=24 で、
 *  inner() の override 引数で吸収する。 */

export const CHART_W = 720;
export const CHART_H = 140;

export type ChartPad = { l: number; r: number; t: number; b: number };

export const DEFAULT_PAD: ChartPad = { l: 28, r: 28, t: 6, b: 20 };

/** pad を解決し、描画領域の幅・高さを返す。 */
export function inner(pad?: Partial<ChartPad>): { w: number; h: number; pad: ChartPad } {
  const p = { ...DEFAULT_PAD, ...pad };
  return { w: CHART_W - p.l - p.r, h: CHART_H - p.t - p.b, pad: p };
}

/** 0..max を約 steps 分割した昇順 tick 列 (末尾は必ず max)。
 *  整数 step (最小 1) なので重複は出ない — チャートの keyed {#each} は
 *  それでも index キー (as v, i (i)) を維持すること (過去に疎データの
 *  duplicate-key クラッシュがあったため)。 */
export function tickValues(max: number, steps = 4): number[] {
  const step = Math.max(1, Math.ceil(max / steps));
  const out: number[] = [];
  for (let v = 0; v <= max; v += step) out.push(v);
  if (out[out.length - 1] !== max) out.push(max);
  return out;
}
