import { describe, expect, it, beforeEach } from "vitest";
import {
  speech,
  DEFAULT_MAX_REPEAT,
  MAX_REPEAT_MIN,
  MAX_REPEAT_MAX,
  DEFAULT_REPEAT_INTERVAL_SEC,
  REPEAT_INTERVAL_MIN,
  REPEAT_INTERVAL_MAX,
  DEFAULT_SPEECH_RATE_WPM,
  SPEECH_RATE_MIN,
  SPEECH_RATE_MAX,
} from "./speech.svelte";

describe("speech store — repeat", () => {
  beforeEach(() => {
    speech.repeat = false;
    speech.repeatCount = 0;
    speech.repeatOnQuestionStart = false;
    speech.maxRepeat = DEFAULT_MAX_REPEAT;
    speech.repeatIntervalSec = DEFAULT_REPEAT_INTERVAL_SEC;
    speech.hideDefault = false;
  });

  it("DEFAULT_MAX_REPEAT is 3 (matches the user-facing spec)", () => {
    expect(DEFAULT_MAX_REPEAT).toBe(3);
    expect(speech.maxRepeat).toBe(3);
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

  it("DEFAULT_REPEAT_INTERVAL_SEC is 1", () => {
    expect(DEFAULT_REPEAT_INTERVAL_SEC).toBe(1);
    expect(speech.repeatIntervalSec).toBe(1);
  });

  it("setRepeatIntervalSec clamps to range and allows 0 (immediate replay)", () => {
    speech.setRepeatIntervalSec(2);
    expect(speech.repeatIntervalSec).toBe(2);

    // 小数 (0.01 刻み) を許容
    speech.setRepeatIntervalSec(0.5);
    expect(speech.repeatIntervalSec).toBe(0.5);
    speech.setRepeatIntervalSec(0.25);
    expect(speech.repeatIntervalSec).toBe(0.25);

    // 0 はポーズなしで有効
    speech.setRepeatIntervalSec(0);
    expect(speech.repeatIntervalSec).toBe(REPEAT_INTERVAL_MIN);

    // 上限超過 → REPEAT_INTERVAL_MAX
    speech.setRepeatIntervalSec(REPEAT_INTERVAL_MAX + 5);
    expect(speech.repeatIntervalSec).toBe(REPEAT_INTERVAL_MAX);

    // 負値 → 下限
    speech.setRepeatIntervalSec(-3);
    expect(speech.repeatIntervalSec).toBe(REPEAT_INTERVAL_MIN);

    // 0.01 刻みより細かい値は小数第 2 位で丸める
    speech.setRepeatIntervalSec(1.234);
    expect(speech.repeatIntervalSec).toBe(1.23);
    speech.setRepeatIntervalSec(1.236);
    expect(speech.repeatIntervalSec).toBe(1.24);
  });
});

describe("speech store — speech rate", () => {
  beforeEach(() => {
    speech.speechRate = DEFAULT_SPEECH_RATE_WPM;
  });

  it("DEFAULT_SPEECH_RATE_WPM defaults to ~180 (matches macOS voice default)", () => {
    expect(DEFAULT_SPEECH_RATE_WPM).toBeGreaterThanOrEqual(100);
    expect(DEFAULT_SPEECH_RATE_WPM).toBeLessThanOrEqual(300);
    expect(speech.speechRate).toBe(DEFAULT_SPEECH_RATE_WPM);
  });

  it("setSpeechRate clamps to range", () => {
    speech.setSpeechRate(200);
    expect(speech.speechRate).toBe(200);

    speech.setSpeechRate(SPEECH_RATE_MAX + 100);
    expect(speech.speechRate).toBe(SPEECH_RATE_MAX);

    speech.setSpeechRate(SPEECH_RATE_MIN - 100);
    expect(speech.speechRate).toBe(SPEECH_RATE_MIN);

    speech.setSpeechRate(220.7);
    expect(speech.speechRate).toBe(221);
  });
});

describe("speech store — hide default", () => {
  beforeEach(() => {
    speech.hideDefault = false;
  });

  it("starts with hideDefault off", () => {
    expect(speech.hideDefault).toBe(false);
  });

  it("setHideDefault updates the in-memory flag", () => {
    speech.setHideDefault(true);
    expect(speech.hideDefault).toBe(true);
    speech.setHideDefault(false);
    expect(speech.hideDefault).toBe(false);
  });

  it("setHideDefault roundtrips boolean values", () => {
    speech.setHideDefault(true);
    speech.setHideDefault(true);
    expect(speech.hideDefault).toBe(true);
    speech.setHideDefault(false);
    speech.setHideDefault(false);
    expect(speech.hideDefault).toBe(false);
  });
});
