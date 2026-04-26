<script lang="ts">
  import { Brain, Sparkles, Plus, FolderOpen, FilePlus } from "lucide-svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import { goto } from "$app/navigation";
  import NoteEditor from "$lib/components/NoteEditor.svelte";

  const selected = $derived(collection.selectedDeck);
  const totalDue = $derived(
    selected
      ? selected.new_count + selected.learn_count + selected.review_count
      : 0,
  );

  async function pickAndOpen() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const picked = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "Anki collection", extensions: ["anki2"] }],
      });
      if (typeof picked === "string") await collection.open(picked);
    } catch (e) {
      console.error(e);
    }
  }

  async function createNew() {
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const picked = await save({
        defaultPath: "memorize-collection.anki2",
        filters: [{ name: "Anki collection", extensions: ["anki2"] }],
      });
      if (typeof picked !== "string") return;
      // CollectionBuilder creates the SQLite db on first build if not present.
      await collection.open(picked);
    } catch (e) {
      console.error(e);
    }
  }

  function startStudy() {
    if (selected && totalDue > 0) goto(`/review/${selected.id}/`);
  }

  let showAddNote = $state(false);

  async function onWordAdded() {
    await collection.refreshDecks();
  }

  type Tone = "accent" | "warning" | "success";
  const toneRing: Record<Tone, string> = {
    accent: "from-(--color-accent-500)/15 to-(--color-accent-500)/0",
    warning: "from-(--color-warning)/15 to-(--color-warning)/0",
    success: "from-(--color-success)/15 to-(--color-success)/0",
  };
  const toneText: Record<Tone, string> = {
    accent: "text-(--color-accent-500)",
    warning: "text-(--color-warning)",
    success: "text-(--color-success)",
  };
</script>

<div class="mx-auto h-full max-w-4xl px-8 py-12">
  {#if !collection.isOpen}
    <div class="grid h-full place-items-center">
      <div class="flex max-w-md flex-col items-center gap-6 text-center">
        <div
          class="grid h-16 w-16 place-items-center rounded-2xl bg-(--color-accent-500) text-(--color-fg-onAccent) shadow-(--shadow-glow)"
        >
          <Brain size={32} strokeWidth={2.25} />
        </div>
        <div class="space-y-2">
          <h1 class="font-display text-3xl font-medium tracking-tight">
            ようこそ
          </h1>
          <p class="text-sm leading-relaxed text-(--color-fg-muted)">
            既存の Anki コレクション (.anki2) を開いて始めましょう。
            <br />Sync・Import / Export は後の Phase で対応します。
          </p>
        </div>
        <div class="flex w-full flex-col gap-2">
          <button
            type="button"
            onclick={pickAndOpen}
            class="flex items-center justify-center gap-2 rounded-(--radius-md) bg-(--color-accent-500) px-5 py-2.5 text-sm font-medium whitespace-nowrap text-(--color-fg-onAccent) shadow-(--shadow-card) transition-all duration-200 hover:bg-(--color-accent-600) active:scale-[0.97]"
          >
            <FolderOpen size={16} strokeWidth={2.25} />
            既存のコレクションを開く
          </button>
          <button
            type="button"
            onclick={createNew}
            class="flex items-center justify-center gap-2 rounded-(--radius-md) border border-(--color-border-strong) bg-(--color-bg-elevated) px-5 py-2.5 text-sm font-medium whitespace-nowrap text-(--color-fg-default) shadow-(--shadow-subtle) transition-all duration-200 hover:bg-(--color-bg-overlay) active:scale-[0.97]"
          >
            <FilePlus size={16} strokeWidth={2.25} />
            新規コレクションを作成
          </button>
        </div>
        {#if collection.error}
          <p class="text-xs text-(--color-danger)">{collection.error}</p>
        {/if}
      </div>
    </div>
  {:else if selected}
    <div class="flex flex-col gap-12">
      <header class="animate-count">
        <p
          class="text-[11px] font-semibold tracking-[0.14em] text-(--color-fg-subtle) uppercase"
        >
          選択中のデッキ
        </p>
        <h1
          class="mt-1.5 font-display text-[2.25rem] leading-tight font-medium tracking-tight"
        >
          {selected.name.split("::").at(-1)}
        </h1>
        {#if selected.name.includes("::")}
          <p class="mt-1 font-mono text-xs text-(--color-fg-subtle)">
            {selected.name}
          </p>
        {/if}
      </header>

      <div class="grid grid-cols-3 gap-4">
        {@render countCard("New", selected.new_count, "accent", 0)}
        {@render countCard("Learning", selected.learn_count, "warning", 60)}
        {@render countCard("Review", selected.review_count, "success", 120)}
      </div>

      <div class="flex flex-col items-center gap-3">
        <button
          type="button"
          onclick={startStudy}
          disabled={totalDue === 0}
          class="group relative flex items-center gap-2 overflow-hidden rounded-full bg-(--color-accent-500) px-7 py-3 text-sm font-medium text-(--color-fg-onAccent) shadow-(--shadow-card) transition-all duration-200 hover:bg-(--color-accent-600) hover:shadow-(--shadow-glow) active:scale-[0.97] disabled:cursor-not-allowed disabled:bg-(--color-bg-overlay) disabled:text-(--color-fg-subtle) disabled:shadow-none enabled:pulse-soft"
        >
          <Sparkles
            size={16}
            strokeWidth={2.25}
            class="transition-transform duration-300 group-hover:rotate-12 group-disabled:rotate-0"
          />
          Study Now
        </button>
        <p class="text-xs text-(--color-fg-subtle) tabular-nums">
          {totalDue > 0 ? `${totalDue} cards waiting` : "今日は終わりました"}
        </p>
        <button
          type="button"
          onclick={() => (showAddNote = true)}
          class="mt-1 flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-border-strong) px-3 py-1.5 text-xs text-(--color-fg-default) transition-colors hover:bg-(--color-bg-overlay) active:scale-[0.98]"
        >
          <Plus size={12} strokeWidth={2.5} />
          このデッキに単語を追加
        </button>
      </div>
    </div>
  {:else}
    <div class="grid h-full place-items-center">
      <div class="flex flex-col items-center gap-3 text-(--color-fg-muted)">
        <Plus size={32} strokeWidth={1.5} />
        <p class="text-sm">デッキがありません</p>
      </div>
    </div>
  {/if}
</div>

{#if showAddNote && selected}
  <NoteEditor
    mode="add"
    initialDeckId={selected.id}
    onClose={() => (showAddNote = false)}
    onSaved={onWordAdded}
  />
{/if}

{#snippet countCard(label: string, count: number, tone: Tone, delayMs: number)}
  <div
    class="animate-count relative overflow-hidden rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) px-6 py-7 shadow-(--shadow-subtle) transition-shadow hover:shadow-(--shadow-card)"
    style="animation-delay: {delayMs}ms; animation-fill-mode: backwards;"
  >
    <div
      class="pointer-events-none absolute inset-0 bg-gradient-to-br {toneRing[tone]}"
    ></div>
    <div class="relative flex flex-col items-center gap-1">
      <p
        class="text-[10px] font-semibold tracking-[0.16em] text-(--color-fg-subtle) uppercase"
      >
        {label}
      </p>
      <p class="number-tabular font-display text-5xl font-medium {toneText[tone]}">
        {count}
      </p>
    </div>
  </div>
{/snippet}
