import { browser } from "$app/environment";

const STORAGE_KEY = "memorize:theme";

export type Theme = "light" | "dark" | "system";

class ThemeStore {
  preference = $state<Theme>("system");

  constructor() {
    if (browser) {
      const stored = localStorage.getItem(STORAGE_KEY) as Theme | null;
      if (stored === "light" || stored === "dark" || stored === "system") {
        this.preference = stored;
      }
      this.apply();
      window
        .matchMedia("(prefers-color-scheme: dark)")
        .addEventListener("change", () => this.apply());
    }
  }

  set(value: Theme) {
    this.preference = value;
    if (browser) {
      localStorage.setItem(STORAGE_KEY, value);
      this.apply();
    }
  }

  toggle() {
    this.set(this.resolved === "dark" ? "light" : "dark");
  }

  get resolved(): "light" | "dark" {
    if (this.preference === "system") {
      return browser &&
        window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light";
    }
    return this.preference;
  }

  private apply() {
    if (!browser) return;
    document.documentElement.classList.toggle("dark", this.resolved === "dark");
  }
}

export const theme = new ThemeStore();
