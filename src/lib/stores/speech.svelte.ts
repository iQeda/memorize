import { browser } from "$app/environment";
import { STORAGE_KEYS } from "$lib/storage-keys";

const SPEAK_QUESTION_KEY = STORAGE_KEYS.speakQuestionOnShow;
const REPEAT_ON_START_KEY = STORAGE_KEYS.repeatOnQuestionStart;
const MAX_REPEAT_KEY = STORAGE_KEYS.maxRepeat;
const REPEAT_INTERVAL_KEY = STORAGE_KEYS.repeatIntervalSec;
const HIDE_DEFAULT_KEY = STORAGE_KEYS.hideDefault;
const SPEECH_RATE_KEY = STORAGE_KEYS.speechRateWpm;
const SENTENCE_PAUSE_KEY = STORAGE_KEYS.sentencePauseMs;
const AUTO_REVEAL_KEY = STORAGE_KEYS.autoRevealAfterRepeat;
const SPEECH_VOLUME_KEY = STORAGE_KEYS.speechVolume;

/** 設定 UI で受け付ける speech パラメータの範囲とデフォルト。
 *
 *  - maxRepeat: 1 サイクルあたりの最大再生回数 (1 回目を含む)。極端値は
 *    再生体験を壊すので 1..10 に clamp。
 *  - repeatIntervalSec: リピート再生間のポーズ秒数。0 = ポーズなしで即リピート。
 *  - rateWpm: `say -r` の読み上げ速度。リスニング学習用にやや遅め (150) を
 *    デフォルトに採用 (`say` voice の組み込み既定は ~175-200)。macOS
 *    Accessibility の Speaking Rate スライダーは `say` から参照できないため
 *    アプリ側で独自に持つ。
 *  - sentencePauseMs: 文末の追加ポーズ。0 = 追加なし。文間に考える間を
 *    入れる目的でデフォルト 500 ms。
 *  - volume: 読み上げ音量 (%)。100 = voice 既定。100 以下は `[[volm]]`
 *    埋め込み (低レイテンシ)、100 超は `say -o aiff` → `afplay -v gain`
 *    で増幅再生。200 超は音割れが顕著なので上限 200。 */
export const SPEECH_LIMITS = {
  maxRepeat: { min: 1, max: 10, default: 3 },
  repeatIntervalSec: { min: 0, max: 10, default: 1 },
  rateWpm: { min: 100, max: 400, default: 150 },
  sentencePauseMs: { min: 0, max: 5000, default: 500 },
  volume: { min: 0, max: 200, default: 100 },
} as const;

function clampMaxRepeat(n: number): number {
  const l = SPEECH_LIMITS.maxRepeat;
  if (!Number.isFinite(n)) return l.default;
  return Math.min(l.max, Math.max(l.min, Math.round(n)));
}

function clampRepeatInterval(n: number): number {
  const l = SPEECH_LIMITS.repeatIntervalSec;
  if (!Number.isFinite(n)) return l.default;
  // 0.01 秒刻みを許容。浮動小数の桁あふれを避けるため小数第 2 位で丸める。
  const rounded = Math.round(n * 100) / 100;
  return Math.min(l.max, Math.max(l.min, rounded));
}

function clampSpeechRate(n: number): number {
  const l = SPEECH_LIMITS.rateWpm;
  if (!Number.isFinite(n)) return l.default;
  return Math.min(l.max, Math.max(l.min, Math.round(n)));
}

function clampSentencePause(n: number): number {
  const l = SPEECH_LIMITS.sentencePauseMs;
  if (!Number.isFinite(n)) return l.default;
  return Math.min(l.max, Math.max(l.min, Math.round(n)));
}

function clampSpeechVolume(n: number): number {
  const l = SPEECH_LIMITS.volume;
  if (!Number.isFinite(n)) return l.default;
  return Math.min(l.max, Math.max(l.min, Math.round(n)));
}

class SpeechStore {
  speakQuestionOnShow = $state(false);
  /** 問題開始時に `repeat` を自動 ON にする永続設定。
   *  ON にしておくと Reviewer に入るたびにチェックボックスが入った状態で
   *  始まり、新カードの自動再生 (`speakQuestionOnShow`) と組み合わさって
   *  毎カード 5 回ループが自動的に走る。 */
  repeatOnQuestionStart = $state(false);
  /** 1 サイクルあたりの最大再生回数 (1 回目を含む)。永続化、デフォルト 3。 */
  maxRepeat = $state<number>(SPEECH_LIMITS.maxRepeat.default);
  /** リピート再生間のポーズ秒数。永続化、デフォルト 1 秒。 */
  repeatIntervalSec = $state<number>(SPEECH_LIMITS.repeatIntervalSec.default);
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
  speechRate = $state<number>(SPEECH_LIMITS.rateWpm.default);
  /** 文末 (`.`/`!`/`?`/`。`/`！`/`？`) の後に挿入するポーズ (ms)。
   *  0 で追加なし。Rust 側で `[[slnc N]]` を埋め込む。 */
  sentencePauseMs = $state<number>(SPEECH_LIMITS.sentencePauseMs.default);
  /** リピート再生がサイクル完了 (`repeatCount >= maxRepeat`) を満たした時、
   *  Reviewer 側で hidden 状態を自動的に解除するか。永続化。デフォルト false。 */
  autoRevealAfterRepeat = $state(false);
  /** 読み上げ音量 (0-100, %)。永続化、デフォルト 100。
   *  Rust 側で `[[volm X.XX]]` を `say` に埋め込んで制御する。 */
  volume = $state<number>(SPEECH_LIMITS.volume.default);

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
      const storedVolume = localStorage.getItem(SPEECH_VOLUME_KEY);
      if (storedVolume !== null) {
        this.volume = clampSpeechVolume(Number.parseInt(storedVolume, 10));
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

  setVolume(value: number) {
    const clamped = clampSpeechVolume(value);
    this.volume = clamped;
    if (browser) {
      localStorage.setItem(SPEECH_VOLUME_KEY, String(clamped));
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
