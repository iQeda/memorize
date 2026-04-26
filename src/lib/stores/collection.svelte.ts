import { invoke } from "$lib/ipc";
import { browser } from "$app/environment";

export type DeckSummary = {
  id: number;
  name: string;
  level: number;
  new_count: number;
  learn_count: number;
  review_count: number;
};

const LAST_PATH_KEY = "memorize:last-collection-path";

class CollectionStore {
  isOpen = $state(false);
  decks = $state<DeckSummary[]>([]);
  selectedDeckId = $state<number | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);

  /** Reconcile the frontend store with the backend AppState, and
   *  auto-reopen the last-used collection if the backend has none open
   *  (handles the cold-start case after a Tauri relaunch). */
  async refresh() {
    try {
      const open = await invoke<boolean>("is_open");
      if (open) {
        this.isOpen = true;
        await this.refreshDecks();
        return;
      }
      const lastPath = browser ? localStorage.getItem(LAST_PATH_KEY) : null;
      if (lastPath) {
        await this.open(lastPath, /* skipPersist */ true);
      }
    } catch (e) {
      console.error("collection.refresh", e);
    }
  }

  async open(path: string, skipPersist = false) {
    this.loading = true;
    this.error = null;
    try {
      await invoke("open_collection", { path });
      this.isOpen = true;
      if (!skipPersist && browser) {
        localStorage.setItem(LAST_PATH_KEY, path);
      }
      await this.refreshDecks();
    } catch (e) {
      this.error = String(e);
      this.isOpen = false;
      // Don't keep stale path that fails to open.
      if (browser) localStorage.removeItem(LAST_PATH_KEY);
    } finally {
      this.loading = false;
    }
  }

  async close() {
    try {
      await invoke("close_collection");
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

  get selectedDeck(): DeckSummary | null {
    return this.decks.find((d) => d.id === this.selectedDeckId) ?? null;
  }
}

export const collection = new CollectionStore();
