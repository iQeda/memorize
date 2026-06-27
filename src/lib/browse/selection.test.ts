import { describe, expect, it } from "vitest";
import {
  allCardIds,
  allSelected,
  selectedNoteIds,
  someSelected,
  type SelectableCard,
} from "./selection";

const cards: SelectableCard[] = [
  { id: 1, note_id: 10 },
  { id: 2, note_id: 20 },
  { id: 3, note_id: 30 },
];

describe("selectedNoteIds", () => {
  it("returns empty for an empty selection", () => {
    expect(selectedNoteIds(cards, new Set())).toEqual([]);
  });

  it("maps selected card ids to their note ids in display order", () => {
    expect(selectedNoteIds(cards, new Set([1, 3]))).toEqual([10, 30]);
  });

  it("dedupes note ids when several selected cards share a note", () => {
    const multi: SelectableCard[] = [
      { id: 1, note_id: 10 },
      { id: 2, note_id: 10 },
      { id: 3, note_id: 20 },
    ];
    expect(selectedNoteIds(multi, new Set([1, 2, 3]))).toEqual([10, 20]);
  });

  it("ignores selected ids not present in the current list", () => {
    expect(selectedNoteIds(cards, new Set([1, 999]))).toEqual([10]);
  });
});

describe("allSelected / someSelected", () => {
  it("is false for an empty selection", () => {
    expect(allSelected(cards, new Set())).toBe(false);
    expect(someSelected(cards, new Set())).toBe(false);
  });

  it("partial selection is some-but-not-all", () => {
    const sel = new Set([2]);
    expect(allSelected(cards, sel)).toBe(false);
    expect(someSelected(cards, sel)).toBe(true);
  });

  it("full selection is allSelected", () => {
    const sel = new Set([1, 2, 3]);
    expect(allSelected(cards, sel)).toBe(true);
    expect(someSelected(cards, sel)).toBe(true);
  });

  it("an empty list is never allSelected", () => {
    expect(allSelected([], new Set())).toBe(false);
  });
});

describe("allCardIds", () => {
  it("returns every visible card id", () => {
    expect(allCardIds(cards)).toEqual(new Set([1, 2, 3]));
  });

  it("returns an empty set for an empty list", () => {
    expect(allCardIds([])).toEqual(new Set());
  });
});
