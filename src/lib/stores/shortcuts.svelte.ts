import { browser } from "$app/environment";

export type Rating = "again" | "hard" | "good" | "easy";
export type Action = Rating | "copy";

const STORAGE_KEY = "memorize:rating-keys";

const defaults: Record<Action, string> = {
  again: "a",
  hard: "s",
  good: "d",
  easy: "f",
  copy: "j",
};

function formatKey(k: string): string {
  if (k === " ") return "Space";
  if (k === "Enter") return "↵";
  if (k === "Escape") return "Esc";
  if (k === "ArrowUp") return "↑";
  if (k === "ArrowDown") return "↓";
  if (k === "ArrowLeft") return "←";
  if (k === "ArrowRight") return "→";
  return k.toUpperCase();
}

class ShortcutsStore {
  keys = $state<Record<Action, string>>({ ...defaults });

  constructor() {
    if (browser) {
      try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (raw) {
          const parsed = JSON.parse(raw) as Partial<Record<Action, string>>;
          for (const r of Object.keys(defaults) as Action[]) {
            if (typeof parsed[r] === "string" && parsed[r]!.length > 0) {
              this.keys[r] = parsed[r]!;
            }
          }
        }
      } catch {}
    }
  }

  set(action: Action, key: string) {
    this.keys[action] = key;
    if (browser) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.keys));
    }
  }

  reset() {
    this.keys = { ...defaults };
    if (browser) localStorage.removeItem(STORAGE_KEY);
  }

  /** Look up the rating bound to a given KeyboardEvent.key. */
  ratingFor(key: string): Rating | null {
    for (const r of ["again", "hard", "good", "easy"] as const) {
      if (this.keys[r] === key) return r;
    }
    return null;
  }

  /** True iff `key` is bound to the Copy action. */
  isCopy(key: string): boolean {
    return this.keys.copy === key;
  }

  label(action: Action): string {
    return formatKey(this.keys[action]);
  }
}

export const shortcuts = new ShortcutsStore();
