import { describe, expect, it } from "vitest";
import { keyAction, type KeyContext } from "./key-action";

const base: KeyContext = {
  meta: false,
  ctrl: false,
  key: "a",
  inTextField: false,
  editorOpen: false,
  hasCards: true,
  hasSelection: false,
};

describe("keyAction", () => {
  it("⌘A selects all when the list has cards and focus is not in a text field", () => {
    expect(keyAction({ ...base, meta: true, key: "a" })).toBe("selectAll");
  });

  it("⌘A while a checkbox is focused still selects all (checkbox is not a text field)", () => {
    // Regression: a checkbox has tagName INPUT but is not text entry.
    expect(keyAction({ ...base, meta: true, key: "a", inTextField: false })).toBe(
      "selectAll",
    );
  });

  it("⌘A is ignored when focus is in a text field (let it select text)", () => {
    expect(keyAction({ ...base, meta: true, key: "a", inTextField: true })).toBeNull();
  });

  it("⌘A is ignored when there are no cards", () => {
    expect(keyAction({ ...base, meta: true, key: "a", hasCards: false })).toBeNull();
  });

  it("Ctrl+A is ignored (Ctrl is not an accepted modifier)", () => {
    expect(keyAction({ ...base, meta: true, ctrl: true, key: "a" })).toBeNull();
  });

  it("uppercase A with ⌘ still selects all", () => {
    expect(keyAction({ ...base, meta: true, key: "A" })).toBe("selectAll");
  });

  it("Escape clears when a selection exists", () => {
    expect(keyAction({ ...base, key: "Escape", hasSelection: true })).toBe("clear");
  });

  it("Escape is ignored when nothing is selected", () => {
    expect(keyAction({ ...base, key: "Escape", hasSelection: false })).toBeNull();
  });

  it("any key is ignored while the editor is open", () => {
    expect(keyAction({ ...base, meta: true, key: "a", editorOpen: true })).toBeNull();
    expect(
      keyAction({ ...base, key: "Escape", hasSelection: true, editorOpen: true }),
    ).toBeNull();
  });

  it("returns null for unrelated keys", () => {
    expect(keyAction({ ...base, key: "x" })).toBeNull();
  });
});
