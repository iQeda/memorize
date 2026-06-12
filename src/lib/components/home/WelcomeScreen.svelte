<script lang="ts">
  // コレクション未読み込み時のウェルカム画面。+page.svelte の
  // !collection.isOpen ブランチからの純移動。
  import { Brain, FolderOpen, FilePlus } from "lucide-svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import AnkiDesktopSuggestion from "$lib/components/AnkiDesktopSuggestion.svelte";
  import { t } from "$lib/i18n/index.svelte";

  const pickAndOpen = () => collection.pickAndOpen();
  const createNew = () => collection.createNew();
</script>

<div class="grid h-full place-items-center">
  <div class="flex max-w-md flex-col items-center gap-6 text-center">
    <div
      class="grid h-16 w-16 place-items-center rounded-2xl bg-(--color-accent-500) text-(--color-fg-onAccent) shadow-(--shadow-glow)"
    >
      <Brain size={32} strokeWidth={2.25} />
    </div>
    <div class="space-y-2">
      <h1 class="font-display text-3xl font-medium tracking-tight">
        {t("welcome.title")}
      </h1>
      <p class="text-sm leading-relaxed whitespace-pre-line text-(--color-fg-muted)">
        {t("welcome.body")}
      </p>
    </div>
    <div class="flex w-full flex-col gap-2">
      <button
        type="button"
        onclick={pickAndOpen}
        class="flex items-center justify-center gap-2 rounded-(--radius-md) bg-(--color-accent-500) px-5 py-2.5 text-sm font-medium whitespace-nowrap text-(--color-fg-onAccent) shadow-(--shadow-card) transition-all duration-200 hover:bg-(--color-accent-600) active:scale-[0.97]"
      >
        <FolderOpen size={16} strokeWidth={2.25} />
        {t("welcome.openExisting")}
      </button>
      <button
        type="button"
        onclick={createNew}
        class="flex items-center justify-center gap-2 rounded-(--radius-md) border border-(--color-border-strong) bg-(--color-bg-elevated) px-5 py-2.5 text-sm font-medium whitespace-nowrap text-(--color-fg-default) shadow-(--shadow-subtle) transition-all duration-200 hover:bg-(--color-bg-overlay) active:scale-[0.97]"
      >
        <FilePlus size={16} strokeWidth={2.25} />
        {t("welcome.createNew")}
      </button>
    </div>
    {#if collection.error}
      <p class="text-xs text-(--color-danger)">{collection.error}</p>
    {/if}
    <AnkiDesktopSuggestion class="w-full text-left" />
  </div>
</div>
