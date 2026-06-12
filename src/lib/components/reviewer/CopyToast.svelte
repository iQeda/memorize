<script lang="ts">
  // Copy 失敗エラー / 一時情報のトースト。error 優先で表示する。
  import { Copy, X } from "lucide-svelte";

  type Props = {
    error: string | null;
    info: string | null;
    onCopyError: () => void;
    onDismiss: () => void;
  };
  let { error, info, onCopyError, onDismiss }: Props = $props();
</script>

{#if error}
  <div
    role="alert"
    class="pointer-events-auto fixed bottom-6 left-1/2 z-20 flex max-w-lg -translate-x-1/2 gap-2 rounded-(--radius-md) border border-(--color-danger)/40 bg-(--color-danger)/10 px-4 py-2.5 text-xs text-(--color-danger) shadow-(--shadow-card) select-text"
  >
    <div class="min-w-0 flex-1">
      <p class="font-medium">Copy failed</p>
      <p class="mt-0.5 font-mono text-[11px] break-all opacity-90 select-all">{error}</p>
    </div>
    <div class="flex shrink-0 flex-col gap-1">
      <button
        type="button"
        onclick={onCopyError}
        aria-label="Copy error"
        title="Copy error"
        class="grid h-5 w-5 place-items-center rounded text-(--color-danger) transition-colors hover:bg-(--color-danger)/20"
      >
        <Copy size={12} />
      </button>
      <button
        type="button"
        onclick={onDismiss}
        aria-label="Dismiss"
        title="Dismiss"
        class="grid h-5 w-5 place-items-center rounded text-(--color-danger) transition-colors hover:bg-(--color-danger)/20"
      >
        <X size={12} />
      </button>
    </div>
  </div>
{:else if info}
  <div
    role="status"
    class="pointer-events-auto fixed bottom-6 left-1/2 z-20 -translate-x-1/2 rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-elevated) px-3 py-1.5 text-xs text-(--color-fg-default) shadow-(--shadow-card) select-text"
  >
    {info}
  </div>
{/if}
