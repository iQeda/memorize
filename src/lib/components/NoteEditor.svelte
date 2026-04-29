<script lang="ts">
  import { collection } from "$lib/stores/collection.svelte";
  import { notes, type NotetypeSummary } from "$lib/stores/notes.svelte";
  import { X, Save, Trash2, Loader2, AlertCircle } from "lucide-svelte";
  import { onMount, tick } from "svelte";
  import { fade, scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { t } from "$lib/i18n/index.svelte";

  type Props = {
    mode: "add" | "edit";
    noteId?: number; // for edit
    initialDeckId?: number; // for add
    onClose: () => void;
    onSaved?: () => void;
  };
  let { mode, noteId, initialDeckId, onClose, onSaved }: Props = $props();

  let notetype = $state<NotetypeSummary | null>(null);
  let fields = $state<string[]>([]);
  let tagsText = $state("");
  let deckId = $state<number | null>(null);
  let loading = $state(true);
  // 各フィールドの contenteditable 要素を直接捕捉する。dialogRoot.querySelector
  // 経由だと描画タイミング次第で初回 focus が空振りすることがあるため。
  let fieldEls = $state<(HTMLElement | undefined)[]>([]);

  function focusFirstField() {
    const first = fieldEls[0];
    if (!first) return;
    first.focus();
    // 既存テキストがあるとき (Edit) はキャレットを末尾へ。空ノードでも無害。
    const sel = window.getSelection();
    if (sel) {
      const range = document.createRange();
      range.selectNodeContents(first);
      range.collapse(false);
      sel.removeAllRanges();
      sel.addRange(range);
    }
  }

  onMount(async () => {
    deckId = initialDeckId ?? null;
    if (notes.notetypes.length === 0) await notes.refreshNotetypes();
    if (mode === "edit" && noteId !== undefined) {
      const detail = await notes.getNote(noteId);
      if (detail) {
        notetype = {
          id: detail.notetype_id,
          name: detail.notetype_name,
          field_names: detail.field_names,
        };
        fields = [...detail.fields];
        tagsText = detail.tags.join(" ");
      }
    } else {
      notetype = notes.notetypes[0] ?? null;
      fields = notetype ? notetype.field_names.map(() => "") : [];
      if (deckId === null && collection.decks.length > 0) {
        deckId = collection.decks[0].id;
      }
    }
    loading = false;

    // 描画反映後に Front (最初の field) へフォーカス。Edit / Add 共通で
    // 開いた直後すぐ入力できるようにする。
    await tick();
    focusFirstField();
  });

  function selectNotetype(nt: NotetypeSummary) {
    notetype = nt;
    fields = nt.field_names.map(() => "");
  }

  async function save() {
    if (!notetype) return;
    const tags = tagsText
      .split(/\s+/)
      .map((t) => t.trim())
      .filter(Boolean);
    if (mode === "add") {
      if (deckId === null) return;
      const id = await notes.addNote({
        deckId,
        notetypeId: notetype.id,
        fields,
        tags,
      });
      if (id !== null) {
        onSaved?.();
        // Add は連続登録できるよう閉じず、フィールドだけ初期化して
        // 最初のフィールドにフォーカスを戻す。デッキ / notetype / タグは
        // 前回値を保持 (Anki Desktop の Add Cards と同じ慣習)。
        // 閉じたいときは X / Esc / Cancel が使える。
        fields = notetype.field_names.map(() => "");
        await tick();
        focusFirstField();
      }
    } else if (noteId !== undefined) {
      const ok = await notes.updateNote({ noteId, fields, tags });
      if (ok) {
        onSaved?.();
        onClose();
      }
    }
  }

  async function deleteNote() {
    if (mode !== "edit" || noteId === undefined) return;
    const { confirm } = await import("@tauri-apps/plugin-dialog");
    const ok = await confirm(t("note.deleteConfirmBody"), {
      title: t("note.deleteConfirmTitle"),
      kind: "warning",
      okLabel: t("note.deleteOk"),
      cancelLabel: t("note.deleteCancel"),
    });
    if (!ok) return;
    const removed = await notes.deleteNotes([noteId]);
    if (removed > 0) {
      onSaved?.();
      onClose();
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    } else if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      void save();
    }
  }
</script>

<svelte:window onkeydown={onKey} />

<div
  in:fade={{ duration: 120, easing: cubicOut }}
  out:fade={{ duration: 80, easing: cubicOut }}
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose();
  }}
  onkeydown={() => {}}
  role="dialog"
  aria-modal="true"
  tabindex="-1"
>
  <div
    in:scale={{ duration: 180, start: 0.96, easing: cubicOut }}
    class="flex max-h-[80vh] w-full max-w-xl flex-col overflow-hidden rounded-(--radius-xl) border border-(--color-border-default) bg-(--color-bg-elevated) shadow-(--shadow-card)"
  >
    <header
      class="flex items-center justify-between border-b border-(--color-border-default) px-5 py-3"
    >
      <h2 class="text-sm font-semibold tracking-wide">
        {mode === "add" ? t("note.addTitle") : t("note.editTitle")}
      </h2>
      <button
        type="button"
        onclick={onClose}
        aria-label="Close"
        class="grid h-6 w-6 place-items-center rounded text-(--color-fg-muted) hover:bg-(--color-bg-overlay)"
      >
        <X size={14} />
      </button>
    </header>

    <div class="flex-1 overflow-y-auto px-5 py-4">
      {#if loading}
        <div class="grid place-items-center py-10 text-(--color-fg-muted)">
          <Loader2 size={20} class="animate-spin" />
        </div>
      {:else if !notetype}
        <p class="text-sm text-(--color-danger)">{t("note.notetypeNotFound")}</p>
      {:else}
        <div class="space-y-3">
          {#if mode === "add"}
            <div class="grid grid-cols-2 gap-3">
              <label class="block">
                <span class="mb-1 block text-[11px] tracking-wider text-(--color-fg-subtle) uppercase">
                  {t("browse.deck")}
                </span>
                <select
                  bind:value={deckId}
                  class="w-full rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-base) px-3 py-1.5 text-sm shadow-(--shadow-subtle) outline-none focus:border-(--color-accent-500)"
                >
                  {#each collection.decks as d (d.id)}
                    <option value={d.id}>{d.name}</option>
                  {/each}
                </select>
              </label>
              <label class="block">
                <span class="mb-1 block text-[11px] tracking-wider text-(--color-fg-subtle) uppercase">
                  {t("note.notetype")}
                </span>
                <select
                  value={notetype.id}
                  onchange={(e) => {
                    const id = Number((e.currentTarget as HTMLSelectElement).value);
                    const nt = notes.notetypes.find((n) => n.id === id);
                    if (nt) selectNotetype(nt);
                  }}
                  class="w-full rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-base) px-3 py-1.5 text-sm shadow-(--shadow-subtle) outline-none focus:border-(--color-accent-500)"
                >
                  {#each notes.notetypes as nt (nt.id)}
                    <option value={nt.id}>{nt.name}</option>
                  {/each}
                </select>
              </label>
            </div>
          {:else}
            <p class="text-xs text-(--color-fg-subtle)">
              {t("note.notetype")}: <span class="text-(--color-fg-default)">{notetype.name}</span>
            </p>
          {/if}

          {#each notetype.field_names as fname, i (i)}
            <div>
              <span class="mb-1 block text-[11px] tracking-wider text-(--color-fg-subtle) uppercase">
                {fname}
              </span>
              <!--
                Anki note fields are stored as HTML, so a <textarea> would
                show raw <b>/<i>/<br> markup. A contenteditable div renders
                the markup, lets the user edit visually, and the standard
                Cmd+B / Cmd+I / Cmd+U / Enter (=> <br>) commands work via
                the browser's built-in editing. bind:innerHTML keeps the
                state field in sync with the rendered DOM.
              -->
              <div
                bind:this={fieldEls[i]}
                contenteditable="true"
                role="textbox"
                aria-label={fname}
                aria-multiline="true"
                bind:innerHTML={fields[i]}
                onpaste={(e) => {
                  // ブラウザ標準のリッチテキスト貼り付けはコピー元の
                  // <span style=...>、<font color=...>、Office 系の
                  // class や mso-* スタイル等をそのまま Anki DB に
                  // 残してしまう。常にプレーンテキストとして挿入する。
                  e.preventDefault();
                  const text = e.clipboardData?.getData("text/plain") ?? "";
                  if (text) document.execCommand("insertText", false, text);
                }}
                class="block min-h-[7rem] w-full rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-base) px-3 py-2 text-sm leading-relaxed shadow-(--shadow-subtle) outline-none focus:border-(--color-accent-500) [&_*]:max-w-full"
              ></div>
            </div>
          {/each}

          <label class="block">
            <span class="mb-1 block text-[11px] tracking-wider text-(--color-fg-subtle) uppercase">
              {t("note.tagsLabel")}
            </span>
            <input
              type="text"
              bind:value={tagsText}
              class="w-full rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-base) px-3 py-1.5 text-sm shadow-(--shadow-subtle) outline-none focus:border-(--color-accent-500)"
            />
          </label>
        </div>
      {/if}

      {#if notes.lastError}
        <p class="mt-3 flex items-start gap-1.5 text-xs text-(--color-danger)">
          <AlertCircle size={12} class="mt-0.5 shrink-0" />
          <span class="break-all">{notes.lastError}</span>
        </p>
      {/if}
    </div>

    <footer
      class="flex items-center justify-between gap-2 border-t border-(--color-border-default) px-5 py-3"
    >
      <div>
        {#if mode === "edit"}
          <button
            type="button"
            onclick={deleteNote}
            disabled={notes.busy}
            class="flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-danger)/40 bg-(--color-danger)/10 px-3 py-1.5 text-xs font-medium text-(--color-danger) hover:bg-(--color-danger)/20 disabled:opacity-50"
          >
            <Trash2 size={12} /> {t("note.delete")}
          </button>
        {/if}
      </div>
      <div class="flex items-center gap-2">
        <button
          type="button"
          onclick={onClose}
          class="rounded-(--radius-md) border border-(--color-border-strong) px-3 py-1.5 text-xs text-(--color-fg-default) hover:bg-(--color-bg-overlay)"
        >
          {t("note.cancel")}
        </button>
        <button
          type="button"
          onclick={save}
          disabled={notes.busy || !notetype}
          class="flex items-center gap-1.5 rounded-(--radius-md) bg-(--color-accent-500) px-3 py-1.5 text-xs font-medium text-(--color-fg-onAccent) shadow-(--shadow-subtle) hover:bg-(--color-accent-600) active:scale-[0.97] disabled:opacity-50"
        >
          {#if notes.busy}
            <Loader2 size={12} class="animate-spin" />
          {:else}
            <Save size={12} />
          {/if}
          {mode === "add" ? t("note.add") : t("note.save")}
          <span class="ml-1 font-mono text-[10px] opacity-70">⌘↵</span>
        </button>
      </div>
    </footer>
  </div>
</div>
