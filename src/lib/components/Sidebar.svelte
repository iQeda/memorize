<script lang="ts">
  import { page } from "$app/stores";
  import {
    Search,
    Settings as SettingsIcon,
    Brain,
    Plus,
    Check,
    X,
    Pencil,
    Trash2,
  } from "lucide-svelte";
  import { collection, type DeckSummary } from "$lib/stores/collection.svelte";
  import { draggable } from "$lib/actions/draggable";
  import ContextMenu from "$lib/components/ContextMenu.svelte";
  import { goto } from "$app/navigation";
  import { tick } from "svelte";

  async function selectDeck(deck: DeckSummary) {
    collection.selectedDeckId = deck.id;
    if ($page.url.pathname !== "/") {
      await goto("/");
    }
  }

  let creating = $state(false);
  let newName = $state("");
  let newInputEl = $state<HTMLInputElement | null>(null);

  async function startCreate() {
    creating = true;
    newName = "";
    await tick();
    newInputEl?.focus();
  }

  async function submitCreate() {
    const name = newName.trim();
    if (!name) {
      creating = false;
      return;
    }
    const id = await collection.createDeck(name);
    creating = false;
    newName = "";
    if (id !== null) collection.selectedDeckId = id;
  }

  function cancelCreate() {
    creating = false;
    newName = "";
  }

  function onInputKey(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      void submitCreate();
    } else if (e.key === "Escape") {
      e.preventDefault();
      cancelCreate();
    }
  }

  // ---- Context menu / rename / delete ----
  let menu = $state<{ x: number; y: number; deck: DeckSummary } | null>(null);
  let renamingId = $state<number | null>(null);
  let renameValue = $state("");
  let renameInputEl = $state<HTMLInputElement | null>(null);

  function openMenu(e: MouseEvent, deck: DeckSummary) {
    e.preventDefault();
    menu = { x: e.clientX, y: e.clientY, deck };
  }

  async function startRename(deck: DeckSummary) {
    menu = null;
    renamingId = deck.id;
    renameValue = deck.name;
    await tick();
    renameInputEl?.focus();
    renameInputEl?.select();
  }

  async function submitRename() {
    if (renamingId === null) return;
    const id = renamingId;
    const name = renameValue.trim();
    renamingId = null;
    if (!name) return;
    await collection.renameDeck(id, name);
  }

  function cancelRename() {
    renamingId = null;
    renameValue = "";
  }

  function onRenameKey(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      void submitRename();
    } else if (e.key === "Escape") {
      e.preventDefault();
      cancelRename();
    }
  }

  async function deleteDeck(deck: DeckSummary) {
    menu = null;
    const { confirm } = await import("@tauri-apps/plugin-dialog");
    const ok = await confirm(
      `デッキ「${deck.name}」とその子デッキを削除します。\n含まれるすべてのカードと単語データも削除されます。\n\nこの操作は取り消せません。続行しますか？`,
      {
        title: "デッキを削除",
        kind: "warning",
        okLabel: "削除",
        cancelLabel: "キャンセル",
      },
    );
    if (!ok) return;
    await collection.deleteDeck(deck.id);
  }

  const navItems = [
    { href: "/browse/", label: "Browse", icon: Search },
    { href: "/settings/", label: "Settings", icon: SettingsIcon },
  ];

  function isActive(href: string): boolean {
    return $page.url.pathname.startsWith(href);
  }

  function deckShortName(name: string): string {
    return name.split("::").at(-1) ?? name;
  }

  function deckBadge(d: DeckSummary): { count: number; tone: "accent" | "warning" | "success" } | null {
    if (d.new_count > 0) return { count: d.new_count, tone: "accent" };
    if (d.learn_count > 0) return { count: d.learn_count, tone: "warning" };
    if (d.review_count > 0) return { count: d.review_count, tone: "success" };
    return null;
  }

  const badgeTone = {
    accent:
      "bg-(--color-accent-500)/12 text-(--color-accent-500) ring-(--color-accent-500)/20",
    warning:
      "bg-(--color-warning)/12 text-(--color-warning) ring-(--color-warning)/20",
    success:
      "bg-(--color-success)/12 text-(--color-success) ring-(--color-success)/20",
  } as const;
</script>

<aside
  class="flex h-full w-60 shrink-0 flex-col border-r border-(--color-border-default) bg-(--color-bg-sunken)"
>
  <div
    use:draggable
    class="flex h-11 items-center pr-4 pl-[78px]"
  >
    <a
      href="/"
      data-sveltekit-preload-data="hover"
      class="no-drag flex items-center gap-2 rounded-md px-1 -mx-1 py-0.5 transition-colors hover:bg-(--color-bg-overlay)"
      aria-label="Home"
    >
      <div
        class="grid h-6 w-6 place-items-center rounded-md bg-(--color-accent-500) text-(--color-fg-onAccent) shadow-(--shadow-subtle)"
      >
        <Brain size={14} strokeWidth={2.5} />
      </div>
      <span class="font-display text-sm font-medium tracking-tight">
        memorize
      </span>
    </a>
  </div>

  <nav class="flex flex-col gap-0.5 px-2 pt-1">
    {#each navItems as item (item.href)}
      {@const active = isActive(item.href)}
      <a
        href={item.href}
        data-sveltekit-preload-data="hover"
        class="flex items-center gap-2.5 rounded-md px-2.5 py-1.5 text-sm transition-all duration-150
          {active
          ? 'bg-(--color-bg-elevated) text-(--color-fg-default) shadow-(--shadow-subtle)'
          : 'text-(--color-fg-muted) hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)'}"
      >
        <item.icon size={16} strokeWidth={2} />
        {item.label}
      </a>
    {/each}
  </nav>

  {#if collection.isOpen}
    <div class="mt-5 flex-1 overflow-y-auto px-2 pb-3">
      <div class="mb-1 flex items-center justify-between gap-1 px-2.5">
        <span
          class="text-[10px] font-semibold tracking-[0.14em] text-(--color-fg-subtle) uppercase"
        >
          デッキ
        </span>
        <div class="flex items-center gap-1">
          <span class="number-tabular text-[10px] text-(--color-fg-subtle)">
            {collection.decks.length}
          </span>
          <button
            type="button"
            onclick={startCreate}
            disabled={creating}
            aria-label="新規デッキ"
            class="grid h-4 w-4 place-items-center rounded text-(--color-fg-subtle) transition-colors hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default) disabled:opacity-40"
          >
            <Plus size={12} strokeWidth={2.5} />
          </button>
        </div>
      </div>

      {#if creating}
        <div class="mb-1 flex items-center gap-1 px-2.5">
          <input
            bind:this={newInputEl}
            bind:value={newName}
            onkeydown={onInputKey}
            placeholder="デッキ名 (a::b でネスト)"
            class="min-w-0 flex-1 rounded-(--radius-sm) border border-(--color-border-strong) bg-(--color-bg-elevated) px-2 py-0.5 text-xs outline-none focus:border-(--color-accent-500)"
          />
          <button
            type="button"
            onclick={submitCreate}
            aria-label="作成"
            class="grid h-5 w-5 place-items-center rounded text-(--color-success) transition-colors hover:bg-(--color-bg-overlay)"
          >
            <Check size={12} strokeWidth={2.5} />
          </button>
          <button
            type="button"
            onclick={cancelCreate}
            aria-label="キャンセル"
            class="grid h-5 w-5 place-items-center rounded text-(--color-fg-subtle) transition-colors hover:bg-(--color-bg-overlay)"
          >
            <X size={12} strokeWidth={2.5} />
          </button>
        </div>
      {/if}
      {#if collection.decks.length === 0 && !creating}
        <p class="px-2.5 py-2 text-[11px] text-(--color-fg-subtle)">
          デッキがまだありません。<br />+ から作成してください。
        </p>
      {/if}
      {#each collection.decks as deck (deck.id)}
        {@const active = collection.selectedDeckId === deck.id}
        {@const badge = deckBadge(deck)}
        {#if renamingId === deck.id}
          <div
            class="flex items-center gap-1 py-0.5 pr-2"
            style="padding-left: {0.625 + deck.level * 0.75}rem;"
          >
            <input
              bind:this={renameInputEl}
              bind:value={renameValue}
              onkeydown={onRenameKey}
              onblur={submitRename}
              class="min-w-0 flex-1 rounded-(--radius-sm) border border-(--color-accent-500) bg-(--color-bg-elevated) px-1.5 py-0.5 text-sm outline-none"
            />
          </div>
        {:else}
          <button
            type="button"
            onclick={() => selectDeck(deck)}
            oncontextmenu={(e) => openMenu(e, deck)}
            class="group flex w-full items-center justify-between gap-2 rounded-md py-1 pr-2 text-left text-sm transition-colors
              {active
              ? 'bg-(--color-bg-elevated) text-(--color-fg-default) shadow-(--shadow-subtle)'
              : 'text-(--color-fg-muted) hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)'}"
            style="padding-left: {0.625 + deck.level * 0.75}rem;"
          >
            <span class="flex min-w-0 items-center gap-2">
              <span
                class="h-1.5 w-1.5 shrink-0 rounded-full transition-colors
                  {badge ? (badge.tone === 'accent' ? 'bg-(--color-accent-500)' : badge.tone === 'warning' ? 'bg-(--color-warning)' : 'bg-(--color-success)') : 'bg-(--color-border-strong) group-hover:bg-(--color-fg-subtle)'}"
              ></span>
              <span class="truncate">{deckShortName(deck.name)}</span>
            </span>
            {#if badge}
              <span
                class="number-tabular shrink-0 rounded-full px-1.5 py-0.5 text-[10px] font-semibold ring-1 ring-inset {badgeTone[badge.tone]}"
              >
                {badge.count}
              </span>
            {/if}
          </button>
        {/if}
      {/each}
    </div>
  {:else}
    <div class="flex-1"></div>
  {/if}
</aside>

{#if menu}
  {@const m = menu}
  <ContextMenu x={m.x} y={m.y} onClose={() => (menu = null)}>
    <button
      type="button"
      onclick={() => startRename(m.deck)}
      class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm text-(--color-fg-default) hover:bg-(--color-bg-overlay)"
    >
      <Pencil size={12} strokeWidth={2} />
      名前を変更
    </button>
    <button
      type="button"
      onclick={() => deleteDeck(m.deck)}
      class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm text-(--color-danger) hover:bg-(--color-danger)/10"
    >
      <Trash2 size={12} strokeWidth={2} />
      削除…
    </button>
  </ContextMenu>
{/if}
