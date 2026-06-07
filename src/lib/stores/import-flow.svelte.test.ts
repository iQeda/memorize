import { describe, expect, it, beforeEach } from "vitest";
import type { CsvPreview } from "./package.svelte";
import { importFlow } from "./import-flow.svelte";

const PREVIEW: CsvPreview = {
  deck: "Default",
  notetype: "Basic",
  delimiter: "\t",
  dupe_resolution: "update",
  columns: 2,
  preview_rows: [["apple", "りんご"]],
  is_html: false,
  tags_column: 0,
};

describe("importFlow", () => {
  beforeEach(() => {
    importFlow.cancel();
    importFlow.csvDupe = "update";
  });

  it("is inactive until a text preview is set", () => {
    expect(importFlow.active).toBe(false);
  });

  it("is active only when both path and preview are present", () => {
    importFlow.csvPath = "/a/words.csv";
    expect(importFlow.active).toBe(false); // preview still missing
    importFlow.csvPreview = PREVIEW;
    expect(importFlow.active).toBe(true);
  });

  it("cancel() clears the pending preview and deactivates", () => {
    importFlow.csvPath = "/a/words.csv";
    importFlow.csvPreview = PREVIEW;
    importFlow.cancel();
    expect(importFlow.csvPath).toBeNull();
    expect(importFlow.csvPreview).toBeNull();
    expect(importFlow.active).toBe(false);
  });
});
