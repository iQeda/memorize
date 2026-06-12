import { describe, expect, it } from "vitest";
import { stripQuestionFromAnswer } from "./answer-html";

describe("stripQuestionFromAnswer", () => {
  it("drops everything before (and including) hr#answer", () => {
    const html = '<div class="q">civil</div><hr id="answer"><div class="a">polite</div>';
    const out = stripQuestionFromAnswer(html);
    expect(out).not.toContain("civil");
    expect(out).not.toContain("hr");
    expect(out).toContain("polite");
  });

  it("returns custom templates without hr#answer untouched", () => {
    const html = '<div class="q">front</div><div class="a">back</div>';
    expect(stripQuestionFromAnswer(html)).toBe(html);
  });

  it("ignores hr elements with other ids", () => {
    const html = '<div>front</div><hr id="divider"><div>back</div>';
    expect(stripQuestionFromAnswer(html)).toBe(html);
  });

  it("handles hr#answer nested inside a wrapper (uses its parent)", () => {
    const html = '<div class="card">q-part<hr id="answer">a-part</div>';
    const out = stripQuestionFromAnswer(html);
    expect(out).not.toContain("q-part");
    expect(out).toContain("a-part");
  });
});
