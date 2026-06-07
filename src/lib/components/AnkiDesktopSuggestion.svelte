<script lang="ts">
  import { collection } from "$lib/stores/collection.svelte";
  import { t } from "$lib/i18n/index.svelte";

  // Extra classes (margin / alignment) so the same card fits both the
  // Settings panel and the centered welcome screen.
  let { class: klass = "" }: { class?: string } = $props();
</script>

{#if collection.ankiDesktopSuggestion}
  <div
    class="rounded-(--radius-md) border border-(--color-accent-500)/40 bg-(--color-accent-500)/8 p-3 text-xs {klass}"
  >
    <p class="text-(--color-fg-default)">
      {t("settings.ankiDesktopDetected")}
    </p>
    <p class="mt-1 truncate font-mono text-[11px] text-(--color-fg-subtle)">
      {collection.ankiDesktopSuggestion}
    </p>
    <p class="mt-2 leading-relaxed whitespace-pre-line text-(--color-fg-muted)">
      {t("settings.ankiDesktopHint")}
    </p>
    <button
      type="button"
      onclick={() => {
        const path = collection.ankiDesktopSuggestion;
        if (path) void collection.open(path);
      }}
      class="mt-2 rounded-(--radius-md) bg-(--color-accent-500) px-3 py-1.5 text-xs font-medium text-(--color-fg-onAccent) hover:bg-(--color-accent-600) active:scale-[0.97]"
    >
      {t("settings.switchToThis")}
    </button>
  </div>
{/if}
