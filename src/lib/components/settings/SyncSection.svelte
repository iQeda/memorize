<script lang="ts">
  import {
    CheckCircle2,
    AlertCircle,
    Loader2,
    RefreshCw,
    LogOut,
    Upload,
    Download,
  } from "lucide-svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import { sync } from "$lib/stores/sync.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import SettingsSection from "./SettingsSection.svelte";
  import SettingRow from "./SettingRow.svelte";
  import ToggleSwitch from "$lib/components/ToggleSwitch.svelte";

  let username = $state("");
  let password = $state("");
  let endpoint = $state("");

  async function handleLogin(e: SubmitEvent) {
    e.preventDefault();
    try {
      await sync.login(username, password, endpoint || undefined);
      password = "";
    } catch {}
  }

  async function confirmFullSync(direction: "upload" | "download") {
    const { confirm } = await import("@tauri-apps/plugin-dialog");
    const ok = await confirm(
      direction === "upload"
        ? t("sync.uploadConfirmBody")
        : t("sync.downloadConfirmBody"),
      {
        title:
          direction === "upload"
            ? t("sync.uploadConfirmTitle")
            : t("sync.downloadConfirmTitle"),
        kind: "warning",
        okLabel:
          direction === "upload" ? t("sync.uploadOk") : t("sync.downloadOk"),
        cancelLabel: t("sync.cancel"),
      },
    );
    if (!ok) return;
    if (direction === "upload") await sync.fullUpload();
    else await sync.fullDownload();
  }
</script>

<SettingsSection id="sync" title={t("sync.title")}>
  <div
    class="rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-5 shadow-(--shadow-subtle)"
  >
    {#if sync.loggedIn}
      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-2.5">
          <CheckCircle2 size={16} class="text-(--color-success)" />
          <span class="text-sm">
            {t("sync.signedInAs", { username: sync.username ?? "" })}
          </span>
        </div>
        <button
          type="button"
          onclick={() => sync.logout()}
          disabled={sync.busy}
          class="flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-border-strong) px-3 py-1.5 text-xs text-(--color-fg-default) transition-colors hover:bg-(--color-bg-overlay) active:scale-[0.98] disabled:opacity-50"
        >
          <LogOut size={12} />
          {t("sync.logout")}
        </button>
      </div>

      <div class="mt-4 flex flex-wrap gap-2">
        <button
          type="button"
          onclick={() => sync.syncNow()}
          disabled={sync.busy || !collection.isOpen}
          class="flex items-center gap-1.5 rounded-(--radius-md) bg-(--color-accent-500) px-4 py-2 text-sm font-medium text-(--color-fg-onAccent) shadow-(--shadow-subtle) transition-all hover:bg-(--color-accent-600) hover:shadow-(--shadow-card) active:scale-[0.97] disabled:cursor-not-allowed disabled:bg-(--color-bg-overlay) disabled:text-(--color-fg-subtle) disabled:shadow-none"
        >
          {#if sync.busy}
            <Loader2 size={14} class="animate-spin" />
          {:else}
            <RefreshCw size={14} />
          {/if}
          {sync.busy && sync.busyReason ? sync.busyReason : t("sync.now")}
        </button>

        {#if sync.fullSyncRequired}
          {#if sync.fullSyncRequired.upload_ok}
            <button
              type="button"
              onclick={() => confirmFullSync("upload")}
              disabled={sync.busy}
              class="flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-warning)/40 bg-(--color-warning)/10 px-3 py-2 text-xs font-medium text-(--color-warning) hover:bg-(--color-warning)/20 disabled:opacity-50"
            >
              <Upload size={12} /> {t("sync.uploadWarn")}
            </button>
          {/if}
          {#if sync.fullSyncRequired.download_ok}
            <button
              type="button"
              onclick={() => confirmFullSync("download")}
              disabled={sync.busy}
              class="flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-warning)/40 bg-(--color-warning)/10 px-3 py-2 text-xs font-medium text-(--color-warning) hover:bg-(--color-warning)/20 disabled:opacity-50"
            >
              <Download size={12} /> {t("sync.downloadWarn")}
            </button>
          {/if}
        {/if}
      </div>

      <SettingRow
        icon={RefreshCw}
        label={t("sync.autoLabel")}
        subtitle={t("sync.autoBody")}
        divider
      >
        {#snippet action()}
          <ToggleSwitch
            checked={sync.autoSyncOnStartStop}
            onToggle={(v) => sync.setAutoSyncOnStartStop(v)}
            label={t("sync.autoLabel")}
          />
        {/snippet}
      </SettingRow>

      {#if !collection.isOpen}
        <p class="mt-3 text-xs text-(--color-fg-subtle)">{t("sync.collectionRequired")}</p>
      {/if}
      {#if sync.lastMessage}
        <p class="mt-3 flex items-center gap-1.5 text-xs text-(--color-fg-muted)">
          <CheckCircle2 size={12} />
          {sync.lastMessage}
        </p>
      {/if}
      {#if sync.lastError}
        <p class="mt-3 flex items-start gap-1.5 text-xs text-(--color-danger)">
          <AlertCircle size={12} class="mt-0.5 shrink-0" />
          <span class="break-all">{sync.lastError}</span>
        </p>
      {/if}
    {:else}
      <form onsubmit={handleLogin} class="space-y-3">
        <p class="text-xs text-(--color-fg-muted)">{t("sync.loginIntro")}</p>
        <label class="block">
          <span class="mb-1 block text-xs text-(--color-fg-muted)">{t("sync.username")}</span>
          <input
            type="text"
            autocomplete="username"
            required
            bind:value={username}
            class="w-full rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-base) px-3 py-1.5 text-sm shadow-(--shadow-subtle) outline-none focus:border-(--color-accent-500)"
          />
        </label>
        <label class="block">
          <span class="mb-1 block text-xs text-(--color-fg-muted)">{t("sync.password")}</span>
          <input
            type="password"
            autocomplete="current-password"
            required
            bind:value={password}
            class="w-full rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-base) px-3 py-1.5 text-sm shadow-(--shadow-subtle) outline-none focus:border-(--color-accent-500)"
          />
        </label>
        <details class="text-xs text-(--color-fg-subtle)">
          <summary class="cursor-pointer">{t("sync.customEndpoint")}</summary>
          <input
            type="text"
            placeholder="https://sync.ankiweb.net/"
            bind:value={endpoint}
            class="mt-2 w-full rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-base) px-3 py-1.5 text-sm shadow-(--shadow-subtle) outline-none focus:border-(--color-accent-500)"
          />
        </details>
        <button
          type="submit"
          disabled={sync.busy}
          class="flex items-center gap-1.5 rounded-(--radius-md) bg-(--color-accent-500) px-4 py-2 text-sm font-medium text-(--color-fg-onAccent) shadow-(--shadow-subtle) transition-all hover:bg-(--color-accent-600) hover:shadow-(--shadow-card) active:scale-[0.97] disabled:opacity-60"
        >
          {#if sync.busy}
            <Loader2 size={14} class="animate-spin" />
          {/if}
          {t("sync.login")}
        </button>
        {#if sync.lastError}
          <p class="flex items-start gap-1.5 text-xs text-(--color-danger)">
            <AlertCircle size={12} class="mt-0.5 shrink-0" />
            <span class="break-all">{sync.lastError}</span>
          </p>
        {/if}
      </form>
    {/if}
  </div>
</SettingsSection>
