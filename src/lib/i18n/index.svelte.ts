import { browser } from "$app/environment";
import { messages, type Locale, type MessageKey } from "./messages";

const STORAGE_KEY = "memorize:locale";
const DEFAULT_LOCALE: Locale = "en";

class I18nStore {
  locale = $state<Locale>(DEFAULT_LOCALE);

  constructor() {
    if (browser) {
      const stored = localStorage.getItem(STORAGE_KEY) as Locale | null;
      if (stored === "en" || stored === "ja") {
        this.locale = stored;
      }
    }
  }

  set(value: Locale) {
    this.locale = value;
    if (browser) localStorage.setItem(STORAGE_KEY, value);
  }

  /** Translate a key. Replaces `{name}` placeholders with `params[name]`. */
  t(key: MessageKey, params?: Record<string, string | number>): string {
    const table = messages[this.locale] ?? messages.en;
    const fallback = messages.en;
    let raw: string =
      (table as Record<string, string>)[key] ??
      (fallback as Record<string, string>)[key] ??
      key;
    if (params) {
      for (const [k, v] of Object.entries(params)) {
        raw = raw.replaceAll(`{${k}}`, String(v));
      }
    }
    return raw;
  }
}

export const i18n = new I18nStore();

/** Shorthand: read inside templates so reactivity tracks the locale. */
export function t(key: MessageKey, params?: Record<string, string | number>): string {
  return i18n.t(key, params);
}

export type { Locale };
