import { invoke } from "$lib/ipc";

export type ImportReport = {
  new: number;
  updated: number;
  duplicate: number;
  conflicting: number;
  first_field_match: number;
  missing_notetype: number;
  missing_deck: number;
  empty_first_field: number;
  found_notes: number;
};

export type ExportReport = { note_count: number };

export type ExportInput = {
  outPath: string;
  withScheduling: boolean;
  withMedia: boolean;
  withDeckConfigs: boolean;
  legacy: boolean;
};

class PackageStore {
  busy = $state(false);
  busyReason = $state<string | null>(null);
  lastImport = $state<ImportReport | null>(null);
  lastExportPath = $state<string | null>(null);
  lastError = $state<string | null>(null);

  async importApkg(inPath: string): Promise<ImportReport | null> {
    this.busy = true;
    this.busyReason = "Import 中…";
    this.lastError = null;
    try {
      const r = await invoke<ImportReport>("import_apkg", { inPath });
      this.lastImport = r;
      return r;
    } catch (e) {
      this.lastError = String(e);
      return null;
    } finally {
      this.busy = false;
      this.busyReason = null;
    }
  }

  async exportAll(input: ExportInput): Promise<ExportReport | null> {
    this.busy = true;
    this.busyReason = "Export 中…";
    this.lastError = null;
    try {
      const r = await invoke<ExportReport>("export_all_apkg", {
        input: {
          out_path: input.outPath,
          with_scheduling: input.withScheduling,
          with_media: input.withMedia,
          with_deck_configs: input.withDeckConfigs,
          legacy: input.legacy,
        },
      });
      this.lastExportPath = input.outPath;
      return r;
    } catch (e) {
      this.lastError = String(e);
      return null;
    } finally {
      this.busy = false;
      this.busyReason = null;
    }
  }
}

export const pkg = new PackageStore();
