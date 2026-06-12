import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: vi.fn(() => ({
    startDragging: vi.fn().mockResolvedValue(undefined),
    toggleMaximize: vi.fn().mockResolvedValue(undefined),
  })),
}));

import { isInteractive } from "./draggable";

function mount(html: string): HTMLElement {
  const root = document.createElement("div");
  root.innerHTML = html;
  document.body.appendChild(root);
  return root;
}

afterEach(() => {
  document.body.innerHTML = "";
});

describe("isInteractive", () => {
  it("returns true for a button itself", () => {
    const root = mount("<button>x</button>");
    expect(isInteractive(root.querySelector("button"))).toBe(true);
  });

  it("returns true for an element nested inside a button", () => {
    const root = mount("<button><span><b>deep</b></span></button>");
    expect(isInteractive(root.querySelector("b"))).toBe(true);
  });

  it("returns true for anchors and form controls", () => {
    const root = mount(
      '<a href="#">link</a><input /><textarea></textarea><select></select><label>l</label>',
    );
    for (const sel of ["a", "input", "textarea", "select", "label"]) {
      expect(isInteractive(root.querySelector(sel)), sel).toBe(true);
    }
  });

  it("returns true when an ancestor has data-no-drag", () => {
    const root = mount('<div data-no-drag><span>inner</span></div>');
    expect(isInteractive(root.querySelector("span"))).toBe(true);
  });

  it('honors data-no-drag="true" as well as the bare attribute', () => {
    const root = mount('<div data-no-drag="true"><span>inner</span></div>');
    expect(isInteractive(root.querySelector("span"))).toBe(true);
  });

  it("returns false for a plain div", () => {
    const root = mount("<div><span>text</span></div>");
    expect(isInteractive(root.querySelector("span"))).toBe(false);
  });

  it("returns false for null", () => {
    expect(isInteractive(null)).toBe(false);
  });
});
