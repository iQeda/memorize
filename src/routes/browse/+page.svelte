<script lang="ts">
  import { Search, Filter, Plus } from "lucide-svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import { invoke } from "$lib/ipc";
  import NoteEditor from "$lib/components/NoteEditor.svelte";
  import { t } from "$lib/i18n";

  type CardSummary = {
    id: number;
    note_id: number;
    deck_id: number;
    template_idx: number;
    text: string;
  };

  let query = $state("");
  let cards = $state<CardSummary[]>([]);
  let loading = $state(false);
  // null = "すべて" (no deck filter)
  let filterDeckId = $state<number | null>(collection.selectedDeckId);

  let editorMode = $state<"add" | "edit" | null>(null);
  let editingNoteId = $state<number | null>(null);

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    const dId = filterDeckId;
    const q = query;
    if (!collection.isOpen) {
      cards = [];
      return;
    }
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      void load(dId, q);
    }, 200);
  });

  async function load(deckId: number | null, q: string) {
    loading = true;
    try {
      cards = await invoke<CardSummary[]>("list_cards", {
        deckId,
        query: q.trim() || null,
        limit: 500,
      });
    } catch (e) {
      console.error(e);
      cards = [];
    } finally {
      loading = false;
    }
  }

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
    await load(filterDeckId, query);
    await collection.refreshDecks();
  }

  function stripHtml(s: string): string {
    return s
      .replace(/<br\s*\/?>(\r?\n)?/gi, " / ")
      .replace(/<[^>]+>/g, "")
      .replace(/&nbsp;/g, " ")
      .replace(/&amp;/g, "&")
      .replace(/&lt;/g, "<")
      .replace(/&gt;/g, ">")
      .trim();
  }
</script>

<div class="grid h-full grid-cols-[280px_1fr]">
  <aside
    class="flex h-full flex-col gap-4 border-r border-(--color-border-default) bg-(--color-bg-sunken) px-4 py-5"
  >
    <h2 class="text-xs font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
      {t("browse.filter")}
    </h2>

    <label class="block">
      <span class="mb-1 block text-[11px] tracking-wider text-(--color-fg-subtle) uppercase">
        {t("browse.deck")}
      </span>
      <select
        value={filterDeckId}
        onchange={(e) => {
          const v = (e.currentTarget as HTMLSelectElement).value;
          filterDeckId = v === "" ? null : Number(v);
        }}
        class="w-full rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-elevated) px-2 py-1.5 text-sm shadow-(--shadow-subtle) outline-none focus:border-(--color-accent-500)"
      >
        <option value="">{t("browse.allDecks")}</option>
        {#each collection.decks as d (d.id)}
          <option value={d.id}>{d.name}</option>
        {/each}
      </select>
    </label>

    <div class="relative">
      <Search
        size={14}
        strokeWidth={2}
        class="pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2 text-(--color-fg-subtle)"
      />
      <input
        type="search"
        bind:value={query}
        placeholder={t("browse.searchPlaceholder")}
        class="w-full rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-elevated) py-1.5 pr-3 pl-7 text-sm shadow-(--shadow-subtle) outline-none focus:border-(--color-accent-500)"
      />
    </div>
    <div class="flex items-center gap-1.5 text-xs text-(--color-fg-subtle)">
      <Filter size={12} strokeWidth={2} />
      <span>{t("browse.searchHint")}</span>
    </div>
  </aside>

  <section class="flex h-full min-w-0 flex-col">
    <header
      class="flex h-12 items-center justify-between border-b border-(--color-border-default) px-6"
    >
      <p class="text-sm text-(--color-fg-muted)">
        {#if loading}{t("browse.loading")}{:else}{t("browse.cardsCount", { count: cards.length })}{/if}
      </p>
      <button
        type="button"
        onclick={openAdd}
        disabled={!collection.isOpen}
        class="flex items-center gap-1.5 rounded-(--radius-md) bg-(--color-accent-500) px-3 py-1.5 text-xs font-medium text-(--color-fg-onAccent) shadow-(--shadow-subtle) hover:bg-(--color-accent-600) active:scale-[0.97] disabled:cursor-not-allowed disabled:bg-(--color-bg-overlay) disabled:text-(--color-fg-subtle) disabled:shadow-none"
      >
        <Plus size={12} strokeWidth={2.5} />
        {t("browse.addWord")}
      </button>
    </header>
    <div class="flex-1 overflow-y-auto">
      {#if cards.length === 0 && !loading}
        <div class="grid h-full place-items-center text-(--color-fg-subtle)">
          <p class="text-sm">{query.trim() ? t("browse.noHits") : t("browse.noCards")}</p>
        </div>
      {:else}
        <table class="w-full text-sm">
          <thead
            class="sticky top-0 bg-(--color-bg-base) text-left text-[11px] font-medium tracking-wider text-(--color-fg-subtle) uppercase"
          >
            <tr>
              <th class="px-6 py-2.5">{t("browse.colWord")}</th>
              <th class="px-6 py-2.5">{t("browse.colNote")}</th>
              <th class="px-6 py-2.5">{t("browse.colTemplate")}</th>
            </tr>
          </thead>
          <tbody>
            {#each cards as c (c.id)}
              <tr
                onclick={() => openEdit(c.note_id)}
                class="cursor-pointer border-t border-(--color-border-default) hover:bg-(--color-bg-overlay)"
              >
                <td class="max-w-[420px] truncate px-6 py-2 text-(--color-fg-default)">
                  {stripHtml(c.text) || t("browse.empty")}
                </td>
                <td class="px-6 py-2 font-mono text-xs tabular-nums text-(--color-fg-subtle)">{c.note_id}</td>
                <td class="px-6 py-2 text-xs text-(--color-fg-muted)">#{c.template_idx}</td>
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
    initialDeckId={filterDeckId ?? collection.selectedDeckId ?? undefined}
    onClose={closeEditor}
    {onSaved}
  />
{/if}
