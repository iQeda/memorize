<script lang="ts">
  import { theme, type Theme } from "$lib/stores/theme.svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import { sync } from "$lib/stores/sync.svelte";
  import { pkg } from "$lib/stores/package.svelte";
  import { i18n, t, type Locale } from "$lib/i18n/index.svelte";
  import { shortcuts, type Action } from "$lib/stores/shortcuts.svelte";
  import { onMount } from "svelte";
  import {
    CheckCircle2,
    AlertCircle,
    Loader2,
    RefreshCw,
    LogOut,
    Upload,
    Download,
    Save,
    Shield,
    History,
    Package,
    FilePlus2,
    Power,
    DownloadCloud,
  } from "lucide-svelte";

  const localeOptions: { value: Locale; label: string }[] = [
    { value: "en", label: "English" },
    { value: "ja", label: "日本語" },
  ];

  const themeOptions = $derived<{ value: Theme; label: string }[]>([
    { value: "light", label: t("settings.themeLight") },
    { value: "dark", label: t("settings.themeDark") },
    { value: "system", label: t("settings.themeSystem") },
  ]);

  const ratingShortcuts = $derived<{ action: Action; label: string }[]>([
    { action: "again", label: t("settings.shortcut.again") },
    { action: "hard", label: t("settings.shortcut.hard") },
    { action: "good", label: t("settings.shortcut.good") },
    { action: "easy", label: t("settings.shortcut.easy") },
    { action: "nani", label: t("settings.shortcut.nani") },
  ]);

  const fixedShortcuts = $derived([
    { keys: ["Space"], label: t("settings.shortcut.spaceLabel") },
    { keys: ["⌘", ","], label: t("settings.shortcut.openSettings") },
  ]);

  let recordingFor = $state<Action | null>(null);

  function startRecord(action: Action) {
    recordingFor = action;
    const handler = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      window.removeEventListener("keydown", handler, true);
      if (e.key === "Escape") {
        recordingFor = null;
        return;
      }
      shortcuts.set(action, e.key);
      recordingFor = null;
    };
    window.addEventListener("keydown", handler, true);
  }

  let username = $state("");
  let password = $state("");
  let endpoint = $state("");

  // ---- Import / Export (.apkg) ----
  let exportWithScheduling = $state(false);
  let exportWithMedia = $state(true);
  let exportWithDeckConfigs = $state(true);

  // ---- Launch at login (autostart) ----
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

  // ---- Manual update check ----
  type UpdateInfo = {
    version: string;
    currentVersion: string;
    body?: string | null;
    downloadAndInstall: (
      onEvent: (event: { event: string; data?: { chunkLength?: number; contentLength?: number } }) => void,
    ) => Promise<void>;
  };

  let appVersion = $state<string | null>(null);
  let updateStatus = $state<
    "idle" | "checking" | "up-to-date" | "available" | "downloading" | "installing"
  >("idle");
  let updateInfo = $state<UpdateInfo | null>(null);
  let downloadProgress = $state<number | null>(null);
  let updateError = $state<string | null>(null);

  async function loadAppVersion() {
    try {
      const { getVersion } = await import("@tauri-apps/api/app");
      appVersion = await getVersion();
    } catch (e) {
      console.error(e);
    }
  }

  async function handleCheckUpdates() {
    if (updateStatus === "checking" || updateStatus === "downloading" || updateStatus === "installing") return;
    updateStatus = "checking";
    updateError = null;
    updateInfo = null;
    downloadProgress = null;
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const u = (await check()) as UpdateInfo | null;
      if (!u) {
        updateStatus = "up-to-date";
        return;
      }
      updateInfo = u;
      updateStatus = "available";
    } catch (e) {
      console.error(e);
      updateError = e instanceof Error ? e.message : String(e);
      updateStatus = "idle";
    }
  }

  async function handleInstallUpdate() {
    if (!updateInfo) return;
    updateError = null;
    updateStatus = "downloading";
    downloadProgress = 0;
    try {
      let total = 0;
      let received = 0;
      await updateInfo.downloadAndInstall((event) => {
        if (event.event === "Started" && event.data?.contentLength) {
          total = event.data.contentLength;
        } else if (event.event === "Progress" && event.data?.chunkLength) {
          received += event.data.chunkLength;
          if (total > 0) {
            downloadProgress = Math.min(100, Math.round((received / total) * 100));
          }
        } else if (event.event === "Finished") {
          downloadProgress = 100;
          updateStatus = "installing";
        }
      });
      const { relaunch } = await import("@tauri-apps/plugin-process");
      await relaunch();
    } catch (e) {
      console.error(e);
      updateError = e instanceof Error ? e.message : String(e);
      updateStatus = "available";
    }
  }

  onMount(() => {
    void sync.refresh();
    void refreshAutostart();
    void loadAppVersion();
  });

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

  async function handleImport() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const picked = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "Anki package", extensions: ["apkg"] }],
      });
      if (typeof picked !== "string") return;
      const r = await pkg.importApkg(picked);
      if (r) await collection.refreshDecks();
    } catch (e) {
      console.error(e);
    }
  }

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

<div class="mx-auto max-w-2xl px-8 py-10">
  <h1 class="font-display text-3xl font-medium tracking-tight">
    {t("settings.title")}
  </h1>

  <section class="mt-10 space-y-3">
    <h2 class="text-xs font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
      {t("settings.language")}
    </h2>
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
  </section>

  <section class="mt-10 space-y-3">
    <h2 class="text-xs font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
      {t("backup.title")}
    </h2>
    <div
      class="rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-5 shadow-(--shadow-subtle)"
    >
      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-2.5">
          <Shield size={16} class="text-(--color-success)" />
          <div class="text-sm">
            <p class="text-(--color-fg-default)">{t("backup.autoLabel")}</p>
            <p class="mt-0.5 text-xs text-(--color-fg-subtle)">{t("backup.autoBody")}</p>
          </div>
        </div>
        <button
          type="button"
          onclick={() => sync.setAutoBackup(!sync.autoBackupBeforeSync)}
          aria-pressed={sync.autoBackupBeforeSync}
          aria-label={t("backup.autoLabel")}
          class="relative h-5 w-9 shrink-0 rounded-full transition-colors {sync.autoBackupBeforeSync
            ? 'bg-(--color-accent-500)'
            : 'bg-(--color-bg-overlay)'}"
        >
          <span
            class="absolute top-0.5 h-4 w-4 rounded-full bg-white shadow-(--shadow-subtle) transition-all {sync.autoBackupBeforeSync
              ? 'left-[18px]'
              : 'left-0.5'}"
          ></span>
        </button>
      </div>

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

      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-2.5">
          <History size={16} class="text-(--color-warning)" />
          <div class="text-sm">
            <p class="text-(--color-fg-default)">{t("backup.restoreLabel")}</p>
            <p class="mt-0.5 text-xs text-(--color-fg-subtle)">{t("backup.restoreBody")}</p>
          </div>
        </div>
        <button
          type="button"
          onclick={handleRestore}
          disabled={sync.busy || !collection.isOpen}
          class="flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-warning)/40 bg-(--color-warning)/10 px-3 py-1.5 text-xs font-medium text-(--color-warning) hover:bg-(--color-warning)/20 disabled:opacity-50"
        >
          <History size={12} />
          {t("backup.restoreButton")}
        </button>
      </div>
    </div>
  </section>

  <section class="mt-10 space-y-3">
    <h2 class="text-xs font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
      {t("sync.title")}
    </h2>
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
  </section>

  <section class="mt-10 space-y-3">
    <h2 class="text-xs font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
      {t("io.title")}
    </h2>
    <div
      class="rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-5 shadow-(--shadow-subtle)"
    >
      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-2.5">
          <FilePlus2 size={16} class="text-(--color-accent-500)" />
          <div class="text-sm">
            <p class="text-(--color-fg-default)">{t("io.importLabel")}</p>
            <p class="mt-0.5 text-xs text-(--color-fg-subtle)">{t("io.importBody")}</p>
          </div>
        </div>
        <button
          type="button"
          onclick={handleImport}
          disabled={pkg.busy || !collection.isOpen}
          class="flex items-center gap-1.5 rounded-(--radius-md) bg-(--color-accent-500) px-3 py-1.5 text-xs font-medium text-(--color-fg-onAccent) shadow-(--shadow-subtle) transition-all hover:bg-(--color-accent-600) hover:shadow-(--shadow-card) active:scale-[0.97] disabled:cursor-not-allowed disabled:bg-(--color-bg-overlay) disabled:text-(--color-fg-subtle) disabled:shadow-none"
        >
          {#if pkg.busy}
            <Loader2 size={12} class="animate-spin" />
          {:else}
            <FilePlus2 size={12} />
          {/if}
          {t("io.importPick")}
        </button>
      </div>

      {#if pkg.lastImport}
        <div class="mt-3 grid grid-cols-3 gap-2 text-xs sm:grid-cols-5">
          {#each [
            { label: t("io.statNew"), value: pkg.lastImport.new },
            { label: t("io.statUpdated"), value: pkg.lastImport.updated },
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
        <div class="flex items-center justify-between gap-4">
          <div class="flex items-center gap-2.5">
            <Package size={16} class="text-(--color-accent-500)" />
            <div class="text-sm">
              <p class="text-(--color-fg-default)">{t("io.exportLabel")}</p>
              <p class="mt-0.5 text-xs text-(--color-fg-subtle)">{t("io.exportBody")}</p>
            </div>
          </div>
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
        </div>

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
  </section>

  <section class="mt-10 space-y-3">
    <h2 class="text-xs font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
      {t("settings.startup")}
    </h2>
    <div
      class="rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-5 shadow-(--shadow-subtle)"
    >
      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-2.5">
          <Power size={16} class="text-(--color-accent-500)" />
          <div class="text-sm">
            <p class="text-(--color-fg-default)">{t("settings.startup.autoLabel")}</p>
            <p class="mt-0.5 text-xs text-(--color-fg-subtle)">{t("settings.startup.autoBody")}</p>
          </div>
        </div>
        <button
          type="button"
          onclick={toggleAutostart}
          disabled={!autostartReady || autostartBusy}
          aria-pressed={autostartEnabled}
          aria-label={t("settings.startup.autoLabel")}
          class="relative h-5 w-9 shrink-0 rounded-full transition-colors disabled:opacity-50 {autostartEnabled
            ? 'bg-(--color-accent-500)'
            : 'bg-(--color-bg-overlay)'}"
        >
          <span
            class="absolute top-0.5 h-4 w-4 rounded-full bg-white shadow-(--shadow-subtle) transition-all {autostartEnabled
              ? 'left-[18px]'
              : 'left-0.5'}"
          ></span>
        </button>
      </div>
      {#if autostartError}
        <p class="mt-3 flex items-start gap-1.5 text-xs text-(--color-danger)">
          <AlertCircle size={12} class="mt-0.5 shrink-0" />
          <span class="break-all">{autostartError}</span>
        </p>
      {/if}
    </div>
  </section>

  <section class="mt-10 space-y-3">
    <h2 class="text-xs font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
      {t("updater.title")}
    </h2>
    <div
      class="rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-5 shadow-(--shadow-subtle)"
    >
      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-2.5">
          <DownloadCloud size={16} class="text-(--color-accent-500)" />
          <div class="text-sm">
            <p class="text-(--color-fg-default)">memorize</p>
            <p class="mt-0.5 text-xs text-(--color-fg-subtle)">
              {appVersion ? t("updater.currentVersion", { version: appVersion }) : ""}
            </p>
          </div>
        </div>
        {#if updateStatus === "available"}
          <button
            type="button"
            onclick={handleInstallUpdate}
            class="flex items-center gap-1.5 rounded-(--radius-md) bg-(--color-accent-500) px-3 py-1.5 text-xs font-medium text-(--color-fg-onAccent) shadow-(--shadow-subtle) transition-all hover:bg-(--color-accent-600) hover:shadow-(--shadow-card) active:scale-[0.97]"
          >
            <DownloadCloud size={12} />
            {t("updater.installNow")}
          </button>
        {:else if updateStatus === "downloading" || updateStatus === "installing"}
          <button
            type="button"
            disabled
            class="flex items-center gap-1.5 rounded-(--radius-md) bg-(--color-bg-overlay) px-3 py-1.5 text-xs text-(--color-fg-subtle)"
          >
            <Loader2 size={12} class="animate-spin" />
            {updateStatus === "installing"
              ? t("updater.installing")
              : t("updater.downloading", { percent: downloadProgress ?? 0 })}
          </button>
        {:else}
          <button
            type="button"
            onclick={handleCheckUpdates}
            disabled={updateStatus === "checking"}
            class="flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-border-strong) px-3 py-1.5 text-xs text-(--color-fg-default) transition-colors hover:bg-(--color-bg-overlay) active:scale-[0.98] disabled:opacity-50"
          >
            {#if updateStatus === "checking"}
              <Loader2 size={12} class="animate-spin" />
              {t("updater.checking")}
            {:else}
              <RefreshCw size={12} />
              {t("updater.check")}
            {/if}
          </button>
        {/if}
      </div>

      {#if updateStatus === "available" && updateInfo}
        <p class="mt-3 flex items-center gap-1.5 text-xs text-(--color-fg-muted)">
          <CheckCircle2 size={12} class="text-(--color-accent-500)" />
          {t("updater.available", { version: updateInfo.version })}
        </p>
        {#if updateInfo.body}
          <pre class="mt-2 max-h-40 overflow-auto rounded-(--radius-sm) bg-(--color-bg-overlay) p-2 font-mono text-[11px] whitespace-pre-wrap text-(--color-fg-muted)">{updateInfo.body}</pre>
        {/if}
      {:else if updateStatus === "up-to-date"}
        <p class="mt-3 flex items-center gap-1.5 text-xs text-(--color-fg-muted)">
          <CheckCircle2 size={12} class="text-(--color-success)" />
          {t("updater.upToDate")}
        </p>
      {/if}

      {#if updateError}
        <p class="mt-3 flex items-start gap-1.5 text-xs text-(--color-danger)">
          <AlertCircle size={12} class="mt-0.5 shrink-0" />
          <span class="break-all">{updateError}</span>
        </p>
      {/if}
    </div>
  </section>

  <section class="mt-10 space-y-3">
    <h2 class="text-xs font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
      {t("settings.appearance")}
    </h2>
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
  </section>

  <section class="mt-10 space-y-3">
    <h2 class="text-xs font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
      {t("settings.collection")}
    </h2>
    <div
      class="rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-4 shadow-(--shadow-subtle)"
    >
      {#if collection.isOpen}
        <p class="text-sm text-(--color-fg-default)">
          {t("settings.collectionOpenedWithDecks", { count: collection.decks.length })}
        </p>
        {#if collection.currentPath}
          <p class="mt-1 truncate font-mono text-[11px] text-(--color-fg-subtle)">
            {collection.currentPath}
          </p>
        {/if}
        <button
          type="button"
          onclick={() => collection.close()}
          class="mt-3 rounded-(--radius-md) border border-(--color-border-strong) px-3 py-1.5 text-sm text-(--color-fg-default) transition-colors hover:bg-(--color-bg-overlay) active:scale-[0.98]"
        >
          {t("settings.closeCollection")}
        </button>
      {:else}
        <p class="text-sm text-(--color-fg-muted)">{t("settings.collectionNotOpen")}</p>
      {/if}

      {#if collection.ankiDesktopPath && collection.ankiDesktopPath !== collection.currentPath}
        <div
          class="mt-4 rounded-(--radius-md) border border-(--color-accent-500)/40 bg-(--color-accent-500)/8 p-3 text-xs"
        >
          <p class="text-(--color-fg-default)">
            {t("settings.ankiDesktopDetected")}
          </p>
          <p class="mt-1 truncate font-mono text-[11px] text-(--color-fg-subtle)">
            {collection.ankiDesktopPath}
          </p>
          <p class="mt-2 leading-relaxed whitespace-pre-line text-(--color-fg-muted)">
            {t("settings.ankiDesktopHint")}
          </p>
          <button
            type="button"
            onclick={() => {
              if (collection.ankiDesktopPath) {
                void collection.open(collection.ankiDesktopPath);
              }
            }}
            class="mt-2 rounded-(--radius-md) bg-(--color-accent-500) px-3 py-1.5 text-xs font-medium text-(--color-fg-onAccent) hover:bg-(--color-accent-600) active:scale-[0.97]"
          >
            {t("settings.switchToThis")}
          </button>
        </div>
      {/if}
    </div>
  </section>

  <section class="mt-10 space-y-3">
    <h2 class="text-xs font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
      {t("settings.shortcuts")}
    </h2>
    <div
      class="overflow-hidden rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) shadow-(--shadow-subtle)"
    >
      {#each ratingShortcuts as s, i (s.action)}
        <div
          class="flex items-center justify-between gap-4 px-4 py-2.5 {i > 0
            ? 'border-t border-(--color-border-default)'
            : ''}"
        >
          <span class="text-sm text-(--color-fg-default)">{s.label}</span>
          <button
            type="button"
            onclick={() => startRecord(s.action)}
            class="rounded-(--radius-xs) border px-2 py-0.5 font-mono text-xs transition-colors
              {recordingFor === s.action
              ? 'border-(--color-accent-500) bg-(--color-accent-500)/10 text-(--color-accent-500) animate-pulse'
              : 'border-(--color-border-default) bg-(--color-bg-base) text-(--color-fg-muted) hover:border-(--color-border-strong) hover:text-(--color-fg-default)'}"
            title={recordingFor === s.action ? "Press a key… (Esc to cancel)" : "Click to rebind"}
          >
            {recordingFor === s.action ? "…" : shortcuts.label(s.action)}
          </button>
        </div>
      {/each}
      {#each fixedShortcuts as s, i (s.label)}
        <div
          class="flex items-center justify-between gap-4 border-t border-(--color-border-default) px-4 py-2.5"
        >
          <span class="text-sm text-(--color-fg-default)">{s.label}</span>
          <div class="flex gap-1">
            {#each s.keys as k (k)}
              <kbd
                class="rounded-(--radius-xs) border border-(--color-border-default) bg-(--color-bg-base) px-1.5 py-0.5 font-mono text-xs text-(--color-fg-muted)"
                >{k}</kbd
              >
            {/each}
          </div>
        </div>
      {/each}
      <div class="flex items-center justify-end gap-2 border-t border-(--color-border-default) px-4 py-2">
        <button
          type="button"
          onclick={() => shortcuts.reset()}
          class="text-[11px] text-(--color-fg-subtle) hover:text-(--color-fg-default)"
        >
          Reset to default (1/2/3/4 + n)
        </button>
      </div>
    </div>
  </section>
</div>
