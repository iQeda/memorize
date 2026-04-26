import { invoke } from "$lib/ipc";

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
};

class NotesStore {
  notetypes = $state<NotetypeSummary[]>([]);
  busy = $state(false);
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
    this.busy = true;
    this.lastError = null;
    try {
      const id = await invoke<number>("add_note", {
        input: {
          deck_id: input.deckId,
          notetype_id: input.notetypeId,
          fields: input.fields,
          tags: input.tags,
        },
      });
      return id;
    } catch (e) {
      this.lastError = String(e);
      return null;
    } finally {
      this.busy = false;
    }
  }

  async updateNote(input: {
    noteId: number;
    fields: string[];
    tags: string[];
  }): Promise<boolean> {
    this.busy = true;
    this.lastError = null;
    try {
      await invoke("update_note", {
        input: {
          note_id: input.noteId,
          fields: input.fields,
          tags: input.tags,
        },
      });
      return true;
    } catch (e) {
      this.lastError = String(e);
      return false;
    } finally {
      this.busy = false;
    }
  }

  async deleteNotes(noteIds: number[]): Promise<number> {
    this.busy = true;
    this.lastError = null;
    try {
      return await invoke<number>("delete_notes", { noteIds });
    } catch (e) {
      this.lastError = String(e);
      return 0;
    } finally {
      this.busy = false;
    }
  }
}

export const notes = new NotesStore();
