<script lang="ts">
  import { Sun, Moon, RefreshCw, Loader2, AlertCircle } from "lucide-svelte";
  import { theme } from "$lib/stores/theme.svelte";
  import { sync } from "$lib/stores/sync.svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import { goto } from "$app/navigation";
  import { draggable } from "$lib/actions/draggable";
  import { onMount } from "svelte";

  onMount(() => {
    void sync.refresh();
  });

  async function handleSync() {
    if (!sync.loggedIn) {
      await goto("/settings/");
      return;
    }
    if (!collection.isOpen) {
      await goto("/");
      return;
    }
    await sync.syncNow();
    if (!sync.lastError) await collection.refreshDecks();
  }

  const syncTitle = $derived(
    !collection.isOpen
      ? "コレクションを開いてください"
      : !sync.loggedIn
        ? "AnkiWeb にログインしてください"
        : sync.busy
          ? (sync.busyReason ?? "同期中…")
          : "今すぐ同期",
  );
</script>

<header
  use:draggable
  class="flex h-11 shrink-0 items-center justify-end gap-1 border-b border-(--color-border-default) bg-(--color-bg-elevated) px-3"
>
  {#if sync.busy && sync.busyReason}
    <span class="hidden truncate text-[11px] text-(--color-fg-subtle) sm:inline">
      {sync.busyReason}
    </span>
  {:else if sync.lastError}
    <span class="hidden items-center gap-1 text-[11px] text-(--color-danger) sm:inline-flex">
      <AlertCircle size={11} />
      Sync エラー
    </span>
  {/if}

  <button
    type="button"
    onclick={handleSync}
    disabled={sync.busy}
    class="grid h-7 w-7 place-items-center rounded-md transition-colors hover:bg-(--color-bg-overlay) active:scale-[0.96] disabled:opacity-60
      {sync.lastError
      ? 'text-(--color-danger)'
      : sync.loggedIn && collection.isOpen
        ? 'text-(--color-fg-default)'
        : 'text-(--color-fg-subtle)'}"
    aria-label={syncTitle}
    title={syncTitle}
  >
    {#if sync.busy}
      <Loader2 size={15} strokeWidth={2} class="animate-spin" />
    {:else}
      <RefreshCw size={14} strokeWidth={2} />
    {/if}
  </button>

  <button
    type="button"
    onclick={() => theme.toggle()}
    class="grid h-7 w-7 place-items-center rounded-md text-(--color-fg-muted) transition-colors hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default) active:scale-[0.96]"
    aria-label="Toggle theme"
  >
    {#if theme.resolved === "dark"}
      <Sun size={15} strokeWidth={2} />
    {:else}
      <Moon size={15} strokeWidth={2} />
    {/if}
  </button>
</header>
