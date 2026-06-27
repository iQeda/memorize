/** A Browse table row. Rows are cards; `note_id` is the word they belong to. */
export type SelectableCard = { id: number; note_id: number };

/**
 * Map a set of selected card ids to the distinct note ids they belong to, in
 * first-seen order. Deleting a note removes all its cards, so several selected
 * cards sharing a note collapse to one id. Card ids not in `cards` (stale
 * selection) are ignored because we iterate the current list.
 */
export function selectedNoteIds(
  cards: SelectableCard[],
  selectedIds: Set<number>,
): number[] {
  const seen = new Set<number>();
  const out: number[] = [];
  for (const c of cards) {
    if (selectedIds.has(c.id) && !seen.has(c.note_id)) {
      seen.add(c.note_id);
      out.push(c.note_id);
    }
  }
  return out;
}

/** True when every visible card is selected. An empty list is never "all". */
export function allSelected(
  cards: SelectableCard[],
  selectedIds: Set<number>,
): boolean {
  return cards.length > 0 && cards.every((c) => selectedIds.has(c.id));
}

/** True when at least one visible card is selected. */
export function someSelected(
  cards: SelectableCard[],
  selectedIds: Set<number>,
): boolean {
  return cards.some((c) => selectedIds.has(c.id));
}

/** Every visible card id — the target of "select all" / ⌘A. */
export function allCardIds(cards: SelectableCard[]): Set<number> {
  return new Set(cards.map((c) => c.id));
}
