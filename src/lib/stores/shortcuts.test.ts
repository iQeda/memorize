import { describe, expect, it, beforeEach } from "vitest";
import { shortcuts } from "./shortcuts.svelte";

describe("shortcuts store", () => {
  beforeEach(() => {
    shortcuts.reset();
  });

  it("starts with the documented defaults", () => {
    expect(shortcuts.keys.again).toBe("a");
    expect(shortcuts.keys.hard).toBe("s");
    expect(shortcuts.keys.good).toBe("d");
    expect(shortcuts.keys.easy).toBe("f");
    expect(shortcuts.keys.copy).toBe("j");
    expect(shortcuts.keys.speak).toBe("k");
  });

  it("ratingFor maps each default key to the corresponding rating", () => {
    expect(shortcuts.ratingFor("a")).toBe("again");
    expect(shortcuts.ratingFor("s")).toBe("hard");
    expect(shortcuts.ratingFor("d")).toBe("good");
    expect(shortcuts.ratingFor("f")).toBe("easy");
  });

  it("ratingFor returns null for unbound keys", () => {
    expect(shortcuts.ratingFor("z")).toBeNull();
    // Non-rating actions must not leak through ratingFor.
    expect(shortcuts.ratingFor("j")).toBeNull();
    expect(shortcuts.ratingFor("k")).toBeNull();
  });

  it("isCopy / isSpeak follow the bound key", () => {
    expect(shortcuts.isCopy("j")).toBe(true);
    expect(shortcuts.isCopy("k")).toBe(false);
    expect(shortcuts.isSpeak("k")).toBe(true);
    expect(shortcuts.isSpeak("j")).toBe(false);
  });

  it("set() rebinds an action and ratingFor reflects it", () => {
    shortcuts.set("again", "1");
    expect(shortcuts.keys.again).toBe("1");
    expect(shortcuts.ratingFor("1")).toBe("again");
    expect(shortcuts.ratingFor("a")).toBeNull();
  });

  it("label() formats special keys for the UI", () => {
    shortcuts.set("again", " ");
    expect(shortcuts.label("again")).toBe("Space");
    shortcuts.set("again", "Enter");
    expect(shortcuts.label("again")).toBe("↵");
    shortcuts.set("again", "Escape");
    expect(shortcuts.label("again")).toBe("Esc");
    shortcuts.set("again", "ArrowUp");
    expect(shortcuts.label("again")).toBe("↑");
    shortcuts.set("again", "a");
    expect(shortcuts.label("again")).toBe("A");
  });

  it("reset() restores defaults after rebinding", () => {
    shortcuts.set("again", "x");
    shortcuts.set("copy", "y");
    shortcuts.reset();
    expect(shortcuts.keys.again).toBe("a");
    expect(shortcuts.keys.copy).toBe("j");
  });
});
