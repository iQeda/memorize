# Browse 一括削除 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Browse 画面でチェックボックスにより複数の単語を選択し、一括削除できるようにする。

**Architecture:** バックエンドは既存の `delete_notes` コマンド／`notes.deleteNotes()` ストアをそのまま再利用。新規追加は (1) 選択ロジックの純粋ヘルパー `src/lib/browse/selection.ts`（Vitest でテスト）と (2) `browse/+page.svelte` の UI 結線（チェックボックス列・選択バー・すべて選択ボタン・⌘A/Esc・確認ダイアログ）。テスト対象のロジックはすべて純粋ヘルパーに寄せ、Svelte 側は薄い結線に留める。

**Tech Stack:** SvelteKit + Svelte 5（runes: `$state` / `$derived` / `$effect`）, Tauri 2（`@tauri-apps/plugin-dialog` の `confirm`）, Vitest（jsdom）, Tailwind v4。

## Global Constraints

- 返信テキストは日本語、コード識別子は英語（プロジェクト CLAUDE.md）。
- 非自明な変更にはユニットテスト同梱必須。テスト対象ロジックは純粋関数へ抽出する。
- i18n キーは `src/lib/i18n/messages.ts` の `en` と `ja` の両方に追加すること（`i18n.test.ts` が「en と ja のキー集合一致」をアサートしているため、片方だけだとテストが落ちる）。
- `.svelte` 内の JS テンプレートリテラルに `<script>` / `</script>` リテラルを書かない（本変更では該当なし）。
- フロント作業完了前に `pnpm build`（SSR バンドル）を通すこと。HMR では検出されない SSR 専用エラーを拾うため。
- runes ストアの拡張子は `.svelte.ts`。ただし本変更で追加する `selection.ts` は runes を含まない純関数のみなので通常の `.ts` でよい。
- 削除対象は「単語=ノート（`note_id`）」。Browse の行は「カード」なので、選択カード id を `note_id` に変換・重複排除してから削除する。
- カラー/アイコンは既存トークンに合わせる: 削除ボタンは `--color-danger`（NoteEditor の削除ボタンと同系）、アイコンは lucide の `Trash2`。

---

## File Structure

- **Create** `src/lib/browse/selection.ts` — 選択ロジックの純粋ヘルパー（`selectedNoteIds` / `allSelected` / `someSelected` / `allCardIds` と型 `SelectableCard`）。
- **Create** `src/lib/browse/selection.test.ts` — 上記のユニットテスト。
- **Modify** `src/lib/i18n/messages.ts` — `browse.*` の新規キーを en/ja に追加。
- **Modify** `src/routes/browse/+page.svelte` — チェックボックス列・選択バー・削除フロー・キーボード操作を結線。

---

## Task 1: 選択ロジックの純粋ヘルパー

**Files:**
- Create: `src/lib/browse/selection.ts`
- Test: `src/lib/browse/selection.test.ts`

**Interfaces:**
- Consumes: なし。
- Produces:
  - `type SelectableCard = { id: number; note_id: number }`
  - `selectedNoteIds(cards: SelectableCard[], selectedIds: Set<number>): number[]` — 選択カード id を、属する note_id（出現順・重複排除）へ変換。
  - `allSelected(cards: SelectableCard[], selectedIds: Set<number>): boolean` — 表示中の全カードが選択済みか（空リストは常に false）。
  - `someSelected(cards: SelectableCard[], selectedIds: Set<number>): boolean` — 1 件以上選択済みか。
  - `allCardIds(cards: SelectableCard[]): Set<number>` — 表示中の全カード id 集合（「すべて選択」/ ⌘A の対象）。

- [ ] **Step 1: 失敗するテストを書く**

Create `src/lib/browse/selection.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  allCardIds,
  allSelected,
  selectedNoteIds,
  someSelected,
  type SelectableCard,
} from "./selection";

const cards: SelectableCard[] = [
  { id: 1, note_id: 10 },
  { id: 2, note_id: 20 },
  { id: 3, note_id: 30 },
];

describe("selectedNoteIds", () => {
  it("returns empty for an empty selection", () => {
    expect(selectedNoteIds(cards, new Set())).toEqual([]);
  });

  it("maps selected card ids to their note ids in display order", () => {
    expect(selectedNoteIds(cards, new Set([1, 3]))).toEqual([10, 30]);
  });

  it("dedupes note ids when several selected cards share a note", () => {
    const multi: SelectableCard[] = [
      { id: 1, note_id: 10 },
      { id: 2, note_id: 10 },
      { id: 3, note_id: 20 },
    ];
    expect(selectedNoteIds(multi, new Set([1, 2, 3]))).toEqual([10, 20]);
  });

  it("ignores selected ids not present in the current list", () => {
    expect(selectedNoteIds(cards, new Set([1, 999]))).toEqual([10]);
  });
});

describe("allSelected / someSelected", () => {
  it("is false for an empty selection", () => {
    expect(allSelected(cards, new Set())).toBe(false);
    expect(someSelected(cards, new Set())).toBe(false);
  });

  it("partial selection is some-but-not-all", () => {
    const sel = new Set([2]);
    expect(allSelected(cards, sel)).toBe(false);
    expect(someSelected(cards, sel)).toBe(true);
  });

  it("full selection is allSelected", () => {
    const sel = new Set([1, 2, 3]);
    expect(allSelected(cards, sel)).toBe(true);
    expect(someSelected(cards, sel)).toBe(true);
  });

  it("an empty list is never allSelected", () => {
    expect(allSelected([], new Set())).toBe(false);
  });
});

describe("allCardIds", () => {
  it("returns every visible card id", () => {
    expect(allCardIds(cards)).toEqual(new Set([1, 2, 3]));
  });

  it("returns an empty set for an empty list", () => {
    expect(allCardIds([])).toEqual(new Set());
  });
});
```

- [ ] **Step 2: テストが失敗することを確認**

Run: `pnpm exec vitest run src/lib/browse/selection.test.ts`
Expected: FAIL（`Cannot find module './selection'` — 実装未作成）

- [ ] **Step 3: 最小実装を書く**

Create `src/lib/browse/selection.ts`:

```ts
/** A Browse table row. Rows are cards; `note_id` is the word they belong to. */
export type SelectableCard = { id: number; note_id: number };

/**
 * Map a set of selected card ids to the distinct note ids they belong to, in
 * first-seen order. Deleting a note removes all its cards, so several selected
 * cards sharing a note collapse to one id. Card ids not in `cards` (stale
 * selection) are ignored because we iterate the current list.
 */
export function selectedNoteIds(
  cards: SelectableCard[],
  selectedIds: Set<number>,
): number[] {
  const seen = new Set<number>();
  const out: number[] = [];
  for (const c of cards) {
    if (selectedIds.has(c.id) && !seen.has(c.note_id)) {
      seen.add(c.note_id);
      out.push(c.note_id);
    }
  }
  return out;
}

/** True when every visible card is selected. An empty list is never "all". */
export function allSelected(
  cards: SelectableCard[],
  selectedIds: Set<number>,
): boolean {
  return cards.length > 0 && cards.every((c) => selectedIds.has(c.id));
}

/** True when at least one visible card is selected. */
export function someSelected(
  cards: SelectableCard[],
  selectedIds: Set<number>,
): boolean {
  return cards.some((c) => selectedIds.has(c.id));
}

/** Every visible card id — the target of "select all" / ⌘A. */
export function allCardIds(cards: SelectableCard[]): Set<number> {
  return new Set(cards.map((c) => c.id));
}
```

- [ ] **Step 4: テストが通ることを確認**

Run: `pnpm exec vitest run src/lib/browse/selection.test.ts`
Expected: PASS（全ケース green）

- [ ] **Step 5: コミット**

```bash
git add src/lib/browse/selection.ts src/lib/browse/selection.test.ts
git commit -m "feat(browse): 選択→note_id 変換などの純粋ヘルパーを追加

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Sdn7n4wMrrEWRbvqtXw9ML"
```

---

## Task 2: i18n キー追加

**Files:**
- Modify: `src/lib/i18n/messages.ts`（en ブロックの `"browse.empty"` 行直後、ja ブロックの `"browse.empty"` 行直後）

**Interfaces:**
- Consumes: なし。
- Produces: 新規メッセージキー（Task 3 が `t()` 経由で参照）
  - `browse.selectedCount`（`{count}` 埋め込み）, `browse.clearSelection`, `browse.deleteSelected`, `browse.selectAll`, `browse.deleteConfirmTitle`, `browse.deleteConfirmBody`（`{count}` 埋め込み）

- [ ] **Step 1: en ブロックにキーを追加**

`src/lib/i18n/messages.ts` の en 側、`"browse.empty": "(empty)",` の行の直後に挿入:

```ts
    "browse.selectedCount": "{count} selected",
    "browse.clearSelection": "Clear",
    "browse.deleteSelected": "Delete",
    "browse.selectAll": "Select all",
    "browse.deleteConfirmTitle": "Delete words",
    "browse.deleteConfirmBody": "Delete {count} words? This cannot be undone.",
```

- [ ] **Step 2: ja ブロックにキーを追加**

同ファイルの ja 側、`"browse.empty": "(空)",` の行の直後に挿入:

```ts
    "browse.selectedCount": "{count}件選択中",
    "browse.clearSelection": "解除",
    "browse.deleteSelected": "削除",
    "browse.selectAll": "すべて選択",
    "browse.deleteConfirmTitle": "単語を削除",
    "browse.deleteConfirmBody": "{count}件の単語を削除します。この操作は取り消せません。",
```

- [ ] **Step 3: パリティ・型チェックを実行**

Run: `pnpm exec vitest run src/lib/i18n/i18n.test.ts`
Expected: PASS（特に「en and ja catalogues have the exact same key sets」が green）

Run: `pnpm exec svelte-check --tsconfig ./tsconfig.json`
Expected: エラー 0（新キーが `MessageKey` 型に反映され、Task 3 の `t("browse.…")` 参照が型として有効になる）

- [ ] **Step 4: コミット**

```bash
git add src/lib/i18n/messages.ts
git commit -m "feat(browse): 一括削除/選択バー用の i18n キーを追加（en/ja）

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Sdn7n4wMrrEWRbvqtXw9ML"
```

---

## Task 3: Browse UI 結線（チェックボックス・選択バー・削除フロー）

**Files:**
- Modify: `src/routes/browse/+page.svelte`（全面的に書き換え。完成形を下記 Step 1 に提示）

**Interfaces:**
- Consumes:
  - `selection.ts` の `selectedNoteIds` / `allSelected` / `someSelected` / `allCardIds`（Task 1）
  - `browse.*` i18n キー（Task 2）
  - 既存: `notes.deleteNotes(noteIds: number[]): Promise<number>`（`$lib/stores/notes.svelte`）, `collection.refreshDecks()` / `collection.selectedDeckId` / `collection.isOpen` / `collection.decks`, `t()`, `invoke()`, `NoteEditor`
- Produces: なし（最終 UI）。

- [ ] **Step 1: `+page.svelte` を完成形に書き換える**

`src/routes/browse/+page.svelte` の内容を以下で置き換える:

```svelte
<script lang="ts">
  import { Search, Filter, Plus, Trash2 } from "lucide-svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import { notes } from "$lib/stores/notes.svelte";
  import { invoke } from "$lib/ipc";
  import NoteEditor from "$lib/components/NoteEditor.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import {
    allCardIds,
    allSelected,
    selectedNoteIds,
    someSelected,
  } from "$lib/browse/selection";

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

  // Selected card ids. Reassign (never mutate in place) so $derived re-runs.
  let selected = $state<Set<number>>(new Set());
  const allSel = $derived(allSelected(cards, selected));
  const someSel = $derived(someSelected(cards, selected));

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
    // Drop stale selection on every reload — selected ids may no longer exist.
    selected = new Set();
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

  function toggleOne(cardId: number) {
    const next = new Set(selected);
    if (next.has(cardId)) next.delete(cardId);
    else next.add(cardId);
    selected = next;
  }

  function selectAll() {
    selected = allCardIds(cards);
  }

  function clearSelection() {
    selected = new Set();
  }

  function toggleAll() {
    selected = allSel ? new Set() : allCardIds(cards);
  }

  async function deleteSelected() {
    const noteIds = selectedNoteIds(cards, selected);
    if (noteIds.length === 0) return;
    const { confirm } = await import("@tauri-apps/plugin-dialog");
    const ok = await confirm(
      t("browse.deleteConfirmBody", { count: noteIds.length }),
      {
        title: t("browse.deleteConfirmTitle"),
        kind: "warning",
        okLabel: t("note.deleteOk"),
        cancelLabel: t("note.deleteCancel"),
      },
    );
    if (!ok) return;
    const removed = await notes.deleteNotes(noteIds);
    if (removed > 0) {
      await load(filterDeckId, query); // also clears the selection
      await collection.refreshDecks();
    }
  }

  function onKey(e: KeyboardEvent) {
    // The editor owns its own keys while open.
    if (editorMode !== null) return;
    if (e.metaKey && !e.ctrlKey && e.key.toLowerCase() === "a") {
      const tag = (e.target as HTMLElement | null)?.tagName;
      // Let inputs keep ⌘A = select text.
      if (tag === "INPUT" || tag === "TEXTAREA") return;
      if (cards.length === 0) return;
      e.preventDefault();
      selectAll();
    } else if (e.key === "Escape" && selected.size > 0) {
      e.preventDefault();
      clearSelection();
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

<svelte:window onkeydown={onKey} />

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

    {#if someSel}
      <div
        class="flex h-10 items-center gap-2 border-b border-(--color-border-default) bg-(--color-bg-sunken) px-6 text-sm"
      >
        <span class="text-(--color-fg-muted)">
          {t("browse.selectedCount", { count: selected.size })}
        </span>
        <div class="flex-1"></div>
        <button
          type="button"
          onclick={selectAll}
          disabled={allSel}
          class="rounded-(--radius-md) px-2 py-1 text-xs text-(--color-fg-muted) hover:bg-(--color-bg-overlay) disabled:cursor-not-allowed disabled:opacity-50"
        >
          {t("browse.selectAll")}
        </button>
        <button
          type="button"
          onclick={clearSelection}
          class="rounded-(--radius-md) px-2 py-1 text-xs text-(--color-fg-muted) hover:bg-(--color-bg-overlay)"
        >
          {t("browse.clearSelection")}
        </button>
        <button
          type="button"
          onclick={deleteSelected}
          class="flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-danger)/40 bg-(--color-danger)/10 px-3 py-1 text-xs font-medium text-(--color-danger) hover:bg-(--color-danger)/20 active:scale-[0.97]"
        >
          <Trash2 size={12} strokeWidth={2.5} />
          {t("browse.deleteSelected")}
        </button>
      </div>
    {/if}

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
              <th class="w-10 px-6 py-2.5">
                <input
                  type="checkbox"
                  aria-label={t("browse.selectAll")}
                  checked={allSel}
                  indeterminate={someSel && !allSel}
                  onchange={toggleAll}
                  class="cursor-pointer accent-(--color-accent-500)"
                />
              </th>
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
                <td class="px-6 py-2" onclick={(e) => e.stopPropagation()}>
                  <input
                    type="checkbox"
                    aria-label={t("browse.selectAll")}
                    checked={selected.has(c.id)}
                    onchange={() => toggleOne(c.id)}
                    class="cursor-pointer accent-(--color-accent-500)"
                  />
                </td>
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
```

- [ ] **Step 2: 型チェック**

Run: `pnpm exec svelte-check --tsconfig ./tsconfig.json`
Expected: エラー 0

- [ ] **Step 3: SSR バンドルが通ることを確認（CLAUDE.md 必須）**

Run: `pnpm build`
Expected: ビルド成功（SSR 専用の Svelte エラーが出ないこと）

- [ ] **Step 4: ユニットテスト全件**

Run: `pnpm test:run`
Expected: PASS（Task 1 の `selection.test.ts` と Task 2 の i18n パリティを含め全 green）

- [ ] **Step 5: 手動確認（Tauri シェル）**

Run: `pnpm tauri dev`
確認項目:
1. Browse を開き、行のチェックボックスを ON → 上部に選択バーが出る。`N件選択中` の件数が一致。
2. チェックボックスのクリックで編集モーダルが開かない（行クリックは従来通り編集を開く）。
3. ヘッダーのチェックボックス: 一部選択で indeterminate（横棒）表示、全選択で ON、再クリックで全解除。
4. 選択バーの「すべて選択」: 全行選択、全選択済みのとき disabled。
5. ⌘A: 一覧に未フォーカス時は全選択。検索入力にフォーカス中はテキスト全選択のまま（横取りしない）。Esc で選択クリア。
6. 「🗑 削除」→ 確認ダイアログ（警告）で OK → 選択した単語が消え、サイドバーのデッキ件数が減る。Cancel で何も起きない。
7. デッキ/検索を切り替えると選択がクリアされる。

- [ ] **Step 6: コミット**

```bash
git add src/routes/browse/+page.svelte
git commit -m "feat(browse): チェックボックス選択 + 単語の一括削除を実装

選択バー（すべて選択/解除/削除）・先頭チェックボックス列・⌘A/Esc を追加。
削除は選択カードを note_id へ重複排除して delete_notes に渡し、確認ダイアログ
（@tauri-apps/plugin-dialog の confirm）を経て collection.refreshDecks で件数更新。

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Sdn7n4wMrrEWRbvqtXw9ML"
```

---

## 完了条件 / Pre-push チェック（CLAUDE.md）

push する場合は以下 3 点をクリアすること（コミット自体は各タスク末尾で実施済み）:

1. **対応テスト同梱**: 選択ロジックは `selection.test.ts`、i18n はパリティテストでカバー。UI 結線は純粋ヘルパーに委譲済みで behavior テストはヘルパー側にある旨をコミットメッセージに記す。
2. **テスト全件 green**:
   ```sh
   pnpm test:run
   PROTOC=/opt/homebrew/bin/protoc cargo test --manifest-path src-tauri/Cargo.toml
   ```
   （Rust 側は本変更で未修整だが、push 前チェックとして全件実行する。）
3. **lint / type-check 0 件**:
   ```sh
   pnpm exec svelte-check --tsconfig ./tsconfig.json
   cargo check --manifest-path src-tauri/Cargo.toml
   ```
