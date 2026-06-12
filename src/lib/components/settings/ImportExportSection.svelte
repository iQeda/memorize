<script lang="ts">
  import { AlertCircle, Loader2, Package, FilePlus2 } from "lucide-svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import { pkg } from "$lib/stores/package.svelte";
  import { importFlow } from "$lib/stores/import-flow.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import SettingsSection from "./SettingsSection.svelte";
  import SettingRow from "./SettingRow.svelte";

  let exportWithScheduling = $state(false);
  let exportWithMedia = $state(true);
  let exportWithDeckConfigs = $state(true);

  async function handleExportAll() {
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const stamp = new Date()
        .toISOString()
        .replace(/[-:T]/g, "")
        .replace(/\.\d+Z$/, "");
      const picked = await save({
        defaultPath: `memorize-all-${stamp}.apkg`,
        filters: [{ name: "Anki package", extensions: ["apkg"] }],
      });
      if (typeof picked !== "string") return;
      await pkg.exportAll({
        outPath: picked,
        withScheduling: exportWithScheduling,
        withMedia: exportWithMedia,
        withDeckConfigs: exportWithDeckConfigs,
        legacy: false,
      });
    } catch (e) {
      console.error(e);
    }
  }
</script>

{#snippet checkbox(label: string, value: boolean, onChange: (v: boolean) => void)}
  <label class="flex cursor-pointer items-center gap-1.5 text-(--color-fg-default)">
    <input
      type="checkbox"
      checked={value}
      onchange={(e) => onChange((e.currentTarget as HTMLInputElement).checked)}
      class="h-3.5 w-3.5 rounded border border-(--color-border-strong) accent-(--color-accent-500)"
    />
    <span>{label}</span>
  </label>
{/snippet}

<SettingsSection id="io" title={t("io.title")}>
  <div
    class="rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-5 shadow-(--shadow-subtle)"
  >
    <SettingRow
      icon={FilePlus2}
      label={t("io.importLabel")}
      subtitle={t("io.importBody")}
    >
      {#snippet action()}
        <button
          type="button"
          onclick={() => importFlow.start()}
          disabled={pkg.busy || !collection.isOpen}
          class="flex shrink-0 items-center gap-1.5 rounded-(--radius-md) bg-(--color-accent-500) px-3 py-1.5 text-xs font-medium text-(--color-fg-onAccent) shadow-(--shadow-subtle) transition-all hover:bg-(--color-accent-600) hover:shadow-(--shadow-card) active:scale-[0.97] disabled:cursor-not-allowed disabled:bg-(--color-bg-overlay) disabled:text-(--color-fg-subtle) disabled:shadow-none"
        >
          {#if pkg.busy}
            <Loader2 size={12} class="animate-spin" />
          {:else}
            <FilePlus2 size={12} />
          {/if}
          {t("io.importPick")}
        </button>
      {/snippet}
    </SettingRow>

    {#if pkg.lastImport}
      <div class="mt-3 grid grid-cols-3 gap-2 text-xs sm:grid-cols-6">
        {#each [
          { label: t("io.statNew"), value: pkg.lastImport.new },
          { label: t("io.statUpdated"), value: pkg.lastImport.updated },
          { label: t("io.statMatched"), value: pkg.lastImport.first_field_match },
          { label: t("io.statDuplicate"), value: pkg.lastImport.duplicate },
          { label: t("io.statConflicting"), value: pkg.lastImport.conflicting },
          { label: t("io.statFound"), value: pkg.lastImport.found_notes }
        ] as stat (stat.label)}
          <div class="rounded-(--radius-sm) bg-(--color-bg-overlay) px-2 py-1.5 text-center">
            <p class="text-[10px] tracking-wider text-(--color-fg-subtle) uppercase">
              {stat.label}
            </p>
            <p class="number-tabular text-sm text-(--color-fg-default)">
              {stat.value}
            </p>
          </div>
        {/each}
      </div>
    {/if}

    <hr class="my-4 border-(--color-border-default)" />

    <div class="space-y-3">
      <SettingRow
        icon={Package}
        label={t("io.exportLabel")}
        subtitle={t("io.exportBody")}
      >
        {#snippet action()}
          <button
            type="button"
            onclick={handleExportAll}
            disabled={pkg.busy || !collection.isOpen}
            class="flex shrink-0 items-center gap-1.5 rounded-(--radius-md) border border-(--color-border-strong) px-3 py-1.5 text-xs text-(--color-fg-default) transition-colors hover:bg-(--color-bg-overlay) active:scale-[0.98] disabled:opacity-50"
          >
            {#if pkg.busy}
              <Loader2 size={12} class="animate-spin" />
            {:else}
              <Package size={12} />
            {/if}
            {t("io.exportButton")}
          </button>
        {/snippet}
      </SettingRow>

      <div class="flex flex-wrap gap-x-5 gap-y-2 pl-7 text-xs">
        {@render checkbox(t("io.includeMedia"), exportWithMedia, (v) => (exportWithMedia = v))}
        {@render checkbox(t("io.includeScheduling"), exportWithScheduling, (v) => (exportWithScheduling = v))}
        {@render checkbox(t("io.includeDeckConfigs"), exportWithDeckConfigs, (v) => (exportWithDeckConfigs = v))}
      </div>
    </div>

    {#if pkg.lastExportPath}
      <p class="mt-3 truncate font-mono text-[11px] text-(--color-fg-subtle)">
        {t("io.lastExport", { path: pkg.lastExportPath })}
      </p>
    {/if}
    {#if pkg.lastError}
      <p class="mt-3 flex items-start gap-1.5 text-xs text-(--color-danger)">
        <AlertCircle size={12} class="mt-0.5 shrink-0" />
        <span class="break-all">{pkg.lastError}</span>
      </p>
    {/if}
  </div>
</SettingsSection>
