<script lang="ts">
  import { theme, type Theme } from "$lib/stores/theme.svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import { sync } from "$lib/stores/sync.svelte";
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
  } from "lucide-svelte";

  const themeOptions: { value: Theme; label: string }[] = [
    { value: "light", label: "Light" },
    { value: "dark", label: "Dark" },
    { value: "system", label: "System" },
  ];

  const shortcuts = [
    { keys: ["1"], label: "Again" },
    { keys: ["2"], label: "Hard" },
    { keys: ["3"], label: "Good" },
    { keys: ["4"], label: "Easy" },
    { keys: ["Space"], label: "解答を表示 / Good" },
    { keys: ["⌘", ","], label: "設定を開く" },
  ];

  let username = $state("");
  let password = $state("");
  let endpoint = $state("");

  onMount(() => {
    void sync.refresh();
  });

  async function handleLogin(e: SubmitEvent) {
    e.preventDefault();
    try {
      await sync.login(username, password, endpoint || undefined);
      password = "";
    } catch {}
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
</script>

<div class="mx-auto max-w-2xl px-8 py-10">
  <h1 class="font-display text-3xl font-medium tracking-tight">Settings</h1>

  <section class="mt-10 space-y-3">
    <h2 class="text-xs font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
      Backup
    </h2>
    <div
      class="rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-5 shadow-(--shadow-subtle)"
    >
      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-2.5">
          <Shield size={16} class="text-(--color-success)" />
          <div class="text-sm">
            <p class="text-(--color-fg-default)">同期前に自動バックアップ</p>
            <p class="mt-0.5 text-xs text-(--color-fg-subtle)">
              app data dir/backups/ に <code>.colpkg</code> を作成。失敗したら同期は中止
            </p>
          </div>
        </div>
        <button
          type="button"
          onclick={() => sync.setAutoBackup(!sync.autoBackupBeforeSync)}
          aria-pressed={sync.autoBackupBeforeSync}
          aria-label="同期前に自動バックアップ"
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
          今すぐバックアップ
        </button>
        <button
          type="button"
          onclick={() => handleManualBackup(true)}
          disabled={sync.busy || !collection.isOpen}
          class="flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-border-strong) px-3 py-1.5 text-xs text-(--color-fg-default) transition-colors hover:bg-(--color-bg-overlay) active:scale-[0.98] disabled:opacity-50"
        >
          <Save size={12} />
          メディアも含めてバックアップ
        </button>
      </div>

      {#if !collection.isOpen}
        <p class="mt-3 text-xs text-(--color-fg-subtle)">
          バックアップにはコレクションを開く必要があります
        </p>
      {/if}
      {#if sync.lastBackupPath}
        <p class="mt-3 truncate font-mono text-[11px] text-(--color-fg-subtle)">
          最終: {sync.lastBackupPath}
        </p>
      {/if}
    </div>
  </section>

  <section class="mt-10 space-y-3">
    <h2 class="text-xs font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
      AnkiWeb Sync
    </h2>
    <div
      class="rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-5 shadow-(--shadow-subtle)"
    >
      {#if sync.loggedIn}
        <div class="flex items-center justify-between gap-4">
          <div class="flex items-center gap-2.5">
            <CheckCircle2 size={16} class="text-(--color-success)" />
            <span class="text-sm">
              <span class="text-(--color-fg-default)">{sync.username}</span>
              <span class="text-(--color-fg-subtle)"> としてログイン中</span>
            </span>
          </div>
          <button
            type="button"
            onclick={() => sync.logout()}
            disabled={sync.busy}
            class="flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-border-strong) px-3 py-1.5 text-xs text-(--color-fg-default) transition-colors hover:bg-(--color-bg-overlay) active:scale-[0.98] disabled:opacity-50"
          >
            <LogOut size={12} />
            ログアウト
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
            {sync.busy && sync.busyReason ? sync.busyReason : "今すぐ同期"}
          </button>

          {#if sync.fullSyncRequired}
            {#if sync.fullSyncRequired.upload_ok}
              <button
                type="button"
                onclick={() => sync.fullUpload()}
                disabled={sync.busy}
                class="flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-warning)/40 bg-(--color-warning)/10 px-3 py-2 text-xs font-medium text-(--color-warning) hover:bg-(--color-warning)/20 disabled:opacity-50"
              >
                <Upload size={12} /> ローカル → サーバー (上書き)
              </button>
            {/if}
            {#if sync.fullSyncRequired.download_ok}
              <button
                type="button"
                onclick={() => sync.fullDownload()}
                disabled={sync.busy}
                class="flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-warning)/40 bg-(--color-warning)/10 px-3 py-2 text-xs font-medium text-(--color-warning) hover:bg-(--color-warning)/20 disabled:opacity-50"
              >
                <Download size={12} /> サーバー → ローカル (上書き)
              </button>
            {/if}
          {/if}
        </div>

        {#if !collection.isOpen}
          <p class="mt-3 text-xs text-(--color-fg-subtle)">
            同期にはコレクションを開く必要があります
          </p>
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
          <p class="text-xs text-(--color-fg-muted)">
            AnkiWeb の認証情報を入力してログインします。host-key は OS キーチェーン (macOS Keychain) に保存されます。
          </p>
          <label class="block">
            <span class="mb-1 block text-xs text-(--color-fg-muted)"
              >ユーザー名 / メールアドレス</span
            >
            <input
              type="text"
              autocomplete="username"
              required
              bind:value={username}
              class="w-full rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-base) px-3 py-1.5 text-sm shadow-(--shadow-subtle) outline-none focus:border-(--color-accent-500)"
            />
          </label>
          <label class="block">
            <span class="mb-1 block text-xs text-(--color-fg-muted)">パスワード</span>
            <input
              type="password"
              autocomplete="current-password"
              required
              bind:value={password}
              class="w-full rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-base) px-3 py-1.5 text-sm shadow-(--shadow-subtle) outline-none focus:border-(--color-accent-500)"
            />
          </label>
          <details class="text-xs text-(--color-fg-subtle)">
            <summary class="cursor-pointer">カスタムサーバー (任意)</summary>
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
            ログイン
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
      Appearance
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
      Collection
    </h2>
    <div
      class="rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-4 shadow-(--shadow-subtle)"
    >
      {#if collection.isOpen}
        <p class="text-sm text-(--color-fg-default)">
          コレクションを {collection.decks.length} 個のデッキで開いています
        </p>
        <button
          type="button"
          onclick={() => collection.close()}
          class="mt-3 rounded-(--radius-md) border border-(--color-border-strong) px-3 py-1.5 text-sm text-(--color-fg-default) transition-colors hover:bg-(--color-bg-overlay) active:scale-[0.98]"
        >
          コレクションを閉じる
        </button>
      {:else}
        <p class="text-sm text-(--color-fg-muted)">
          コレクションが開いていません
        </p>
      {/if}
    </div>
  </section>

  <section class="mt-10 space-y-3">
    <h2 class="text-xs font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
      Keyboard shortcuts
    </h2>
    <div
      class="overflow-hidden rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) shadow-(--shadow-subtle)"
    >
      {#each shortcuts as s, i (s.label)}
        <div
          class="flex items-center justify-between gap-4 px-4 py-2.5 {i > 0
            ? 'border-t border-(--color-border-default)'
            : ''}"
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
    </div>
  </section>
</div>
