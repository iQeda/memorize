import { describe, expect, it } from "vitest";
import { importKindForPath } from "./package.svelte";

describe("importKindForPath", () => {
  it("classifies .apkg as a package import", () => {
    expect(importKindForPath("/a/b/deck.apkg")).toBe("apkg");
  });

  it("classifies .tsv / .csv / .txt as a text import", () => {
    expect(importKindForPath("/a/words.tsv")).toBe("text");
    expect(importKindForPath("/a/words.csv")).toBe("text");
    expect(importKindForPath("/a/words.txt")).toBe("text");
  });

  it("is case-insensitive on the extension", () => {
    expect(importKindForPath("/a/DECK.APKG")).toBe("apkg");
    expect(importKindForPath("/a/WORDS.CSV")).toBe("text");
  });

  it("handles filenames with multiple dots (uses the last extension)", () => {
    expect(importKindForPath("/a/my.vocab.list.csv")).toBe("text");
    expect(importKindForPath("/a/backup.2026.apkg")).toBe("apkg");
  });

  it("returns null for unsupported or missing extensions", () => {
    expect(importKindForPath("/a/collection.colpkg")).toBeNull();
    expect(importKindForPath("/a/notes.json")).toBeNull();
    expect(importKindForPath("/a/README")).toBeNull();
    expect(importKindForPath("")).toBeNull();
  });
});
