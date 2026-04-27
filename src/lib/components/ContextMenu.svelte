<script lang="ts">
  import { fade } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { onMount, type Snippet } from "svelte";

  type Props = {
    x: number;
    y: number;
    onClose: () => void;
    children: Snippet;
  };
  let { x, y, onClose, children }: Props = $props();

  let menuEl = $state<HTMLDivElement | null>(null);
  let pos = $state({ x: 0, y: 0 });

  onMount(() => {
    if (!menuEl) {
      pos = { x, y };
      return;
    }
    const rect = menuEl.getBoundingClientRect();
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    let nx = x;
    let ny = y;
    if (nx + rect.width > vw - 8) nx = vw - rect.width - 8;
    if (ny + rect.height > vh - 8) ny = vh - rect.height - 8;
    pos = { x: nx, y: ny };
  });

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

<svelte:window onkeydown={onKey} />

<div
  class="fixed inset-0 z-40"
  onclick={onClose}
  oncontextmenu={(e) => {
    e.preventDefault();
    onClose();
  }}
  onkeydown={() => {}}
  role="presentation"
></div>

<div
  bind:this={menuEl}
  in:fade={{ duration: 80, easing: cubicOut }}
  style="left: {pos.x}px; top: {pos.y}px;"
  class="fixed z-50 min-w-[160px] overflow-hidden rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-elevated) py-1 shadow-(--shadow-card)"
  role="menu"
>
  {@render children()}
</div>
