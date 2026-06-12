import { describe, expect, it, beforeEach } from "vitest";
import { speech, SPEECH_LIMITS } from "./speech.svelte";

describe("speech store — repeat", () => {
  beforeEach(() => {
    speech.repeat = false;
    speech.repeatCount = 0;
    speech.repeatOnQuestionStart = false;
    speech.maxRepeat = SPEECH_LIMITS.maxRepeat.default;
    speech.repeatIntervalSec = SPEECH_LIMITS.repeatIntervalSec.default;
    speech.hideDefault = false;
  });

  it("SPEECH_LIMITS.maxRepeat.default is 3 (matches the user-facing spec)", () => {
    expect(SPEECH_LIMITS.maxRepeat.default).toBe(3);
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

    // 上限超過 → SPEECH_LIMITS.maxRepeat.max で頭打ち
    speech.setMaxRepeat(SPEECH_LIMITS.maxRepeat.max + 10);
    expect(speech.maxRepeat).toBe(SPEECH_LIMITS.maxRepeat.max);

    // 下限未満 / 不正値 → SPEECH_LIMITS.maxRepeat.min
    speech.setMaxRepeat(0);
    expect(speech.maxRepeat).toBe(SPEECH_LIMITS.maxRepeat.min);
    speech.setMaxRepeat(-7);
    expect(speech.maxRepeat).toBe(SPEECH_LIMITS.maxRepeat.min);

    // 非整数 → round
    speech.setMaxRepeat(4.4);
    expect(speech.maxRepeat).toBe(4);
    speech.setMaxRepeat(4.6);
    expect(speech.maxRepeat).toBe(5);
  });

  it("SPEECH_LIMITS.repeatIntervalSec.default is 1", () => {
    expect(SPEECH_LIMITS.repeatIntervalSec.default).toBe(1);
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
    expect(speech.repeatIntervalSec).toBe(SPEECH_LIMITS.repeatIntervalSec.min);

    // 上限超過 → SPEECH_LIMITS.repeatIntervalSec.max
    speech.setRepeatIntervalSec(SPEECH_LIMITS.repeatIntervalSec.max + 5);
    expect(speech.repeatIntervalSec).toBe(SPEECH_LIMITS.repeatIntervalSec.max);

    // 負値 → 下限
    speech.setRepeatIntervalSec(-3);
    expect(speech.repeatIntervalSec).toBe(SPEECH_LIMITS.repeatIntervalSec.min);

    // 0.01 刻みより細かい値は小数第 2 位で丸める
    speech.setRepeatIntervalSec(1.234);
    expect(speech.repeatIntervalSec).toBe(1.23);
    speech.setRepeatIntervalSec(1.236);
    expect(speech.repeatIntervalSec).toBe(1.24);
  });
});

describe("speech store — speech rate", () => {
  beforeEach(() => {
    speech.speechRate = SPEECH_LIMITS.rateWpm.default;
  });

  it("SPEECH_LIMITS.rateWpm.default defaults to 150 (slow-ish for listening practice)", () => {
    expect(SPEECH_LIMITS.rateWpm.default).toBe(150);
    expect(speech.speechRate).toBe(SPEECH_LIMITS.rateWpm.default);
  });

  it("setSpeechRate clamps to range", () => {
    speech.setSpeechRate(200);
    expect(speech.speechRate).toBe(200);

    speech.setSpeechRate(SPEECH_LIMITS.rateWpm.max + 100);
    expect(speech.speechRate).toBe(SPEECH_LIMITS.rateWpm.max);

    speech.setSpeechRate(SPEECH_LIMITS.rateWpm.min - 100);
    expect(speech.speechRate).toBe(SPEECH_LIMITS.rateWpm.min);

    speech.setSpeechRate(220.7);
    expect(speech.speechRate).toBe(221);
  });
});

describe("speech store — sentence pause", () => {
  beforeEach(() => {
    speech.sentencePauseMs = SPEECH_LIMITS.sentencePauseMs.default;
  });

  it("defaults to 500 ms (sentence-level pause for listening practice)", () => {
    expect(SPEECH_LIMITS.sentencePauseMs.default).toBe(500);
    expect(speech.sentencePauseMs).toBe(500);
  });

  it("setSentencePauseMs clamps to range and rounds to int", () => {
    speech.setSentencePauseMs(500);
    expect(speech.sentencePauseMs).toBe(500);

    speech.setSentencePauseMs(SPEECH_LIMITS.sentencePauseMs.max + 5000);
    expect(speech.sentencePauseMs).toBe(SPEECH_LIMITS.sentencePauseMs.max);

    speech.setSentencePauseMs(SPEECH_LIMITS.sentencePauseMs.min - 100);
    expect(speech.sentencePauseMs).toBe(SPEECH_LIMITS.sentencePauseMs.min);

    speech.setSentencePauseMs(123.6);
    expect(speech.sentencePauseMs).toBe(124);
  });
});

describe("speech store — auto reveal after repeat", () => {
  beforeEach(() => {
    speech.autoRevealAfterRepeat = false;
  });

  it("defaults to false", () => {
    expect(speech.autoRevealAfterRepeat).toBe(false);
  });

  it("setAutoRevealAfterRepeat updates the flag", () => {
    speech.setAutoRevealAfterRepeat(true);
    expect(speech.autoRevealAfterRepeat).toBe(true);
    speech.setAutoRevealAfterRepeat(false);
    expect(speech.autoRevealAfterRepeat).toBe(false);
  });
});

describe("speech store — volume", () => {
  beforeEach(() => {
    speech.volume = SPEECH_LIMITS.volume.default;
  });

  it("defaults to 100 (full volume, voice default)", () => {
    expect(SPEECH_LIMITS.volume.default).toBe(100);
    expect(speech.volume).toBe(100);
  });

  it("setVolume clamps to range and rounds to int", () => {
    speech.setVolume(50);
    expect(speech.volume).toBe(50);

    // 0 (mute) は有効
    speech.setVolume(0);
    expect(speech.volume).toBe(SPEECH_LIMITS.volume.min);

    // 上限超過 → 100
    speech.setVolume(SPEECH_LIMITS.volume.max + 50);
    expect(speech.volume).toBe(SPEECH_LIMITS.volume.max);

    // 負値 → 0
    speech.setVolume(-30);
    expect(speech.volume).toBe(SPEECH_LIMITS.volume.min);

    // 非整数 → round
    speech.setVolume(72.4);
    expect(speech.volume).toBe(72);
    speech.setVolume(72.6);
    expect(speech.volume).toBe(73);
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
