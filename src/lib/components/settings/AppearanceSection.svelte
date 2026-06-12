<script lang="ts">
  import { theme, type Theme } from "$lib/stores/theme.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import SettingsSection from "./SettingsSection.svelte";

  const themeOptions = $derived<{ value: Theme; label: string }[]>([
    { value: "light", label: t("settings.themeLight") },
    { value: "dark", label: t("settings.themeDark") },
    { value: "system", label: t("settings.themeSystem") },
  ]);
</script>

<SettingsSection id="appearance" title={t("settings.appearance")}>
  <div
    class="rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-1 shadow-(--shadow-subtle)"
  >
    <div class="grid grid-cols-3 gap-1">
      {#each themeOptions as opt (opt.value)}
        {@const active = theme.preference === opt.value}
        <button
          type="button"
          onclick={() => theme.set(opt.value)}
          class="rounded-(--radius-md) px-3 py-2 text-sm transition-colors
            {active
            ? 'bg-(--color-accent-500) text-(--color-fg-onAccent) shadow-(--shadow-subtle)'
            : 'text-(--color-fg-muted) hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)'}"
        >
          {opt.label}
        </button>
      {/each}
    </div>
  </div>
</SettingsSection>
