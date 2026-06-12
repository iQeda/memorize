import { describe, expect, it } from "vitest";
import { downloadPercent } from "./updater-progress";

describe("downloadPercent", () => {
  it("returns null while total is unknown", () => {
    expect(downloadPercent(1234, 0)).toBeNull();
    expect(downloadPercent(0, -1)).toBeNull();
  });

  it("rounds to the nearest integer percent", () => {
    expect(downloadPercent(0, 1000)).toBe(0);
    expect(downloadPercent(500, 1000)).toBe(50);
    expect(downloadPercent(333, 1000)).toBe(33);
    expect(downloadPercent(335, 1000)).toBe(34);
  });

  it("clamps over-receipt at 100", () => {
    expect(downloadPercent(1200, 1000)).toBe(100);
  });
});
