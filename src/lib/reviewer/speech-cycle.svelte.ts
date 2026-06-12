/** リピート再生サイクル管理。repeatTimer / lastSpokenFrame / repeatCount の
 *  絡みを 1 クラスに集約する。setTimeout のハンドルは speech store ではなく
 *  Reviewer ローカル (このインスタンス) に置く: Reviewer を離れたら確実に
 *  止めたいから。カウンタ (repeatCount) は speech store 側 — popover の
 *  チェックボックスなど UI が直接読むため。 */

import { speech } from "$lib/stores/speech.svelte";

export type SpeechFinishedOutcome = "idle" | "max-reached" | "scheduled";

export class SpeechCycle {
  private timer: ReturnType<typeof setTimeout> | null = null;
  private lastFrame: HTMLIFrameElement | null = null;

  /** speak は実際の再生 (iframe からテキスト抽出 → start_speak_text)。 */
  constructor(private speak: (frame: HTMLIFrameElement) => void) {}

  /** 新サイクル開始: 進行中の setTimeout を捨て、count=1 から数え直す。 */
  start(frame: HTMLIFrameElement): void {
    this.cancelTimer();
    speech.repeatCount = 1;
    this.lastFrame = frame;
    this.speak(frame);
  }

  /** `memorize://speech-finished` 受信時に呼ぶ。
   *  - リピート OFF / 再生元 frame 不明 → "idle" (何もしない)
   *  - 最大回数到達 → "max-reached" (呼び出し側が auto-reveal 判定に使う)
   *  - それ以外 → interval 後の再再生を予約して "scheduled" */
  onSpeechFinished(): SpeechFinishedOutcome {
    if (!speech.repeat) return "idle";
    if (speech.repeatCount >= speech.maxRepeat) return "max-reached";
    const frame = this.lastFrame;
    if (!frame) return "idle";
    this.cancelTimer();
    this.timer = setTimeout(() => {
      this.timer = null;
      speech.repeatCount += 1;
      this.speak(frame);
    }, speech.repeatIntervalSec * 1000);
    return "scheduled";
  }

  /** 進行中の再再生予約をキャンセルする (カード切替 / unmount / リピート OFF)。
   *  repeatCount は触らない — リセットの要否は呼び出し側のセマンティクス。 */
  cancelTimer(): void {
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
    }
  }
}
