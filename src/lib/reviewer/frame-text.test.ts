import { describe, expect, it, vi } from "vitest";
import { extractCardText, whenFrameReady } from "./frame-text";

function docWith(html: string): Document {
  return new DOMParser().parseFromString(html, "text/html");
}

describe("extractCardText", () => {
  it("extracts and whitespace-normalizes the host text", () => {
    const doc = docWith('<div class="memorize-card-host">  hello \n  world  </div>');
    expect(extractCardText(doc)).toBe("hello world");
  });

  it("returns null when the host is missing", () => {
    expect(extractCardText(docWith("<div>no host</div>"))).toBeNull();
  });

  it("returns null when the host is empty", () => {
    expect(extractCardText(docWith('<div class="memorize-card-host">   </div>'))).toBeNull();
  });
});

describe("whenFrameReady", () => {
  it("runs synchronously when the frame document is complete with a host", () => {
    const run = vi.fn();
    const frame = {
      contentDocument: docWith('<div class="memorize-card-host">x</div>'),
      addEventListener: vi.fn(),
    } as unknown as HTMLIFrameElement;
    // jsdom の DOMParser 製 document は readyState="complete"。
    whenFrameReady(frame, run);
    expect(run).toHaveBeenCalledOnce();
  });

  it("defers to the load event when the host is not present yet", () => {
    const run = vi.fn();
    const addEventListener = vi.fn();
    const frame = {
      contentDocument: docWith("<div>not ready</div>"),
      addEventListener,
    } as unknown as HTMLIFrameElement;
    whenFrameReady(frame, run);
    expect(run).not.toHaveBeenCalled();
    expect(addEventListener).toHaveBeenCalledWith("load", run, { once: true });
  });
});
