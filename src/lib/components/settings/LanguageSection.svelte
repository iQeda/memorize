<script lang="ts">
  import { i18n, t, type Locale } from "$lib/i18n/index.svelte";
  import SettingsSection from "./SettingsSection.svelte";

  const localeOptions: { value: Locale; label: string }[] = [
    { value: "en", label: "English" },
    { value: "ja", label: "日本語" },
  ];
</script>

<SettingsSection id="language" title={t("settings.language")} first>
  <div
    class="rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-1 shadow-(--shadow-subtle)"
  >
    <div class="grid grid-cols-2 gap-1">
      {#each localeOptions as opt (opt.value)}
        {@const active = i18n.locale === opt.value}
        <button
          type="button"
          onclick={() => i18n.set(opt.value)}
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
