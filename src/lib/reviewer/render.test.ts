import { describe, expect, it } from "vitest";
import { setCardHtml } from "./render";

describe("setCardHtml", () => {
  it("replaces innerHTML with the new markup", async () => {
    const root = document.createElement("div");
    root.innerHTML = "<p>old</p>";
    await setCardHtml(root, "<p>new</p>");
    expect(root.innerHTML).toBe("<p>new</p>");
  });

  it("replaces inline <script> tags with fresh nodes so the browser will re-evaluate them", async () => {
    // jsdom's default mode does not actually execute injected scripts, so we
    // verify the structural contract instead: the original <script> node is
    // detached and replaced by a freshly-created one carrying the same body
    // and attributes. That swap is the only reason innerHTML-rendered
    // templates run their inline JS at all (see render.ts comment).
    const root = document.createElement("div");
    const html =
      '<span id="probe">x</span><' + 'script data-tag="t1">window.x = 1;</' + "script>";
    await setCardHtml(root, html);

    const scripts = Array.from(root.getElementsByTagName("script"));
    expect(scripts).toHaveLength(1);
    expect(scripts[0].getAttribute("data-tag")).toBe("t1");
    expect(scripts[0].innerHTML).toContain("window.x = 1;");
    expect(root.querySelector("#probe")?.textContent).toBe("x");
  });

  it("pauses and clears <video> elements before swapping in new HTML", async () => {
    const root = document.createElement("div");
    const video = document.createElement("video");
    let paused = false;
    video.pause = () => {
      paused = true;
    };
    video.appendChild(document.createElement("source"));
    root.appendChild(video);
    await setCardHtml(root, "<p>after</p>");
    expect(paused).toBe(true);
    expect(root.innerHTML).toBe("<p>after</p>");
  });
});
