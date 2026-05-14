import { browser } from "$app/environment";

const SPEAK_QUESTION_KEY = "memorize:speak-question-on-show";
const REPEAT_ON_START_KEY = "memorize:repeat-on-question-start";
const MAX_REPEAT_KEY = "memorize:max-repeat";

/** デフォルト最大連続再生回数 (1 回目を含む)。設定画面の数値入力で上書き可能。 */
export const DEFAULT_MAX_REPEAT = 3;
/** 設定 UI で受け付ける範囲。極端値は再生体験を壊すので 1..10 に clamp。 */
export const MAX_REPEAT_MIN = 1;
export const MAX_REPEAT_MAX = 10;

function clampMaxRepeat(n: number): number {
  if (!Number.isFinite(n)) return DEFAULT_MAX_REPEAT;
  return Math.min(MAX_REPEAT_MAX, Math.max(MAX_REPEAT_MIN, Math.round(n)));
}

class SpeechStore {
  speakQuestionOnShow = $state(false);
  /** 問題開始時に `repeat` を自動 ON にする永続設定。
   *  ON にしておくと Reviewer に入るたびにチェックボックスが入った状態で
   *  始まり、新カードの自動再生 (`speakQuestionOnShow`) と組み合わさって
   *  毎カード 5 回ループが自動的に走る。 */
  repeatOnQuestionStart = $state(false);
  /** 1 サイクルあたりの最大再生回数 (1 回目を含む)。永続化、デフォルト 5。 */
  maxRepeat = $state(DEFAULT_MAX_REPEAT);
  /** Reviewer 表示中のみ意味を持つセッションフラグ。永続化しない。
   *  ON のあいだ、`memorize://speech-finished` 受信時に 1 秒ポーズして
   *  同じテキストを再生し直す。 */
  repeat = $state(false);
  /** 現在のリピートサイクル内で何回再生したか (1 回目で 1)。`maxRepeat` 到達でループ停止。 */
  repeatCount = $state(0);

  constructor() {
    if (browser) {
      // Default: OFF. Stored "1" = ON. macOS の "選択項目を読み上げる" は
      // OS 設定とアクセシビリティ権限を要求するため、明示的なオプトイン。
      const stored = localStorage.getItem(SPEAK_QUESTION_KEY);
      if (stored === "1") this.speakQuestionOnShow = true;
      const storedRepeat = localStorage.getItem(REPEAT_ON_START_KEY);
      if (storedRepeat === "1") this.repeatOnQuestionStart = true;
      const storedMax = localStorage.getItem(MAX_REPEAT_KEY);
      if (storedMax !== null) {
        this.maxRepeat = clampMaxRepeat(Number.parseInt(storedMax, 10));
      }
    }
  }

  setSpeakQuestionOnShow(enabled: boolean) {
    this.speakQuestionOnShow = enabled;
    if (browser) {
      localStorage.setItem(SPEAK_QUESTION_KEY, enabled ? "1" : "0");
    }
  }

  setRepeatOnQuestionStart(enabled: boolean) {
    this.repeatOnQuestionStart = enabled;
    if (browser) {
      localStorage.setItem(REPEAT_ON_START_KEY, enabled ? "1" : "0");
    }
  }

  setMaxRepeat(value: number) {
    const clamped = clampMaxRepeat(value);
    this.maxRepeat = clamped;
    if (browser) {
      localStorage.setItem(MAX_REPEAT_KEY, String(clamped));
    }
  }

  toggleRepeat() {
    this.repeat = !this.repeat;
    // 切り替え時はカウンタも 0 に戻す (ON 直後の再生から数え始める)。
    this.repeatCount = 0;
  }

  /** Reviewer がカード切替などで再生サイクルを最初から数え直したいときに呼ぶ。 */
  resetRepeatCount() {
    this.repeatCount = 0;
  }
}

export const speech = new SpeechStore();
