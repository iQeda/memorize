import { describe, expect, it } from "vitest";
import { CHART_H, CHART_W, DEFAULT_PAD, inner, tickValues } from "./chart-utils";

describe("inner", () => {
  it("uses the default pad when no override is given", () => {
    const { w, h, pad } = inner();
    expect(pad).toEqual(DEFAULT_PAD);
    expect(w).toBe(CHART_W - 28 - 28);
    expect(h).toBe(CHART_H - 6 - 20);
  });

  it("merges partial overrides (ButtonsChart: r=6, b=24)", () => {
    const { w, h, pad } = inner({ r: 6, b: 24 });
    expect(pad).toEqual({ l: 28, r: 6, t: 6, b: 24 });
    expect(w).toBe(CHART_W - 28 - 6);
    expect(h).toBe(CHART_H - 6 - 24);
  });
});

describe("tickValues", () => {
  it("max=1 gives [0, 1]", () => {
    expect(tickValues(1)).toEqual([0, 1]);
  });

  it("max=4 gives every integer", () => {
    expect(tickValues(4)).toEqual([0, 1, 2, 3, 4]);
  });

  it("max=5 appends the max after the stepped values", () => {
    expect(tickValues(5)).toEqual([0, 2, 4, 5]);
  });

  it("max=97 steps by 25 and ends exactly at max", () => {
    expect(tickValues(97)).toEqual([0, 25, 50, 75, 97]);
  });

  it("sparse data never produces duplicate ticks (historical keyed-each crash)", () => {
    // 過去に疎データで重複 tick → keyed {#each} の duplicate-key クラッシュ
    // が起きた。0..200 の全 max で重複なし・昇順・末尾 === max を検証する。
    for (let max = 0; max <= 200; max++) {
      const ticks = tickValues(max);
      expect(new Set(ticks).size, `max=${max}`).toBe(ticks.length);
      expect([...ticks].sort((a, b) => a - b), `max=${max}`).toEqual(ticks);
      expect(ticks[ticks.length - 1], `max=${max}`).toBe(max);
    }
  });
});
