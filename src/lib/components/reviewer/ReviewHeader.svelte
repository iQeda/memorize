<script lang="ts">
  // Reviewer ヘッダー: 戻る / 進捗カウント / Audio popover / 編集ボタン。
  import { ArrowLeft, Pencil } from "lucide-svelte";
  import { t } from "$lib/i18n/index.svelte";
  import type { Counts } from "$lib/reviewer/types";
  import AudioSettingsPopover from "./AudioSettingsPopover.svelte";

  type Props = {
    totals: Counts;
    cursor: number;
    initialTotal: number;
    totalDue: number;
    hasCard: boolean;
    onBack: () => void;
    onEdit: () => void;
    onRepeatToggled: () => void;
  };
  let {
    totals,
    cursor,
    initialTotal,
    totalDue,
    hasCard,
    onBack,
    onEdit,
    onRepeatToggled,
  }: Props = $props();

  const badges = $derived([
    { label: t("decks.new"), value: totals.new },
    { label: t("decks.learning"), value: totals.learning },
    { label: t("decks.review"), value: totals.review },
  ]);
</script>

<div class="flex items-center justify-between px-6 py-3">
  <button
    type="button"
    onclick={onBack}
    class="flex items-center gap-1.5 rounded-(--radius-md) px-2 py-1 text-sm text-(--color-fg-muted) transition-colors hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)"
  >
    <ArrowLeft size={14} />
    {t("reviewer.back")}
    <span class="ml-1 font-mono text-[10px] opacity-70">Esc</span>
  </button>
  <p class="flex items-center gap-6 text-xs text-(--color-fg-subtle)">
    <span class="number-tabular">
      {cursor + (hasCard ? 1 : 0)} / {initialTotal || totalDue || "—"}
    </span>
    <span class="hidden items-center gap-1.5 sm:flex">
      {#each badges as b, i (i)}
        <span
          class="inline-flex items-center gap-1.5 rounded-full border border-(--color-border-default) bg-(--color-bg-elevated) px-2 py-0.5"
        >
          <span class="text-[10px] font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
            {b.label}
          </span>
          <span class="number-tabular text-xs font-medium text-(--color-fg-default)">{b.value}</span>
        </span>
      {/each}
    </span>
  </p>
  <div class="flex items-center gap-1">
    {#if hasCard}
      <AudioSettingsPopover {onRepeatToggled} />
      <button
        type="button"
        onclick={onEdit}
        class="flex h-7 items-center gap-1.5 rounded-(--radius-md) px-2 text-(--color-fg-muted) transition-colors hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)"
        aria-label={t("settings.shortcut.editNote")}
        title="{t('settings.shortcut.editNote')} (E)"
      >
        <Pencil size={14} />
        <span class="font-mono text-[10px] opacity-70">E</span>
      </button>
    {/if}
  </div>
</div>
