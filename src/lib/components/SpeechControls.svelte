<script lang="ts">
  // speech 設定 UI の単一実装。settings (layout="rows") と reviewer の
  // Audio popover (layout="popover") の二重実装をここに統合する。
  // 共有 7 コントロール: 自動読み上げ / repeat-on-start / maxRepeat /
  // interval / rate+プレビュー / volume+プレビュー / sentence pause。
  import { Volume2, Repeat } from "lucide-svelte";
  import { speech, SPEECH_LIMITS } from "$lib/stores/speech.svelte";
  import { invoke } from "$lib/ipc";
  import { t } from "$lib/i18n/index.svelte";
  import ToggleSwitch from "$lib/components/ToggleSwitch.svelte";
  import SettingRow from "$lib/components/settings/SettingRow.svelte";

  type Props = { layout: "rows" | "popover" };
  let { layout }: Props = $props();

  /** settings に完全重複で 2 箇所あったプレビュー再生の単一ハンドラ。
   *  reviewer popover の同一 invoke もここに合流する。 */
  function preview() {
    void invoke<void>("start_speak_text", {
      text: t("settings.speech.ratePreviewText"),
      rate: speech.speechRate,
      sentencePauseMs: speech.sentencePauseMs,
      volume: speech.volume,
    }).catch((e) => console.error("sample play failed", e));
  }

  function intInput(apply: (n: number) => void) {
    return (e: Event) => {
      const next = Number.parseInt((e.currentTarget as HTMLInputElement).value, 10);
      if (Number.isFinite(next)) apply(next);
    };
  }

  function floatInput(apply: (n: number) => void) {
    return (e: Event) => {
      const next = Number.parseFloat((e.currentTarget as HTMLInputElement).value);
      if (Number.isFinite(next)) apply(next);
    };
  }
</script>

{#if layout === "rows"}
  <SettingRow
    icon={Volume2}
    label={t("settings.speech.autoLabel")}
    subtitle={t("settings.speech.autoBody")}
  >
    {#snippet action()}
      <ToggleSwitch
        checked={speech.speakQuestionOnShow}
        onToggle={(v) => speech.setSpeakQuestionOnShow(v)}
        label={t("settings.speech.autoLabel")}
      />
    {/snippet}
  </SettingRow>
  <SettingRow
    icon={Volume2}
    label={t("settings.speech.repeatOnStartLabel")}
    subtitle={t("settings.speech.repeatOnStartBody")}
    divider
  >
    {#snippet action()}
      <ToggleSwitch
        checked={speech.repeatOnQuestionStart}
        onToggle={(v) => speech.setRepeatOnQuestionStart(v)}
        label={t("settings.speech.repeatOnStartLabel")}
      />
    {/snippet}
  </SettingRow>
  <SettingRow
    icon={Repeat}
    label={t("settings.speech.maxRepeatLabel")}
    subtitle={t("settings.speech.maxRepeatBody")}
    divider
  >
    {#snippet action()}
      <div class="flex items-center gap-2">
        <input
          type="range"
          min={SPEECH_LIMITS.maxRepeat.min}
          max={SPEECH_LIMITS.maxRepeat.max}
          step="1"
          value={speech.maxRepeat}
          oninput={intInput((n) => speech.setMaxRepeat(n))}
          aria-label={t("settings.speech.maxRepeatLabel")}
          class="w-32 accent-(--color-accent-500)"
        />
        <span class="number-tabular w-10 text-right text-xs text-(--color-fg-muted)">
          {speech.maxRepeat}
        </span>
      </div>
    {/snippet}
  </SettingRow>
  <SettingRow
    icon={Repeat}
    label={t("settings.speech.repeatIntervalLabel")}
    subtitle={t("settings.speech.repeatIntervalBody")}
    divider
  >
    {#snippet action()}
      <div class="flex items-center gap-2">
        <input
          type="range"
          min={SPEECH_LIMITS.repeatIntervalSec.min}
          max={SPEECH_LIMITS.repeatIntervalSec.max}
          step="0.01"
          value={speech.repeatIntervalSec}
          oninput={floatInput((n) => speech.setRepeatIntervalSec(n))}
          aria-label={t("settings.speech.repeatIntervalLabel")}
          class="w-32 accent-(--color-accent-500)"
        />
        <span class="number-tabular w-10 text-right text-xs text-(--color-fg-muted)">
          {speech.repeatIntervalSec.toFixed(2)}s
        </span>
      </div>
    {/snippet}
  </SettingRow>
  <SettingRow
    icon={Volume2}
    label={t("settings.speech.rateLabel")}
    subtitle={t("settings.speech.rateBody")}
    divider
  >
    {#snippet action()}
      <div class="flex items-center gap-2">
        <input
          type="range"
          min={SPEECH_LIMITS.rateWpm.min}
          max={SPEECH_LIMITS.rateWpm.max}
          step="10"
          value={speech.speechRate}
          oninput={intInput((n) => speech.setSpeechRate(n))}
          aria-label={t("settings.speech.rateLabel")}
          class="w-32 accent-(--color-accent-500)"
        />
        <span class="number-tabular w-12 text-right text-xs text-(--color-fg-muted)">
          {speech.speechRate}
        </span>
        {@render previewButton()}
      </div>
    {/snippet}
  </SettingRow>
  <SettingRow
    icon={Volume2}
    label={t("settings.speech.volumeLabel")}
    subtitle={t("settings.speech.volumeBody")}
    divider
  >
    {#snippet action()}
      <div class="flex items-center gap-2">
        <input
          type="range"
          min={SPEECH_LIMITS.volume.min}
          max={SPEECH_LIMITS.volume.max}
          step="1"
          value={speech.volume}
          oninput={intInput((n) => speech.setVolume(n))}
          aria-label={t("settings.speech.volumeLabel")}
          class="w-32 accent-(--color-accent-500)"
        />
        <span class="number-tabular w-10 text-right text-xs text-(--color-fg-muted)">
          {speech.volume}%
        </span>
        {@render previewButton()}
      </div>
    {/snippet}
  </SettingRow>
  <SettingRow
    icon={Volume2}
    label={t("settings.speech.sentencePauseLabel")}
    subtitle={t("settings.speech.sentencePauseBody")}
    divider
  >
    {#snippet action()}
      <div class="flex items-center gap-2">
        <input
          type="range"
          min={SPEECH_LIMITS.sentencePauseMs.min}
          max={SPEECH_LIMITS.sentencePauseMs.max}
          step="100"
          value={speech.sentencePauseMs}
          oninput={intInput((n) => speech.setSentencePauseMs(n))}
          aria-label={t("settings.speech.sentencePauseLabel")}
          class="w-32 accent-(--color-accent-500)"
        />
        <span class="number-tabular w-14 text-right text-xs text-(--color-fg-muted)">
          {speech.sentencePauseMs}ms
        </span>
      </div>
    {/snippet}
  </SettingRow>
{:else}
  <!-- popover layout: reviewer の Audio popover 用コンパクト版。
       縦積みの小型コントロール (Phase 7 で reviewer から移植)。 -->
  <div class="space-y-3 text-xs">
    <label class="flex cursor-pointer items-center justify-between gap-3">
      <span class="text-(--color-fg-default)">{t("settings.speech.autoLabel")}</span>
      <input
        type="checkbox"
        checked={speech.speakQuestionOnShow}
        onchange={(e) =>
          speech.setSpeakQuestionOnShow((e.currentTarget as HTMLInputElement).checked)}
        class="h-3.5 w-3.5 accent-(--color-accent-500)"
      />
    </label>
    <label class="flex cursor-pointer items-center justify-between gap-3">
      <span class="text-(--color-fg-default)">{t("settings.speech.repeatOnStartLabel")}</span>
      <input
        type="checkbox"
        checked={speech.repeatOnQuestionStart}
        onchange={(e) =>
          speech.setRepeatOnQuestionStart((e.currentTarget as HTMLInputElement).checked)}
        class="h-3.5 w-3.5 accent-(--color-accent-500)"
      />
    </label>
    <div>
      <div class="flex items-center justify-between gap-3">
        <span class="text-(--color-fg-muted)">{t("settings.speech.maxRepeatLabel")}</span>
        <span class="number-tabular text-(--color-fg-default)">{speech.maxRepeat}</span>
      </div>
      <input
        type="range"
        min={SPEECH_LIMITS.maxRepeat.min}
        max={SPEECH_LIMITS.maxRepeat.max}
        step="1"
        value={speech.maxRepeat}
        oninput={intInput((n) => speech.setMaxRepeat(n))}
        aria-label={t("settings.speech.maxRepeatLabel")}
        class="mt-1 w-full accent-(--color-accent-500)"
      />
    </div>
    <div>
      <div class="flex items-center justify-between gap-3">
        <span class="text-(--color-fg-muted)">{t("settings.speech.repeatIntervalLabel")}</span>
        <span class="number-tabular text-(--color-fg-default)">{speech.repeatIntervalSec.toFixed(2)}s</span>
      </div>
      <input
        type="range"
        min={SPEECH_LIMITS.repeatIntervalSec.min}
        max={SPEECH_LIMITS.repeatIntervalSec.max}
        step="0.01"
        value={speech.repeatIntervalSec}
        oninput={floatInput((n) => speech.setRepeatIntervalSec(n))}
        aria-label={t("settings.speech.repeatIntervalLabel")}
        class="mt-1 w-full accent-(--color-accent-500)"
      />
    </div>
    <div>
      <div class="flex items-center justify-between gap-3">
        <span class="text-(--color-fg-muted)">{t("settings.speech.rateLabel")}</span>
        <span class="number-tabular text-(--color-fg-default)">{speech.speechRate}</span>
      </div>
      <input
        type="range"
        min={SPEECH_LIMITS.rateWpm.min}
        max={SPEECH_LIMITS.rateWpm.max}
        step="10"
        value={speech.speechRate}
        oninput={intInput((n) => speech.setSpeechRate(n))}
        aria-label={t("settings.speech.rateLabel")}
        class="mt-1 w-full accent-(--color-accent-500)"
      />
    </div>
    <div>
      <div class="flex items-center justify-between gap-3">
        <span class="text-(--color-fg-muted)">{t("settings.speech.volumeLabel")}</span>
        <span class="number-tabular text-(--color-fg-default)">{speech.volume}%</span>
      </div>
      <input
        type="range"
        min={SPEECH_LIMITS.volume.min}
        max={SPEECH_LIMITS.volume.max}
        step="1"
        value={speech.volume}
        oninput={intInput((n) => speech.setVolume(n))}
        aria-label={t("settings.speech.volumeLabel")}
        class="mt-1 w-full accent-(--color-accent-500)"
      />
    </div>
    <div>
      <div class="flex items-center justify-between gap-3">
        <span class="text-(--color-fg-muted)">{t("settings.speech.sentencePauseLabel")}</span>
        <span class="number-tabular text-(--color-fg-default)">{speech.sentencePauseMs}ms</span>
      </div>
      <input
        type="range"
        min={SPEECH_LIMITS.sentencePauseMs.min}
        max={SPEECH_LIMITS.sentencePauseMs.max}
        step="100"
        value={speech.sentencePauseMs}
        oninput={intInput((n) => speech.setSentencePauseMs(n))}
        aria-label={t("settings.speech.sentencePauseLabel")}
        class="mt-1 w-full accent-(--color-accent-500)"
      />
    </div>
    {@render previewButton()}
  </div>
{/if}

{#snippet previewButton()}
  <button
    type="button"
    onclick={preview}
    aria-label={t("settings.speech.ratePreview")}
    class="flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-base) px-3 py-1 text-xs shadow-(--shadow-subtle) hover:bg-(--color-bg-overlay) active:scale-[0.97]"
  >
    <Volume2 size={12} strokeWidth={2.25} />
    {t("settings.speech.ratePreview")}
  </button>
{/snippet}
