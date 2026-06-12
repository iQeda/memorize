import { invoke } from "$lib/ipc";
import { t } from "$lib/i18n/index.svelte";
import { runAsync } from "./run-async";

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

export type CsvPreview = {
  deck: string;
  notetype: string;
  delimiter: string;
  dupe_resolution: string;
  columns: number;
  preview_rows: string[][];
  is_html: boolean;
  tags_column: number;
};

export type DupeResolution = "update" | "preserve" | "duplicate";

export type ImportKind = "apkg" | "text";

/** Route an import file to the right backend based on its extension.
 *  `.apkg` is a one-shot package merge; `.tsv` / `.csv` / `.txt` go
 *  through the text-import preview flow. Returns null for anything we
 *  can't import. Used by the unified "Import" button so .apkg and text
 *  files share a single picker instead of separate buttons. */
export function importKindForPath(path: string): ImportKind | null {
  const ext = path.toLowerCase().split(".").pop() ?? "";
  if (ext === "apkg") return "apkg";
  if (ext === "tsv" || ext === "csv" || ext === "txt") return "text";
  return null;
}

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
    return runAsync(
      this,
      async () => {
        const r = await invoke<ImportReport>("import_apkg", { inPath });
        this.lastImport = r;
        return r;
      },
      { reason: t("io.importing") },
    );
  }

  /** Detect import settings for a CSV/TSV file without mutating the collection. */
  async csvMetadata(inPath: string): Promise<CsvPreview | null> {
    this.lastError = null;
    try {
      return await invoke<CsvPreview>("csv_metadata", { inPath });
    } catch (e) {
      this.lastError = String(e);
      return null;
    }
  }

  async importTsv(
    inPath: string,
    dupeResolution: DupeResolution,
  ): Promise<ImportReport | null> {
    return runAsync(
      this,
      async () => {
        const r = await invoke<ImportReport>("import_tsv", { inPath, dupeResolution });
        this.lastImport = r;
        return r;
      },
      { reason: t("io.importing") },
    );
  }

  async exportAll(input: ExportInput): Promise<ExportReport | null> {
    return runAsync(
      this,
      async () => {
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
      },
      { reason: t("io.exporting") },
    );
  }
}

export const pkg = new PackageStore();
