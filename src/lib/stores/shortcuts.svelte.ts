import { browser } from "$app/environment";

export type Rating = "again" | "hard" | "good" | "easy";

const STORAGE_KEY = "memorize:rating-keys";

const defaults: Record<Rating, string> = {
  again: "1",
  hard: "2",
  good: "3",
  easy: "4",
};

class ShortcutsStore {
  keys = $state<Record<Rating, string>>({ ...defaults });

  constructor() {
    if (browser) {
      try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (raw) {
          const parsed = JSON.parse(raw) as Partial<Record<Rating, string>>;
          for (const r of Object.keys(defaults) as Rating[]) {
            if (typeof parsed[r] === "string" && parsed[r]!.length > 0) {
              this.keys[r] = parsed[r]!;
            }
          }
        }
      } catch {}
    }
  }

  /** Set the key for a given rating. */
  set(rating: Rating, key: string) {
    this.keys[rating] = key;
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

  /** Display label for a key (mostly raw). */
  label(rating: Rating): string {
    const k = this.keys[rating];
    if (k === " ") return "Space";
    if (k === "Enter") return "↵";
    if (k === "Escape") return "Esc";
    if (k === "ArrowUp") return "↑";
    if (k === "ArrowDown") return "↓";
    if (k === "ArrowLeft") return "←";
    if (k === "ArrowRight") return "→";
    return k.toUpperCase();
  }
}

export const shortcuts = new ShortcutsStore();
