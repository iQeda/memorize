import { collection } from "./collection.svelte";
import {
  pkg,
  importKindForPath,
  type CsvPreview,
  type DupeResolution,
} from "./package.svelte";

/** Shared import flow used by every "Import" entry point (Settings and the
 *  sidebar deck list). One picker accepts every importable type and routes
 *  by extension: `.apkg` merges immediately, while text files (.tsv/.csv)
 *  open the preview dialog (`ImportPreviewModal`, rendered once in the
 *  layout) so the user can confirm the detected settings and duplicate
 *  strategy before committing. */
class ImportFlow {
  csvPath = $state<string | null>(null);
  csvPreview = $state<CsvPreview | null>(null);
  csvDupe = $state<DupeResolution>("update");

  /** True while the text-import preview dialog should be shown. */
  get active(): boolean {
    return this.csvPreview !== null && this.csvPath !== null;
  }

  /** Open the file picker and route the chosen file to the right backend. */
  async start() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const picked = await open({
        multiple: false,
        directory: false,
        filters: [
          {
            name: "Anki package / Text",
            extensions: ["apkg", "tsv", "csv", "txt"],
          },
        ],
      });
      if (typeof picked !== "string") return;
      const kind = importKindForPath(picked);
      if (kind === "apkg") {
        const r = await pkg.importApkg(picked);
        if (r) await collection.refreshDecks();
      } else if (kind === "text") {
        await this.openTextPreview(picked);
      }
    } catch (e) {
      console.error("importFlow.start", e);
    }
  }

  private async openTextPreview(path: string) {
    const preview = await pkg.csvMetadata(path);
    if (!preview) return;
    this.csvPath = path;
    this.csvPreview = preview;
    this.csvDupe = (["update", "preserve", "duplicate"] as const).includes(
      preview.dupe_resolution as DupeResolution,
    )
      ? (preview.dupe_resolution as DupeResolution)
      : "update";
  }

  cancel() {
    this.csvPath = null;
    this.csvPreview = null;
  }

  /** Commit the previewed text import using the selected duplicate strategy. */
  async confirm() {
    if (!this.csvPath) return;
    const path = this.csvPath;
    const dupe = this.csvDupe;
    this.csvPath = null;
    this.csvPreview = null;
    const r = await pkg.importTsv(path, dupe);
    if (r) await collection.refreshDecks();
  }
}

export const importFlow = new ImportFlow();
