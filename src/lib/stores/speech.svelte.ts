import { browser } from "$app/environment";

const SPEAK_QUESTION_KEY = "memorize:speak-question-on-show";

/** 1 シーケンスあたりの最大連続再生回数 (1 回目を含む)。
 *  ユーザー要件: 「最大 5 回連続再生」「最大回数に達したら repeat は自動 off」。 */
export const MAX_REPEAT = 5;

class SpeechStore {
  speakQuestionOnShow = $state(false);
  /** Reviewer 表示中のみ意味を持つセッションフラグ。永続化しない。
   *  ON のあいだ、`memorize://speech-finished` 受信時に 1 秒ポーズして
   *  同じテキストを再生し直す。`repeatCount` が `MAX_REPEAT` に達したら
   *  自動で `false` に戻る。 */
  repeat = $state(false);
  /** 現在のリピートサイクル内で何回再生したか (1 回目で 1)。`MAX_REPEAT` 到達で自動 off。 */
  repeatCount = $state(0);

  constructor() {
    if (browser) {
      // Default: OFF. Stored "1" = ON. macOS の "選択項目を読み上げる" は
      // OS 設定とアクセシビリティ権限を要求するため、明示的なオプトイン。
      const stored = localStorage.getItem(SPEAK_QUESTION_KEY);
      if (stored === "1") this.speakQuestionOnShow = true;
    }
  }

  setSpeakQuestionOnShow(enabled: boolean) {
    this.speakQuestionOnShow = enabled;
    if (browser) {
      localStorage.setItem(SPEAK_QUESTION_KEY, enabled ? "1" : "0");
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
