import { invoke } from "$lib/ipc";
import { runAsync } from "./run-async";

export type NotetypeSummary = {
  id: number;
  name: string;
  field_names: string[];
};

export type NoteDetail = {
  id: number;
  notetype_id: number;
  notetype_name: string;
  field_names: string[];
  fields: string[];
  tags: string[];
  deck_id: number;
};

class NotesStore {
  notetypes = $state<NotetypeSummary[]>([]);
  busy = $state(false);
  busyReason = $state<string | null>(null);
  lastError = $state<string | null>(null);

  async refreshNotetypes() {
    try {
      this.notetypes = await invoke<NotetypeSummary[]>("list_notetypes");
    } catch (e) {
      this.lastError = String(e);
    }
  }

  async getNote(noteId: number): Promise<NoteDetail | null> {
    try {
      return await invoke<NoteDetail>("get_note", { noteId });
    } catch (e) {
      this.lastError = String(e);
      return null;
    }
  }

  async addNote(input: {
    deckId: number;
    notetypeId: number;
    fields: string[];
    tags: string[];
  }): Promise<number | null> {
    return runAsync(this, () =>
      invoke<number>("add_note", {
        input: {
          deck_id: input.deckId,
          notetype_id: input.notetypeId,
          fields: input.fields,
          tags: input.tags,
        },
      }),
    );
  }

  async updateNote(input: {
    noteId: number;
    fields: string[];
    tags: string[];
  }): Promise<boolean> {
    const ok = await runAsync(this, async () => {
      await invoke("update_note", {
        input: {
          note_id: input.noteId,
          fields: input.fields,
          tags: input.tags,
        },
      });
      return true;
    });
    return ok ?? false;
  }

  async setNoteDeck(input: {
    noteId: number;
    deckId: number;
  }): Promise<boolean> {
    const ok = await runAsync(this, async () => {
      await invoke("set_note_deck", {
        input: { note_id: input.noteId, deck_id: input.deckId },
      });
      return true;
    });
    return ok ?? false;
  }

  async deleteNotes(noteIds: number[]): Promise<number> {
    const n = await runAsync(this, () =>
      invoke<number>("delete_notes", { noteIds }),
    );
    return n ?? 0;
  }
}

export const notes = new NotesStore();
