import { describe, expect, it } from "vitest";
import { adjustRemainingAfterSync } from "./totals";

describe("adjustRemainingAfterSync", () => {
  it("decrements the new bucket when the previous totals had new cards", () => {
    const out = adjustRemainingAfterSync(
      { new: 2, learning: 0, review: 0 },
      { new: 5, learning: 1, review: 1 },
    );
    expect(out).toEqual({ new: 4, learning: 1, review: 1 });
  });

  it("falls through new → learning → review priority", () => {
    const out = adjustRemainingAfterSync(
      { new: 0, learning: 3, review: 2 },
      { new: 0, learning: 4, review: 2 },
    );
    expect(out).toEqual({ new: 0, learning: 3, review: 2 });
  });

  it("decrements review when only review remains", () => {
    const out = adjustRemainingAfterSync(
      { new: 0, learning: 0, review: 1 },
      { new: 0, learning: 0, review: 3 },
    );
    expect(out).toEqual({ new: 0, learning: 0, review: 2 });
  });

  it("never goes negative when the synced bucket is already 0", () => {
    const out = adjustRemainingAfterSync(
      { new: 1, learning: 0, review: 0 },
      { new: 0, learning: 0, review: 0 },
    );
    expect(out).toEqual({ new: 0, learning: 0, review: 0 });
  });

  it("does nothing when previous totals were empty", () => {
    const out = adjustRemainingAfterSync(
      { new: 0, learning: 0, review: 0 },
      { new: 2, learning: 2, review: 2 },
    );
    expect(out).toEqual({ new: 2, learning: 2, review: 2 });
  });
});
