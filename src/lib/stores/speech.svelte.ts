import { browser } from "$app/environment";

const SPEAK_QUESTION_KEY = "memorize:speak-question-on-show";
const REPEAT_ON_START_KEY = "memorize:repeat-on-question-start";
const MAX_REPEAT_KEY = "memorize:max-repeat";
const REPEAT_INTERVAL_KEY = "memorize:repeat-interval-sec";
const HIDE_DEFAULT_KEY = "memorize:hide-default";
const SPEECH_RATE_KEY = "memorize:speech-rate-wpm";
const SENTENCE_PAUSE_KEY = "memorize:sentence-pause-ms";
const AUTO_REVEAL_KEY = "memorize:auto-reveal-after-repeat";

/** デフォルト最大連続再生回数 (1 回目を含む)。設定画面の数値入力で上書き可能。 */
export const DEFAULT_MAX_REPEAT = 3;
/** 設定 UI で受け付ける範囲。極端値は再生体験を壊すので 1..10 に clamp。 */
export const MAX_REPEAT_MIN = 1;
export const MAX_REPEAT_MAX = 10;

/** リピート再生間のポーズ秒数のデフォルト。設定画面の数値入力で上書き可能。 */
export const DEFAULT_REPEAT_INTERVAL_SEC = 1;
/** 設定 UI で受け付ける範囲。0 = ポーズなしで即リピート。 */
export const REPEAT_INTERVAL_MIN = 0;
export const REPEAT_INTERVAL_MAX = 10;

/** デフォルト読み上げ速度 (words per minute)。
 *  リスニング学習用にやや遅め (150) を採用。`say` voice の組み込み既定は ~175-200。
 *  macOS Accessibility の Speaking Rate スライダーは `say` から参照できないため
 *  アプリ側で独自に持つ。 */
export const DEFAULT_SPEECH_RATE_WPM = 150;
/** 設定 UI で受け付ける wpm 範囲。極端値は実用性ゼロなので clamp。 */
export const SPEECH_RATE_MIN = 100;
export const SPEECH_RATE_MAX = 400;

/** 文末の追加ポーズ (ms)。0 = 追加なし。リスニング学習で文間に考える間を入れる
 *  目的でデフォルト 500 ms。0 にするには設定 UI から手動で 0 を入力する。 */
export const DEFAULT_SENTENCE_PAUSE_MS = 500;
export const SENTENCE_PAUSE_MIN = 0;
export const SENTENCE_PAUSE_MAX = 5000;

function clampMaxRepeat(n: number): number {
  if (!Number.isFinite(n)) return DEFAULT_MAX_REPEAT;
  return Math.min(MAX_REPEAT_MAX, Math.max(MAX_REPEAT_MIN, Math.round(n)));
}

function clampRepeatInterval(n: number): number {
  if (!Number.isFinite(n)) return DEFAULT_REPEAT_INTERVAL_SEC;
  // 0.01 秒刻みを許容。浮動小数の桁あふれを避けるため小数第 2 位で丸める。
  const rounded = Math.round(n * 100) / 100;
  return Math.min(REPEAT_INTERVAL_MAX, Math.max(REPEAT_INTERVAL_MIN, rounded));
}

function clampSpeechRate(n: number): number {
  if (!Number.isFinite(n)) return DEFAULT_SPEECH_RATE_WPM;
  return Math.min(SPEECH_RATE_MAX, Math.max(SPEECH_RATE_MIN, Math.round(n)));
}

function clampSentencePause(n: number): number {
  if (!Number.isFinite(n)) return DEFAULT_SENTENCE_PAUSE_MS;
  return Math.min(SENTENCE_PAUSE_MAX, Math.max(SENTENCE_PAUSE_MIN, Math.round(n)));
}

class SpeechStore {
  speakQuestionOnShow = $state(false);
  /** 問題開始時に `repeat` を自動 ON にする永続設定。
   *  ON にしておくと Reviewer に入るたびにチェックボックスが入った状態で
   *  始まり、新カードの自動再生 (`speakQuestionOnShow`) と組み合わさって
   *  毎カード 5 回ループが自動的に走る。 */
  repeatOnQuestionStart = $state(false);
  /** 1 サイクルあたりの最大再生回数 (1 回目を含む)。永続化、デフォルト 3。 */
  maxRepeat = $state(DEFAULT_MAX_REPEAT);
  /** リピート再生間のポーズ秒数。永続化、デフォルト 1 秒。 */
  repeatIntervalSec = $state(DEFAULT_REPEAT_INTERVAL_SEC);
  /** Reviewer 表示中のみ意味を持つセッションフラグ。永続化しない。
   *  ON のあいだ、`memorize://speech-finished` 受信時に 1 秒ポーズして
   *  同じテキストを再生し直す。 */
  repeat = $state(false);
  /** 現在のリピートサイクル内で何回再生したか (1 回目で 1)。`maxRepeat` 到達でループ停止。 */
  repeatCount = $state(0);
  /** 新しいカードを表示するたびに front 側を非表示状態で開始するか。永続化、デフォルト OFF。
   *  ON のとき Reviewer は `l` キーで都度 reveal する運用になる。 */
  hideDefault = $state(false);
  /** `say -r` に渡す読み上げ速度 (wpm)。永続化、デフォルト 180。 */
  speechRate = $state(DEFAULT_SPEECH_RATE_WPM);
  /** 文末 (`.`/`!`/`?`/`。`/`！`/`？`) の後に挿入するポーズ (ms)。
   *  0 で追加なし。Rust 側で `[[slnc N]]` を埋め込む。 */
  sentencePauseMs = $state(DEFAULT_SENTENCE_PAUSE_MS);
  /** リピート再生がサイクル完了 (`repeatCount >= maxRepeat`) を満たした時、
   *  Reviewer 側で hidden 状態を自動的に解除するか。永続化。デフォルト false。 */
  autoRevealAfterRepeat = $state(false);

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
      const storedInterval = localStorage.getItem(REPEAT_INTERVAL_KEY);
      if (storedInterval !== null) {
        this.repeatIntervalSec = clampRepeatInterval(
          Number.parseFloat(storedInterval),
        );
      }
      const storedHide = localStorage.getItem(HIDE_DEFAULT_KEY);
      if (storedHide === "1") this.hideDefault = true;
      const storedRate = localStorage.getItem(SPEECH_RATE_KEY);
      if (storedRate !== null) {
        this.speechRate = clampSpeechRate(Number.parseInt(storedRate, 10));
      }
      const storedPause = localStorage.getItem(SENTENCE_PAUSE_KEY);
      if (storedPause !== null) {
        this.sentencePauseMs = clampSentencePause(Number.parseInt(storedPause, 10));
      }
      const storedAutoReveal = localStorage.getItem(AUTO_REVEAL_KEY);
      if (storedAutoReveal === "1") this.autoRevealAfterRepeat = true;
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

  setRepeatIntervalSec(value: number) {
    const clamped = clampRepeatInterval(value);
    this.repeatIntervalSec = clamped;
    if (browser) {
      localStorage.setItem(REPEAT_INTERVAL_KEY, String(clamped));
    }
  }

  setHideDefault(enabled: boolean) {
    this.hideDefault = enabled;
    if (browser) {
      localStorage.setItem(HIDE_DEFAULT_KEY, enabled ? "1" : "0");
    }
  }

  setSpeechRate(value: number) {
    const clamped = clampSpeechRate(value);
    this.speechRate = clamped;
    if (browser) {
      localStorage.setItem(SPEECH_RATE_KEY, String(clamped));
    }
  }

  setSentencePauseMs(value: number) {
    const clamped = clampSentencePause(value);
    this.sentencePauseMs = clamped;
    if (browser) {
      localStorage.setItem(SENTENCE_PAUSE_KEY, String(clamped));
    }
  }

  setAutoRevealAfterRepeat(enabled: boolean) {
    this.autoRevealAfterRepeat = enabled;
    if (browser) {
      localStorage.setItem(AUTO_REVEAL_KEY, enabled ? "1" : "0");
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
