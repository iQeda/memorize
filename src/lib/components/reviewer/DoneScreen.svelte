<script lang="ts">
  // セッション完了画面。
  import { scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { t } from "$lib/i18n/index.svelte";

  type Props = { onBack: () => void };
  let { onBack }: Props = $props();
</script>

<div in:scale={{ duration: 240, start: 0.92, easing: cubicOut }} class="text-center">
  <p class="font-display text-3xl font-medium">{t("reviewer.done")}</p>
  <p class="mt-2 text-sm text-(--color-fg-muted)">{t("reviewer.sessionFinished")}</p>
  <button
    type="button"
    onclick={onBack}
    class="mt-6 inline-flex items-center gap-2 rounded-(--radius-md) border border-(--color-border-strong) px-4 py-1.5 text-sm transition-colors hover:bg-(--color-bg-overlay)"
  >
    {t("reviewer.backToDecks")}
    <span class="font-mono text-[10px] opacity-70">↵ / Space</span>
  </button>
</div>
