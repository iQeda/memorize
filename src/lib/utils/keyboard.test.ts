import { describe, expect, it } from "vitest";
import { hasModifier, isTextFieldTarget } from "./keyboard";

describe("isTextFieldTarget", () => {
  it("returns true for input elements", () => {
    expect(isTextFieldTarget(document.createElement("input"))).toBe(true);
  });

  it("returns true for textarea elements", () => {
    expect(isTextFieldTarget(document.createElement("textarea"))).toBe(true);
  });

  it("returns true for select elements", () => {
    expect(isTextFieldTarget(document.createElement("select"))).toBe(true);
  });

  it("returns true for contentEditable elements", () => {
    const div = document.createElement("div");
    // jsdom does not implement isContentEditable (always undefined), so
    // stub the property the way a real browser would reflect it.
    Object.defineProperty(div, "isContentEditable", { value: true });
    expect(isTextFieldTarget(div)).toBe(true);
  });

  it("returns false for a plain div", () => {
    expect(isTextFieldTarget(document.createElement("div"))).toBe(false);
  });

  it("returns false for null", () => {
    expect(isTextFieldTarget(null)).toBe(false);
  });
});

describe("hasModifier", () => {
  it.each([
    ["metaKey", { metaKey: true }],
    ["ctrlKey", { ctrlKey: true }],
    ["altKey", { altKey: true }],
  ])("returns true when %s is pressed", (_name, init) => {
    expect(hasModifier(new KeyboardEvent("keydown", init))).toBe(true);
  });

  it("returns false with no modifiers", () => {
    expect(hasModifier(new KeyboardEvent("keydown", { key: "n" }))).toBe(false);
  });

  it("ignores shift (shift is not a routing modifier)", () => {
    expect(hasModifier(new KeyboardEvent("keydown", { shiftKey: true }))).toBe(false);
  });
});
