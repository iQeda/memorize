import { describe, expect, it, beforeEach } from "vitest";
import { speech, MAX_REPEAT } from "./speech.svelte";

describe("speech store — repeat", () => {
  beforeEach(() => {
    speech.repeat = false;
    speech.repeatCount = 0;
    speech.repeatOnQuestionStart = false;
  });

  it("MAX_REPEAT is 5 (matches the user-facing spec)", () => {
    // ハードコード値ではなく定数を介して読むことを Reviewer 側に強制したいので、
    // 定数の値そのものをテストで固定しておく。
    expect(MAX_REPEAT).toBe(5);
  });

  it("starts with repeat off and count zero", () => {
    expect(speech.repeat).toBe(false);
    expect(speech.repeatCount).toBe(0);
  });

  it("toggleRepeat flips the flag and resets count to 0", () => {
    speech.repeatCount = 3; // 仮の進行中状態
    speech.toggleRepeat();
    expect(speech.repeat).toBe(true);
    expect(speech.repeatCount).toBe(0);

    speech.repeatCount = 2;
    speech.toggleRepeat();
    expect(speech.repeat).toBe(false);
    expect(speech.repeatCount).toBe(0);
  });

  it("resetRepeatCount only zeroes the counter, not the repeat flag", () => {
    speech.repeat = true;
    speech.repeatCount = 4;
    speech.resetRepeatCount();
    expect(speech.repeat).toBe(true);
    expect(speech.repeatCount).toBe(0);
  });

  it("setRepeatOnQuestionStart updates the in-memory flag", () => {
    // ストアの constructor / setter は `if (browser)` で localStorage アクセスを
    // ガードしている。テスト時は `$app/environment` のモックが `browser=false` を
    // 返すので、ここでは in-memory の値だけが書き換わることを検証する。
    speech.setRepeatOnQuestionStart(true);
    expect(speech.repeatOnQuestionStart).toBe(true);
    speech.setRepeatOnQuestionStart(false);
    expect(speech.repeatOnQuestionStart).toBe(false);
  });
});
