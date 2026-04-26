<script lang="ts">
  import { Search, Filter, Plus } from "lucide-svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import { invoke } from "$lib/ipc";
  import NoteEditor from "$lib/components/NoteEditor.svelte";

  type CardSummary = {
    id: number;
    note_id: number;
    deck_id: number;
    template_idx: number;
  };

  let query = $state("");
  let cards = $state<CardSummary[]>([]);
  let loading = $state(false);
  let selectedDeckId = $derived(collection.selectedDeckId);

  let editorMode = $state<"add" | "edit" | null>(null);
  let editingNoteId = $state<number | null>(null);

  $effect(() => {
    if (selectedDeckId !== null && collection.isOpen) {
      void load(selectedDeckId);
    } else {
      cards = [];
    }
  });

  async function load(deckId: number) {
    loading = true;
    try {
      cards = await invoke<CardSummary[]>("list_cards", {
        deckId,
        limit: 200,
      });
    } catch (e) {
      console.error(e);
      cards = [];
    } finally {
      loading = false;
    }
  }

  const filtered = $derived(
    query.trim()
      ? cards.filter(
          (c) =>
            String(c.id).includes(query.trim()) ||
            String(c.note_id).includes(query.trim()),
        )
      : cards,
  );

  function openAdd() {
    editingNoteId = null;
    editorMode = "add";
  }

  function openEdit(noteId: number) {
    editingNoteId = noteId;
    editorMode = "edit";
  }

  function closeEditor() {
    editorMode = null;
    editingNoteId = null;
  }

  async function onSaved() {
    if (selectedDeckId !== null) await load(selectedDeckId);
    await collection.refreshDecks();
  }
</script>

<div class="grid h-full grid-cols-[280px_1fr]">
  <aside
    class="flex h-full flex-col gap-4 border-r border-(--color-border-default) bg-(--color-bg-sunken) px-4 py-5"
  >
    <h2 class="text-xs font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
      Filter
    </h2>
    <div class="relative">
      <Search
        size={14}
        strokeWidth={2}
        class="pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2 text-(--color-fg-subtle)"
      />
      <input
        type="search"
        bind:value={query}
        placeholder="card id / note id…"
        class="w-full rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-elevated) py-1.5 pr-3 pl-7 text-sm shadow-(--shadow-subtle) outline-none focus:border-(--color-accent-500)"
      />
    </div>
    <div class="flex items-center gap-1.5 text-xs text-(--color-fg-subtle)">
      <Filter size={12} strokeWidth={2} />
      <span>id 部分一致のみ</span>
    </div>
  </aside>

  <section class="flex h-full min-w-0 flex-col">
    <header
      class="flex h-12 items-center justify-between border-b border-(--color-border-default) px-6"
    >
      <p class="text-sm text-(--color-fg-muted)">
        {#if loading}読み込み中…{:else}{filtered.length} cards{/if}
      </p>
      <button
        type="button"
        onclick={openAdd}
        disabled={!collection.isOpen || selectedDeckId === null}
        class="flex items-center gap-1.5 rounded-(--radius-md) bg-(--color-accent-500) px-3 py-1.5 text-xs font-medium text-(--color-fg-onAccent) shadow-(--shadow-subtle) hover:bg-(--color-accent-600) active:scale-[0.97] disabled:cursor-not-allowed disabled:bg-(--color-bg-overlay) disabled:text-(--color-fg-subtle) disabled:shadow-none"
      >
        <Plus size={12} strokeWidth={2.5} />
        ノートを追加
      </button>
    </header>
    <div class="flex-1 overflow-y-auto">
      {#if filtered.length === 0 && !loading}
        <div class="grid h-full place-items-center text-(--color-fg-subtle)">
          <p class="text-sm">カードがありません</p>
        </div>
      {:else}
        <table class="w-full text-sm">
          <thead
            class="sticky top-0 bg-(--color-bg-base) text-left text-[11px] font-medium tracking-wider text-(--color-fg-subtle) uppercase"
          >
            <tr>
              <th class="px-6 py-2.5">Card ID</th>
              <th class="px-6 py-2.5">Note ID</th>
              <th class="px-6 py-2.5">Template</th>
            </tr>
          </thead>
          <tbody>
            {#each filtered as c (c.id)}
              <tr
                onclick={() => openEdit(c.note_id)}
                class="cursor-pointer border-t border-(--color-border-default) hover:bg-(--color-bg-overlay)"
              >
                <td class="px-6 py-2 font-mono tabular-nums">{c.id}</td>
                <td class="px-6 py-2 font-mono tabular-nums text-(--color-fg-muted)">{c.note_id}</td>
                <td class="px-6 py-2 text-(--color-fg-muted)">#{c.template_idx}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  </section>
</div>

{#if editorMode}
  <NoteEditor
    mode={editorMode}
    noteId={editingNoteId ?? undefined}
    initialDeckId={selectedDeckId ?? undefined}
    onClose={closeEditor}
    {onSaved}
  />
{/if}
