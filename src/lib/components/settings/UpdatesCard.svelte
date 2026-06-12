<script lang="ts">
  // アップデートカード (settings 最上部)。updater 状態機械を内包する。
  // available 時はアクセントカラーで強調して気付きやすくする。
  import { onMount } from "svelte";
  import {
    CheckCircle2,
    AlertCircle,
    Loader2,
    RefreshCw,
    DownloadCloud,
  } from "lucide-svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { downloadPercent } from "$lib/updater-progress";

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
          const pct = downloadPercent(received, total);
          if (pct !== null) downloadProgress = pct;
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
    void loadAppVersion();
  });
</script>

<section
  id="updates"
  class="mt-6 scroll-mt-20 rounded-(--radius-lg) border p-5 shadow-(--shadow-subtle) {updateStatus === 'available'
    ? 'border-(--color-accent-500)/40 bg-(--color-accent-50)'
    : 'border-(--color-border-default) bg-(--color-bg-elevated)'}"
>
  <div class="flex items-center justify-between gap-4">
    <div class="flex items-center gap-2.5">
      <DownloadCloud size={18} class="text-(--color-accent-500)" />
      <div class="text-sm">
        <p class="font-medium text-(--color-fg-default)">
          {updateStatus === "available"
            ? t("updater.askTitle")
            : t("updater.title")}
        </p>
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
</section>
