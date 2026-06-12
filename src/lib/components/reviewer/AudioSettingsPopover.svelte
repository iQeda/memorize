<script lang="ts">
  // Reviewer ヘッダーの Audio popover。speech 設定 UI は SpeechControls
  // (layout="popover") に委譲し、reviewer 固有のセッショントグル
  // (リピート再生 / リピート完了で自動表示) だけをここで持つ。
  // これで speech 設定 UI の二重実装 (settings との) が根絶される。
  import { Repeat, SlidersHorizontal } from "lucide-svelte";
  import { speech } from "$lib/stores/speech.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import SpeechControls from "$lib/components/SpeechControls.svelte";

  type Props = {
    /** リピートチェックの切替時 (OFF にしたら進行中の再再生予約を破棄)。 */
    onRepeatToggled: () => void;
  };
  let { onRepeatToggled }: Props = $props();

  let open = $state(false);

  // popover 外クリックで閉じる action。Svelte 5 でも `use:` action は同じ。
  function clickOutside(node: HTMLElement, callback: () => void) {
    const handler = (e: MouseEvent) => {
      if (!node.contains(e.target as Node)) callback();
    };
    document.addEventListener("mousedown", handler);
    return {
      destroy() {
        document.removeEventListener("mousedown", handler);
      },
    };
  }
</script>

<div class="relative" use:clickOutside={() => (open = false)}>
  <button
    type="button"
    onclick={() => (open = !open)}
    aria-haspopup="dialog"
    aria-expanded={open}
    class="flex h-7 items-center gap-1.5 rounded-(--radius-md) px-2 text-(--color-fg-muted) transition-colors hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default) {open
      ? 'bg-(--color-bg-overlay) text-(--color-fg-default)'
      : ''}"
    aria-label={t("reviewer.audioSettings")}
    title={t("reviewer.audioSettings")}
  >
    <SlidersHorizontal size={14} />
  </button>
  {#if open}
    <div
      class="audio-popover absolute right-0 top-9 z-50 max-h-[80vh] w-[400px] overflow-y-auto rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-5 shadow-(--shadow-card)"
      role="dialog"
      aria-label={t("reviewer.audioSettings")}
    >
      <p class="mb-4 text-xs font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
        {t("reviewer.audioSettings")}
      </p>
      <SpeechControls layout="popover" />
      <div class="my-4 border-t border-(--color-border-default)"></div>
      <!-- Repeat session toggle -->
      <label class="mb-3 flex cursor-pointer items-center justify-between gap-2 text-sm text-(--color-fg-default) select-none">
        <span class="flex items-center gap-2">
          <Repeat size={14} />
          {t("reviewer.repeat")}
        </span>
        <input
          type="checkbox"
          bind:checked={speech.repeat}
          onchange={onRepeatToggled}
          class="h-4 w-4 cursor-pointer accent-(--color-accent-500)"
          aria-label={t("reviewer.repeat")}
        />
      </label>
      <!-- Auto-reveal after repeat -->
      <label class="flex cursor-pointer items-center justify-between gap-2 text-sm text-(--color-fg-default) select-none">
        <span>{t("reviewer.autoRevealAfterRepeat")}</span>
        <input
          type="checkbox"
          checked={speech.autoRevealAfterRepeat}
          onchange={(e) =>
            speech.setAutoRevealAfterRepeat(
              (e.currentTarget as HTMLInputElement).checked,
            )}
          class="h-4 w-4 cursor-pointer accent-(--color-accent-500)"
        />
      </label>
    </div>
  {/if}
</div>
