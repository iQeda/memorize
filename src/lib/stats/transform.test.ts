import { describe, expect, it } from "vitest";
import {
  buildAddedCols,
  buildCalendarPerDay,
  buildReviewsCols,
  formatDuration,
} from "./transform";
import type { ReviewsBucket } from "./types";

function review(day: number, partial: Partial<ReviewsBucket> = {}): ReviewsBucket {
  return { day, learn: 0, relearn: 0, young: 0, mature: 0, filtered: 0, ...partial };
}

describe("formatDuration", () => {
  it("formats sub-minute as seconds", () => {
    expect(formatDuration(0)).toBe("0s");
    expect(formatDuration(59_400)).toBe("59s");
  });

  it("formats minutes under an hour", () => {
    expect(formatDuration(60_000)).toBe("1m");
    expect(formatDuration(59 * 60_000)).toBe("59m");
  });

  it("formats hours with minute remainder", () => {
    expect(formatDuration(60 * 60_000)).toBe("1h 0m");
    expect(formatDuration(95 * 60_000)).toBe("1h 35m");
  });
});

describe("buildReviewsCols", () => {
  it("maps negative days into ascending columns ending at today (0)", () => {
    const out = buildReviewsCols([
      review(-2, { learn: 5 }),
      review(0, { mature: 3 }),
    ]);
    expect(out.minDay).toBe(-2);
    expect(out.cols).toBe(3);
    // Learn series: day -2 → index 0
    expect(out.series[0].values).toEqual([5, 0, 0]);
    // Mature series: day 0 → index 2
    expect(out.series[3].values).toEqual([0, 0, 3]);
  });

  it("zero-fills sparse days", () => {
    const out = buildReviewsCols([review(-4, { young: 1 }), review(-1, { young: 2 })]);
    expect(out.cols).toBe(5);
    expect(out.series[2].values).toEqual([1, 0, 0, 2, 0]);
  });

  it("empty input yields a single all-zero column (today)", () => {
    const out = buildReviewsCols([]);
    expect(out.minDay).toBe(0);
    expect(out.cols).toBe(1);
    for (const s of out.series) expect(s.values).toEqual([0]);
  });
});

describe("buildCalendarPerDay", () => {
  it("sums all five categories per day", () => {
    const out = buildCalendarPerDay([
      review(-1, { learn: 1, relearn: 2, young: 3, mature: 4, filtered: 5 }),
    ]);
    expect(out).toEqual([{ day: -1, total: 15 }]);
  });

  it("empty input stays empty", () => {
    expect(buildCalendarPerDay([])).toEqual([]);
  });
});

describe("buildAddedCols", () => {
  it("maps negative day keys to zero-filled columns", () => {
    const out = buildAddedCols([
      { key: -3, value: 7 },
      { key: 0, value: 2 },
    ]);
    expect(out.minDay).toBe(-3);
    expect(out.values).toEqual([7, 0, 0, 2]);
  });

  it("empty input yields a single zero column", () => {
    const out = buildAddedCols([]);
    expect(out.cols).toBe(1);
    expect(out.values).toEqual([0]);
  });
});
