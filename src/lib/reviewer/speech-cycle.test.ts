import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SpeechCycle } from "./speech-cycle.svelte";
import { speech } from "$lib/stores/speech.svelte";

const FRAME = {} as HTMLIFrameElement;

beforeEach(() => {
  vi.useFakeTimers();
  speech.repeat = true;
  speech.repeatCount = 0;
  speech.maxRepeat = 3;
  speech.repeatIntervalSec = 1;
});

afterEach(() => {
  vi.useRealTimers();
});

describe("SpeechCycle", () => {
  it("start() speaks immediately and resets the counter to 1", () => {
    const speak = vi.fn();
    const cycle = new SpeechCycle(speak);
    speech.repeatCount = 5; // 前のカードの進行中状態
    cycle.start(FRAME);
    expect(speak).toHaveBeenCalledTimes(1);
    expect(speech.repeatCount).toBe(1);
  });

  it("replays after the configured interval and increments the counter", () => {
    const speak = vi.fn();
    const cycle = new SpeechCycle(speak);
    cycle.start(FRAME);

    expect(cycle.onSpeechFinished()).toBe("scheduled");
    expect(speak).toHaveBeenCalledTimes(1);
    // interval (1s) 未満では再生しない
    vi.advanceTimersByTime(999);
    expect(speak).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(1);
    expect(speak).toHaveBeenCalledTimes(2);
    expect(speech.repeatCount).toBe(2);
  });

  it("stops at maxRepeat and reports max-reached", () => {
    const speak = vi.fn();
    const cycle = new SpeechCycle(speak);
    cycle.start(FRAME); // count=1

    expect(cycle.onSpeechFinished()).toBe("scheduled");
    vi.runAllTimers(); // count=2
    expect(cycle.onSpeechFinished()).toBe("scheduled");
    vi.runAllTimers(); // count=3 (= maxRepeat)
    expect(cycle.onSpeechFinished()).toBe("max-reached");
    vi.runAllTimers();
    expect(speak).toHaveBeenCalledTimes(3);
    expect(speech.repeatCount).toBe(3);
  });

  it("does nothing when repeat is off", () => {
    const speak = vi.fn();
    const cycle = new SpeechCycle(speak);
    cycle.start(FRAME);
    speech.repeat = false;
    expect(cycle.onSpeechFinished()).toBe("idle");
    vi.runAllTimers();
    expect(speak).toHaveBeenCalledTimes(1);
  });

  it("cancelTimer() drops a pending replay", () => {
    const speak = vi.fn();
    const cycle = new SpeechCycle(speak);
    cycle.start(FRAME);
    cycle.onSpeechFinished();
    cycle.cancelTimer();
    vi.runAllTimers();
    expect(speak).toHaveBeenCalledTimes(1);
    expect(speech.repeatCount).toBe(1);
  });

  it("a new cycle restarts the count even mid-replay-wait", () => {
    const speak = vi.fn();
    const cycle = new SpeechCycle(speak);
    cycle.start(FRAME);
    cycle.onSpeechFinished(); // 再再生待ち
    cycle.start(FRAME); // 新カード: 待ちを破棄して count=1
    expect(speech.repeatCount).toBe(1);
    vi.runAllTimers();
    // 破棄された timer は発火せず、start の即時再生 2 回のみ。
    expect(speak).toHaveBeenCalledTimes(2);
  });

  it("ignores onSpeechFinished before any start (no frame yet)", () => {
    const cycle = new SpeechCycle(vi.fn());
    expect(cycle.onSpeechFinished()).toBe("idle");
  });
});
