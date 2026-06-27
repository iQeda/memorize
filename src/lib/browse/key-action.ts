/** Inputs the Browse keyboard handler needs to decide what a keypress does. */
export type KeyContext = {
  /** event.metaKey (⌘ on macOS) */
  meta: boolean;
  /** event.ctrlKey — rejected so ⌘ and Ctrl don't both bind (repo convention) */
  ctrl: boolean;
  /** event.key */
  key: string;
  /** focus is in a TEXT-entry field (text input / search / textarea) — NOT a checkbox/radio */
  inTextField: boolean;
  /** the NoteEditor modal is open (it owns its own keys) */
  editorOpen: boolean;
  /** there is at least one card visible */
  hasCards: boolean;
  /** there is at least one selected card */
  hasSelection: boolean;
};

/**
 * Decide which Browse selection action a keypress maps to.
 * - ⌘A (not Ctrl): select all visible cards, unless focus is in a text field
 *   (let it select text) or the list is empty.
 * - Esc: clear the selection when one exists.
 * While the editor is open, the Browse view yields all keys to it.
 */
export function keyAction(ctx: KeyContext): "selectAll" | "clear" | null {
  if (ctx.editorOpen) return null;
  if (ctx.meta && !ctx.ctrl && ctx.key.toLowerCase() === "a") {
    if (ctx.inTextField) return null;
    if (!ctx.hasCards) return null;
    return "selectAll";
  }
  if (ctx.key === "Escape" && ctx.hasSelection) return "clear";
  return null;
}
