// Svelte action that turns the host element into a window drag handle.
// Falls back gracefully when not running inside Tauri.

import { getCurrentWindow } from "@tauri-apps/api/window";

const interactiveTags = new Set([
  "BUTTON",
  "A",
  "INPUT",
  "TEXTAREA",
  "SELECT",
  "LABEL",
]);

function isInteractive(el: EventTarget | null): boolean {
  let node = el as HTMLElement | null;
  while (node && node !== document.body) {
    if (interactiveTags.has(node.tagName)) return true;
    if (node.dataset.noDrag === "" || node.dataset.noDrag === "true") return true;
    node = node.parentElement;
  }
  return false;
}

export function draggable(node: HTMLElement) {
  const onMouseDown = (event: MouseEvent) => {
    if (event.button !== 0) return;
    if (isInteractive(event.target)) return;
    event.preventDefault();
    void getCurrentWindow().startDragging();
  };

  const onDoubleClick = (event: MouseEvent) => {
    if (isInteractive(event.target)) return;
    void getCurrentWindow().toggleMaximize();
  };

  node.addEventListener("mousedown", onMouseDown);
  node.addEventListener("dblclick", onDoubleClick);

  return {
    destroy() {
      node.removeEventListener("mousedown", onMouseDown);
      node.removeEventListener("dblclick", onDoubleClick);
    },
  };
}
