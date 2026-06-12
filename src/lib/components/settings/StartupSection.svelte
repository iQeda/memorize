<script lang="ts">
  import { onMount } from "svelte";
  import { Power, AlertCircle } from "lucide-svelte";
  import { t } from "$lib/i18n/index.svelte";
  import SettingsSection from "./SettingsSection.svelte";
  import SettingRow from "./SettingRow.svelte";
  import ToggleSwitch from "$lib/components/ToggleSwitch.svelte";

  let autostartEnabled = $state(false);
  let autostartReady = $state(false);
  let autostartBusy = $state(false);
  let autostartError = $state<string | null>(null);

  async function refreshAutostart() {
    try {
      const { isEnabled } = await import("@tauri-apps/plugin-autostart");
      autostartEnabled = await isEnabled();
      autostartReady = true;
    } catch (e) {
      console.error(e);
      autostartError = e instanceof Error ? e.message : String(e);
    }
  }

  async function toggleAutostart() {
    if (autostartBusy) return;
    autostartBusy = true;
    autostartError = null;
    try {
      const { enable, disable } = await import("@tauri-apps/plugin-autostart");
      if (autostartEnabled) {
        await disable();
        autostartEnabled = false;
      } else {
        await enable();
        autostartEnabled = true;
      }
    } catch (e) {
      console.error(e);
      autostartError = e instanceof Error ? e.message : String(e);
      // Re-sync from OS in case the toggle partially succeeded.
      await refreshAutostart();
    } finally {
      autostartBusy = false;
    }
  }

  onMount(() => {
    void refreshAutostart();
  });
</script>

<SettingsSection id="startup" title={t("settings.startup")}>
  <div
    class="rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-5 shadow-(--shadow-subtle)"
  >
    <SettingRow
      icon={Power}
      label={t("settings.startup.autoLabel")}
      subtitle={t("settings.startup.autoBody")}
    >
      {#snippet action()}
        <ToggleSwitch
          checked={autostartEnabled}
          onToggle={() => toggleAutostart()}
          label={t("settings.startup.autoLabel")}
          disabled={!autostartReady || autostartBusy}
        />
      {/snippet}
    </SettingRow>
    {#if autostartError}
      <p class="mt-3 flex items-start gap-1.5 text-xs text-(--color-danger)">
        <AlertCircle size={12} class="mt-0.5 shrink-0" />
        <span class="break-all">{autostartError}</span>
      </p>
    {/if}
  </div>
</SettingsSection>
