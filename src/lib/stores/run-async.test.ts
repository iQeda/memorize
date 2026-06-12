import { describe, expect, it } from "vitest";
import { runAsync, type BusyState } from "./run-async";

function freshState(): BusyState {
  return { busy: false, busyReason: null, lastError: null };
}

describe("runAsync", () => {
  it("returns the fn result and manages the busy lifecycle", async () => {
    const s = freshState();
    let busyDuringFn = false;
    let reasonDuringFn: string | null = null;
    const result = await runAsync(
      s,
      async () => {
        busyDuringFn = s.busy;
        reasonDuringFn = s.busyReason;
        return 42;
      },
      { reason: "working" },
    );
    expect(result).toBe(42);
    expect(busyDuringFn).toBe(true);
    expect(reasonDuringFn).toBe("working");
    expect(s.busy).toBe(false);
    expect(s.busyReason).toBeNull();
    expect(s.lastError).toBeNull();
  });

  it("clears a stale lastError before running fn", async () => {
    const s = freshState();
    s.lastError = "old error";
    let errorDuringFn: string | null = "sentinel";
    await runAsync(s, async () => {
      errorDuringFn = s.lastError;
    });
    expect(errorDuringFn).toBeNull();
  });

  it("swallows errors by default: returns null and records lastError", async () => {
    const s = freshState();
    const result = await runAsync(s, async () => {
      throw new Error("boom");
    });
    expect(result).toBeNull();
    expect(s.lastError).toContain("boom");
    expect(s.busy).toBe(false);
    expect(s.busyReason).toBeNull();
  });

  it("rethrows when opts.rethrow is set, still recording lastError", async () => {
    const s = freshState();
    await expect(
      runAsync(
        s,
        async () => {
          throw new Error("boom");
        },
        { rethrow: true },
      ),
    ).rejects.toThrow("boom");
    expect(s.lastError).toContain("boom");
    expect(s.busy).toBe(false);
    expect(s.busyReason).toBeNull();
  });

  it("resets busy even when fn throws synchronously-shaped rejections", async () => {
    const s = freshState();
    await runAsync(s, () => Promise.reject("plain string rejection"));
    expect(s.busy).toBe(false);
    expect(s.lastError).toBe("plain string rejection");
  });
});
