import { invoke } from "$lib/ipc";
import { browser } from "$app/environment";
import { STORAGE_KEYS } from "$lib/storage-keys";

export type DeckSummary = {
  id: number;
  name: string;
  level: number;
  new_count: number;
  learn_count: number;
  review_count: number;
};

const LAST_PATH_KEY = STORAGE_KEYS.lastCollectionPath;

type CollectionInfo = {
  current_path: string | null;
  anki_desktop_path: string | null;
};

class CollectionStore {
  isOpen = $state(false);
  decks = $state<DeckSummary[]>([]);
  selectedDeckId = $state<number | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);
  currentPath = $state<string | null>(null);
  ankiDesktopPath = $state<string | null>(null);

  async refreshInfo() {
    try {
      const info = await invoke<CollectionInfo>("collection_info");
      this.currentPath = info.current_path;
      this.ankiDesktopPath = info.anki_desktop_path;
    } catch (e) {
      console.error("collection.refreshInfo", e);
    }
  }

  /** Reconcile the frontend store with the backend AppState, and
   *  auto-reopen the last-used collection if the backend has none open
   *  (handles the cold-start case after a Tauri relaunch). */
  async refresh() {
    try {
      const open = await invoke<boolean>("is_open");
      if (open) {
        this.isOpen = true;
        await this.refreshDecks();
        await this.refreshInfo();
        return;
      }
      const lastPath = browser ? localStorage.getItem(LAST_PATH_KEY) : null;
      // DEV モードでは sandbox 以外の lastPath は使わない。本物の Anki
      // collection を持つ手元では lastPath にプロダクションパスが残っている
      // ことが多く、それを再 open してしまうと「dev deck が読み込まれる」と
      // いう dev フローの前提が崩れる。sandbox path 以外は無視し、必ず
      // bootstrap に流す。
      const isDevSandbox = lastPath?.includes("/.memorize-dev/collection.anki2") ?? false;
      if (lastPath && (!import.meta.env.DEV || isDevSandbox)) {
        await this.open(lastPath, /* skipPersist */ true);
      }
      // Dev builds bootstrap a sandbox collection at <repo>/.memorize-dev
      // so `pnpm tauri dev` always lands in a usable state. Production
      // builds skip this entirely (the command is cfg(debug_assertions)).
      if (!this.isOpen && import.meta.env.DEV) {
        await this.bootstrapDevCollection();
      }
      await this.refreshInfo();
    } catch (e) {
      console.error("collection.refresh", e);
    }
  }

  private async bootstrapDevCollection() {
    try {
      await invoke<void>("bootstrap_dev_collection");
      this.isOpen = true;
      await this.refreshDecks();
    } catch (e) {
      console.error("collection.bootstrapDevCollection", e);
    }
  }

  // runAsync は使わない: この store は busy/lastError でなく loading/error を
  // 公開しており (UI が collection.error を参照)、エラー時に isOpen リセットと
  // stale path の削除という固有の副作用も持つため、共通ヘルパーに合わない。
  async open(path: string, skipPersist = false) {
    this.loading = true;
    this.error = null;
    try {
      await invoke<void>("open_collection", { path });
      this.isOpen = true;
      if (!skipPersist && browser) {
        localStorage.setItem(LAST_PATH_KEY, path);
      }
      await this.refreshDecks();
      await this.refreshInfo();
    } catch (e) {
      this.error = String(e);
      this.isOpen = false;
      // Don't keep stale path that fails to open.
      if (browser) localStorage.removeItem(LAST_PATH_KEY);
    } finally {
      this.loading = false;
    }
  }

  /** Show a file picker for an existing .anki2 and open it. Used from
   *  the welcome screen and from Settings (after an explicit close). */
  async pickAndOpen(): Promise<boolean> {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const picked = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "Anki collection", extensions: ["anki2"] }],
      });
      if (typeof picked !== "string") return false;
      await this.open(picked);
      return this.isOpen;
    } catch (e) {
      console.error("pickAndOpen failed", e);
      return false;
    }
  }

  /** Show a save picker for a new .anki2 path; rslib creates the file
   *  on first build() so we can route through the same `open` flow. */
  async createNew(): Promise<boolean> {
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const picked = await save({
        defaultPath: "memorize-collection.anki2",
        filters: [{ name: "Anki collection", extensions: ["anki2"] }],
      });
      if (typeof picked !== "string") return false;
      await this.open(picked);
      return this.isOpen;
    } catch (e) {
      console.error("createNew failed", e);
      return false;
    }
  }

  async close() {
    try {
      await invoke<void>("close_collection");
    } finally {
      this.isOpen = false;
      this.decks = [];
      this.selectedDeckId = null;
      if (browser) localStorage.removeItem(LAST_PATH_KEY);
    }
  }

  async refreshDecks() {
    if (!this.isOpen) return;
    this.decks = await invoke<DeckSummary[]>("list_decks");
    if (this.selectedDeckId === null && this.decks.length > 0) {
      this.selectedDeckId = this.decks[0].id;
    }
  }

  async createDeck(name: string): Promise<number | null> {
    try {
      const id = await invoke<number>("create_deck", { name });
      await this.refreshDecks();
      this.selectedDeckId = id;
      return id;
    } catch (e) {
      this.error = String(e);
      return null;
    }
  }

  async renameDeck(deckId: number, newName: string): Promise<boolean> {
    try {
      await invoke<void>("rename_deck", { deckId, newName });
      await this.refreshDecks();
      return true;
    } catch (e) {
      this.error = String(e);
      return false;
    }
  }

  async deleteDeck(deckId: number): Promise<boolean> {
    try {
      await invoke<void>("delete_deck", { deckId });
      if (this.selectedDeckId === deckId) {
        this.selectedDeckId = null;
      }
      await this.refreshDecks();
      return true;
    } catch (e) {
      this.error = String(e);
      return false;
    }
  }

  get selectedDeck(): DeckSummary | null {
    return this.decks.find((d) => d.id === this.selectedDeckId) ?? null;
  }

  /** The Anki Desktop collection path to suggest switching to, or null
   *  when there's nothing to suggest. Returns a path only when an Anki
   *  Desktop collection was detected AND it differs from whatever is
   *  currently open (when nothing is open, currentPath is null so any
   *  detected path is offered). Drives the "switch to your AnkiWeb-synced
   *  collection" suggestion shown on both the welcome screen and Settings. */
  get ankiDesktopSuggestion(): string | null {
    if (!this.ankiDesktopPath) return null;
    if (this.ankiDesktopPath === this.currentPath) return null;
    return this.ankiDesktopPath;
  }
}

export const collection = new CollectionStore();
