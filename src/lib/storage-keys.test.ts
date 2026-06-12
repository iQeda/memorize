import { describe, expect, it } from "vitest";
import { STORAGE_KEYS } from "./storage-keys";

describe("STORAGE_KEYS", () => {
  it("values are frozen to the historical strings (renaming orphans user settings)", () => {
    expect(STORAGE_KEYS).toEqual({
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
      sidebarWidth: "sidebar.width",
    });
  });

  it("all values are unique", () => {
    const values = Object.values(STORAGE_KEYS);
    expect(new Set(values).size).toBe(values.length);
  });
});
