import { describe, expect, it, beforeEach } from "vitest";
import { collection } from "./collection.svelte";

describe("collection.ankiDesktopSuggestion", () => {
  beforeEach(() => {
    collection.ankiDesktopPath = null;
    collection.currentPath = null;
  });

  it("is null when no Anki Desktop collection was detected", () => {
    collection.ankiDesktopPath = null;
    collection.currentPath = null;
    expect(collection.ankiDesktopSuggestion).toBeNull();
  });

  it("returns the detected path when nothing is open yet (welcome screen)", () => {
    collection.ankiDesktopPath = "/Users/me/Anki2/User 1/collection.anki2";
    collection.currentPath = null;
    expect(collection.ankiDesktopSuggestion).toBe(
      "/Users/me/Anki2/User 1/collection.anki2",
    );
  });

  it("returns the detected path when a different collection is open", () => {
    collection.ankiDesktopPath = "/Users/me/Anki2/User 1/collection.anki2";
    collection.currentPath = "/Users/me/other/collection.anki2";
    expect(collection.ankiDesktopSuggestion).toBe(
      "/Users/me/Anki2/User 1/collection.anki2",
    );
  });

  it("is null when the detected collection is already the one open", () => {
    collection.ankiDesktopPath = "/Users/me/Anki2/User 1/collection.anki2";
    collection.currentPath = "/Users/me/Anki2/User 1/collection.anki2";
    expect(collection.ankiDesktopSuggestion).toBeNull();
  });
});
