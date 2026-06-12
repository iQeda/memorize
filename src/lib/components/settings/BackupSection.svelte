<script lang="ts">
  import { Shield, History, Save } from "lucide-svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import { sync } from "$lib/stores/sync.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import SettingsSection from "./SettingsSection.svelte";
  import SettingRow from "./SettingRow.svelte";
  import ToggleSwitch from "$lib/components/ToggleSwitch.svelte";

  async function handleManualBackup(includeMedia: boolean) {
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const stamp = new Date()
        .toISOString()
        .replace(/[-:T]/g, "")
        .replace(/\.\d+Z$/, "");
      const picked = await save({
        defaultPath: `memorize-${stamp}.colpkg`,
        filters: [{ name: "Anki collection package", extensions: ["colpkg"] }],
      });
      if (typeof picked === "string") {
        await sync.manualBackup(picked, includeMedia);
      }
    } catch (e) {
      console.error(e);
    }
  }

  async function handleRestore() {
    try {
      const { open, confirm } = await import("@tauri-apps/plugin-dialog");
      const picked = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "Anki collection package", extensions: ["colpkg"] }],
      });
      if (typeof picked !== "string") return;

      const ok = await confirm(t("backup.restoreConfirmBody", { path: picked }), {
        title: t("backup.restoreConfirmTitle"),
        kind: "warning",
        okLabel: t("backup.restoreOk"),
        cancelLabel: t("backup.restoreCancel"),
      });
      if (!ok) return;

      await sync.restore(picked);
    } catch (e) {
      console.error(e);
    }
  }
</script>

<SettingsSection id="backup" title={t("backup.title")}>
  <div
    class="rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-5 shadow-(--shadow-subtle)"
  >
    <SettingRow
      icon={Shield}
      iconClass="text-(--color-success)"
      label={t("backup.autoLabel")}
      subtitle={t("backup.autoBody")}
    >
      {#snippet action()}
        <ToggleSwitch
          checked={sync.autoBackupBeforeSync}
          onToggle={(v) => sync.setAutoBackup(v)}
          label={t("backup.autoLabel")}
        />
      {/snippet}
    </SettingRow>

    <div class="mt-4 flex flex-wrap gap-2">
      <button
        type="button"
        onclick={() => handleManualBackup(false)}
        disabled={sync.busy || !collection.isOpen}
        class="flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-border-strong) px-3 py-1.5 text-xs text-(--color-fg-default) transition-colors hover:bg-(--color-bg-overlay) active:scale-[0.98] disabled:opacity-50"
      >
        <Save size={12} />
        {t("backup.now")}
      </button>
      <button
        type="button"
        onclick={() => handleManualBackup(true)}
        disabled={sync.busy || !collection.isOpen}
        class="flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-border-strong) px-3 py-1.5 text-xs text-(--color-fg-default) transition-colors hover:bg-(--color-bg-overlay) active:scale-[0.98] disabled:opacity-50"
      >
        <Save size={12} />
        {t("backup.nowWithMedia")}
      </button>
    </div>

    {#if !collection.isOpen}
      <p class="mt-3 text-xs text-(--color-fg-subtle)">
        {t("backup.collectionRequired")}
      </p>
    {/if}
    {#if sync.lastBackupPath}
      <p class="mt-3 truncate font-mono text-[11px] text-(--color-fg-subtle)">
        {t("backup.lastPath", { path: sync.lastBackupPath })}
      </p>
    {/if}

    <hr class="my-4 border-(--color-border-default)" />

    <SettingRow
      icon={History}
      iconClass="text-(--color-warning)"
      label={t("backup.restoreLabel")}
      subtitle={t("backup.restoreBody")}
    >
      {#snippet action()}
        <button
          type="button"
          onclick={handleRestore}
          disabled={sync.busy || !collection.isOpen}
          class="flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-warning)/40 bg-(--color-warning)/10 px-3 py-1.5 text-xs font-medium text-(--color-warning) hover:bg-(--color-warning)/20 disabled:opacity-50"
        >
          <History size={12} />
          {t("backup.restoreButton")}
        </button>
      {/snippet}
    </SettingRow>
  </div>
</SettingsSection>
