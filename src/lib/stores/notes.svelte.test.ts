import { describe, expect, it, beforeEach, vi } from "vitest";

// Mock the IPC layer so the store can be exercised without Tauri.
// `vi.mock` is hoisted, so the factory must be self-contained — capture the
// mock fn through a getter exported by the mock module itself.
vi.mock("$lib/ipc", () => {
  const invoke = vi.fn();
  return { invoke };
});

import { invoke } from "$lib/ipc";
import { notes } from "./notes.svelte";

const mockInvoke = vi.mocked(invoke);

describe("notes store — setNoteDeck", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    notes.lastError = null;
    notes.busy = false;
  });

  it("invokes the set_note_deck command with snake_case input", async () => {
    mockInvoke.mockResolvedValueOnce(null);

    const ok = await notes.setNoteDeck({ noteId: 42, deckId: 7 });

    expect(ok).toBe(true);
    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(mockInvoke).toHaveBeenCalledWith("set_note_deck", {
      input: { note_id: 42, deck_id: 7 },
    });
    expect(notes.lastError).toBeNull();
  });

  it("stores stringified errors and returns false on failure", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("nope"));

    const ok = await notes.setNoteDeck({ noteId: 1, deckId: 2 });

    expect(ok).toBe(false);
    expect(notes.lastError).toContain("nope");
    // The busy flag must always settle back to false so the UI doesn't
    // permanently disable the picker after a single failed change.
    expect(notes.busy).toBe(false);
  });
});
