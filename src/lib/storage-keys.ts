/** アプリ全体の localStorage キーの単一ソース。
 *
 *  値はユーザーの既存設定と 1:1 で紐づくため、リネームすると設定が
 *  silent に消える (マイグレーションなし)。文字列値は変更禁止 —
 *  storage-keys.test.ts のスナップショットが守る。 */
export const STORAGE_KEYS = {
  theme: "memorize:theme",
  ratingKeys: "memorize:rating-keys",
  speakQuestionOnShow: "memorize:speak-question-on-show",
  repeatOnQuestionStart: "memorize:repeat-on-question-start",
  maxRepeat: "memorize:max-repeat",
  repeatIntervalSec: "memorize:repeat-interval-sec",
  hideDefault: "memorize:hide-default",
  speechRateWpm: "memorize:speech-rate-wpm",
  sentencePauseMs: "memorize:sentence-pause-ms",
  autoRevealAfterRepeat: "memorize:auto-reveal-after-repeat",
  speechVolume: "memorize:speech-volume",
  lastCollectionPath: "memorize:last-collection-path",
  autoBackupBeforeSync: "memorize:auto-backup-before-sync",
  autoSyncOnStartStop: "memorize:auto-sync-on-start-stop",
  deckOrder: "memorize:deck-order",
  locale: "memorize:locale",
  // 歴史的経緯で memorize: プレフィックスなし。値を揃えるのもリネーム扱い
  // になりユーザーのサイドバー幅設定が消えるため、このまま維持する。
  sidebarWidth: "sidebar.width",
} as const;
