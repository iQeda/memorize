<script lang="ts">
  import { FolderOpen, FilePlus2 } from "lucide-svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import AnkiDesktopSuggestion from "$lib/components/AnkiDesktopSuggestion.svelte";
  import SettingsSection from "./SettingsSection.svelte";
  import { t } from "$lib/i18n/index.svelte";
</script>

<SettingsSection id="collection" title={t("settings.collection")} first>
  <div
    class="rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-4 shadow-(--shadow-subtle)"
  >
    {#if collection.isOpen}
      <p class="text-sm text-(--color-fg-default)">
        {t("settings.collectionOpenedWithDecks", { count: collection.decks.length })}
      </p>
      {#if collection.currentPath}
        <p class="mt-1 truncate font-mono text-[11px] text-(--color-fg-subtle)">
          {collection.currentPath}
        </p>
      {/if}
      <button
        type="button"
        onclick={() => collection.close()}
        class="mt-3 rounded-(--radius-md) border border-(--color-border-strong) px-3 py-1.5 text-sm text-(--color-fg-default) transition-colors hover:bg-(--color-bg-overlay) active:scale-[0.98]"
      >
        {t("settings.closeCollection")}
      </button>
    {:else}
      <p class="text-sm text-(--color-fg-muted)">{t("settings.collectionNotOpen")}</p>
      <div class="mt-3 flex flex-wrap gap-2">
        <button
          type="button"
          onclick={() => collection.pickAndOpen()}
          class="flex items-center gap-1.5 rounded-(--radius-md) bg-(--color-accent-500) px-3 py-1.5 text-xs font-medium text-(--color-fg-onAccent) shadow-(--shadow-subtle) transition-colors hover:bg-(--color-accent-600) active:scale-[0.97]"
        >
          <FolderOpen size={12} />
          {t("welcome.openExisting")}
        </button>
        <button
          type="button"
          onclick={() => collection.createNew()}
          class="flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-border-strong) px-3 py-1.5 text-xs text-(--color-fg-default) transition-colors hover:bg-(--color-bg-overlay) active:scale-[0.98]"
        >
          <FilePlus2 size={12} />
          {t("welcome.createNew")}
        </button>
      </div>
    {/if}

    <AnkiDesktopSuggestion class="mt-4" />
  </div>
</SettingsSection>
