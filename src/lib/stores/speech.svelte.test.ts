import { describe, expect, it, beforeEach } from "vitest";
import {
  speech,
  DEFAULT_MAX_REPEAT,
  MAX_REPEAT_MIN,
  MAX_REPEAT_MAX,
} from "./speech.svelte";

describe("speech store — repeat", () => {
  beforeEach(() => {
    speech.repeat = false;
    speech.repeatCount = 0;
    speech.repeatOnQuestionStart = false;
    speech.maxRepeat = DEFAULT_MAX_REPEAT;
  });

  it("DEFAULT_MAX_REPEAT is 5 (matches the user-facing spec)", () => {
    expect(DEFAULT_MAX_REPEAT).toBe(5);
    expect(speech.maxRepeat).toBe(5);
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

  it("setMaxRepeat clamps values to the documented range", () => {
    speech.setMaxRepeat(3);
    expect(speech.maxRepeat).toBe(3);

    // 上限超過 → MAX_REPEAT_MAX で頭打ち
    speech.setMaxRepeat(MAX_REPEAT_MAX + 10);
    expect(speech.maxRepeat).toBe(MAX_REPEAT_MAX);

    // 下限未満 / 不正値 → MAX_REPEAT_MIN
    speech.setMaxRepeat(0);
    expect(speech.maxRepeat).toBe(MAX_REPEAT_MIN);
    speech.setMaxRepeat(-7);
    expect(speech.maxRepeat).toBe(MAX_REPEAT_MIN);

    // 非整数 → round
    speech.setMaxRepeat(4.4);
    expect(speech.maxRepeat).toBe(4);
    speech.setMaxRepeat(4.6);
    expect(speech.maxRepeat).toBe(5);
  });
});
