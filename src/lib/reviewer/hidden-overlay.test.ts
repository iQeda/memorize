import { describe, expect, it } from "vitest";
import { setHiddenOverlay } from "./hidden-overlay";

function freshDoc(): Document {
  return new DOMParser().parseFromString("<body><p>card text</p></body>", "text/html");
}

describe("setHiddenOverlay", () => {
  it("adds the hidden class and overlay label when hiding", () => {
    const doc = freshDoc();
    setHiddenOverlay(doc, true, "Click or L to reveal");
    expect(doc.body.classList.contains("memorize-hidden")).toBe(true);
    const label = doc.getElementById("memorize-hidden-label");
    expect(label).not.toBeNull();
    expect(label!.textContent).toContain("[hidden mode]");
    expect(label!.textContent).toContain("Click or L to reveal");
  });

  it("removes the class and label when revealing", () => {
    const doc = freshDoc();
    setHiddenOverlay(doc, true, "hint");
    setHiddenOverlay(doc, false, "hint");
    expect(doc.body.classList.contains("memorize-hidden")).toBe(false);
    expect(doc.getElementById("memorize-hidden-label")).toBeNull();
  });

  it("is idempotent: repeated hide calls keep exactly one label", () => {
    const doc = freshDoc();
    setHiddenOverlay(doc, true, "hint");
    setHiddenOverlay(doc, true, "hint");
    setHiddenOverlay(doc, true, "hint");
    expect(doc.querySelectorAll("#memorize-hidden-label")).toHaveLength(1);
  });

  it("repeated reveal calls are safe no-ops", () => {
    const doc = freshDoc();
    setHiddenOverlay(doc, false, "hint");
    setHiddenOverlay(doc, false, "hint");
    expect(doc.getElementById("memorize-hidden-label")).toBeNull();
  });
});
