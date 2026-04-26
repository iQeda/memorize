import { invoke } from "$lib/ipc";

export type DeckSummary = {
  id: number;
  name: string;
  level: number;
  new_count: number;
  learn_count: number;
  review_count: number;
};

class CollectionStore {
  isOpen = $state(false);
  decks = $state<DeckSummary[]>([]);
  selectedDeckId = $state<number | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);

  async open(path: string) {
    this.loading = true;
    this.error = null;
    try {
      await invoke("open_collection", { path });
      this.isOpen = true;
      await this.refreshDecks();
    } catch (e) {
      this.error = String(e);
      this.isOpen = false;
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
    }
  }

  async refreshDecks() {
    if (!this.isOpen) return;
    this.decks = await invoke<DeckSummary[]>("list_decks");
    if (this.selectedDeckId === null && this.decks.length > 0) {
      this.selectedDeckId = this.decks[0].id;
    }
  }

  get selectedDeck(): DeckSummary | null {
    return this.decks.find((d) => d.id === this.selectedDeckId) ?? null;
  }
}

export const collection = new CollectionStore();
