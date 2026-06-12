<script lang="ts">
  import { Sparkles, Plus } from "lucide-svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import { sync } from "$lib/stores/sync.svelte";
  import { goto } from "$app/navigation";
  import NoteEditor from "$lib/components/NoteEditor.svelte";
  import WelcomeScreen from "$lib/components/home/WelcomeScreen.svelte";
  import StatsPanelGrid from "$lib/components/home/StatsPanelGrid.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { invoke } from "$lib/ipc";
  import { hasModifier, isTextFieldTarget } from "$lib/utils/keyboard";
  import type { DeckGraphStats, DeckStats } from "$lib/stats/types";

  let stats = $state<DeckStats | null>(null);
  let statsDeckId = $state<number | null>(null);
  let graph = $state<DeckGraphStats | null>(null);
  let graphDays = $state<number>(31);
  let graphError = $state<string | null>(null);

  // ⌘S で normal_sync を呼ぶと rslib が SQLite を上書きするが、フロント側の
  // collection.decks は (Svelte runes 的に) その変更を検知できない。home の
  // stats/graph effect は collection.decks の差し替えを依存にしているので、
  // sync 完了 (busy: true → false) のタイミングで refreshDecks + refreshInfo
  // を明示的に呼んで decks 配列を新規参照に差し替える。これで下の effect が
  // 自然に反応して deck_stats / deck_graph_stats を再フェッチし、別端末で
  // 学習した結果が home パネルに反映される。reviewer 内の totals 更新は
  // review/[deckId]/+page.svelte 側で別途行っている。
  let prevSyncBusy = false;
  $effect(() => {
    const isBusy = sync.busy;
    if (prevSyncBusy && !isBusy && collection.isOpen) {
      void collection.refresh();
    }
    prevSyncBusy = isBusy;
  });

  $effect(() => {
    const dId = collection.selectedDeckId;
    const days = graphDays;
    // collection.decks は refreshDecks() で配列ごと差し替えられる。これを
    // 依存に加えることで、review 中の answer / sync 完了 / 単語追加で
    // collection.decks が更新されるたびに、選択中デッキの stats と graph
    // パネルも自動で再フェッチされる (従来は selectedDeckId / graphDays が
    // 変わらないと固まったままだった)。
    void collection.decks;
    if (dId === null || !collection.isOpen) {
      stats = null;
      graph = null;
      statsDeckId = null;
      graphError = null;
      return;
    }
    statsDeckId = dId;
    void (async () => {
      try {
        stats = await invoke<DeckStats>("deck_stats", { deckId: dId });
      } catch (e) {
        console.error("deck_stats", e);
        stats = null;
      }
      try {
        graph = await invoke<DeckGraphStats>("deck_graph_stats", {
          deckId: dId,
          days,
        });
        graphError = null;
      } catch (e) {
        console.error("deck_graph_stats", e);
        graph = null;
        graphError = String(e);
      }
    })();
  });

  const selected = $derived(collection.selectedDeck);
  const totalDue = $derived(
    selected
      ? selected.new_count + selected.learn_count + selected.review_count
      : 0,
  );

  function startStudy() {
    if (selected && totalDue > 0) goto(`/review/${selected.id}/`);
  }

  function onKey(e: KeyboardEvent) {
    if (e.repeat || e.defaultPrevented) return;
    // Skip when the user is typing in a form field (deck rename, note editor,
    // launcher search, etc).
    if (isTextFieldTarget(e.target)) return;
    if ((e.key === "Enter" || e.key === " ") && selected && totalDue > 0) {
      e.preventDefault();
      startStudy();
      return;
    }
    if ((e.key === "n" || e.key === "N") && !hasModifier(e) && collection.isOpen) {
      e.preventDefault();
      showAddNote = true;
    }
  }

  let showAddNote = $state(false);

  async function onWordAdded() {
    await collection.refreshDecks();
    if (statsDeckId !== null) {
      try {
        stats = await invoke<DeckStats>("deck_stats", { deckId: statsDeckId });
      } catch {}
    }
  }

  type Tone = "accent" | "warning" | "success" | "muted";
  const toneRing: Record<Tone, string> = {
    accent: "from-(--color-accent-500)/15 to-(--color-accent-500)/0",
    warning: "from-(--color-warning)/15 to-(--color-warning)/0",
    success: "from-(--color-success)/15 to-(--color-success)/0",
    muted: "from-(--color-fg-subtle)/8 to-(--color-fg-subtle)/0",
  };
  const toneText: Record<Tone, string> = {
    accent: "text-(--color-accent-500)",
    warning: "text-(--color-warning)",
    success: "text-(--color-success)",
    muted: "text-(--color-fg-muted)",
  };
</script>

<svelte:window onkeydown={onKey} />

<div class="mx-auto h-full max-w-5xl px-8 py-10">
  {#if !collection.isOpen}
    <WelcomeScreen />
  {:else if selected}
    <div class="flex flex-col gap-8">
      <header class="animate-count flex items-start justify-between gap-4">
        <div class="min-w-0">
          <p
            class="text-[11px] font-semibold tracking-[0.14em] text-(--color-fg-subtle) uppercase"
          >
            {t("decks.selectedHeader")}
          </p>
          <h1
            class="mt-1.5 truncate font-display text-[2.25rem] leading-tight font-medium tracking-tight"
          >
            {selected.name.split("::").at(-1)}
          </h1>
          {#if selected.name.includes("::")}
            <p class="mt-1 truncate font-mono text-xs text-(--color-fg-subtle)">
              {selected.name}
            </p>
          {/if}
          {#if stats}
            <p class="mt-2 text-xs text-(--color-fg-subtle) tabular-nums">
              {t("decks.totalNotes")}: <span class="text-(--color-fg-muted)">{stats.total_notes}</span>
            </p>
          {/if}
        </div>
        <button
          type="button"
          onclick={() => (showAddNote = true)}
          class="shrink-0 mt-1 flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-border-strong) bg-(--color-bg-elevated) px-3 py-1.5 text-xs font-medium text-(--color-fg-default) transition-colors hover:bg-(--color-bg-overlay) active:scale-[0.98]"
        >
          <Plus size={12} strokeWidth={2.5} />
          {t("decks.addWord")}
          <span class="ml-1 font-mono text-[10px] opacity-70">N</span>
        </button>
      </header>

      <div class="grid grid-cols-5 gap-3">
        {@render countCard(t("decks.new"), selected.new_count, "accent", 0)}
        {@render countCard(t("decks.learning"), selected.learn_count, "warning", 40)}
        {@render countCard(t("decks.review"), selected.review_count, "success", 80)}
        {@render countCard(t("decks.suspended"), stats?.suspended ?? 0, "muted", 120)}
        {@render countCard(t("decks.buried"), stats?.buried ?? 0, "muted", 160)}
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
          {t("decks.studyNow")}
          <span class="ml-1 font-mono text-[10px] opacity-70">↵ / Space</span>
        </button>
        <p class="text-xs text-(--color-fg-subtle) tabular-nums">
          {totalDue > 0
            ? t("decks.cardsWaiting", { count: totalDue })
            : t("decks.allDoneToday")}
        </p>
      </div>

      {#if graphError}
        <section class="rounded-(--radius-lg) border border-(--color-danger)/40 bg-(--color-danger)/10 p-4 text-xs text-(--color-danger)">
          deck_graph_stats failed: <span class="break-all font-mono">{graphError}</span>
        </section>
      {/if}

      {#if graph}
        <StatsPanelGrid {graph} {graphDays} onDaysChange={(d) => (graphDays = d)} />
      {/if}
    </div>
  {:else}
    <div class="grid h-full place-items-center">
      <div class="flex flex-col items-center gap-3 text-(--color-fg-muted)">
        <Plus size={32} strokeWidth={1.5} />
        <p class="text-sm">{t("decks.empty")}</p>
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
    class="animate-count relative overflow-hidden rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) px-3 py-5 shadow-(--shadow-subtle) transition-shadow hover:shadow-(--shadow-card)"
    style="animation-delay: {delayMs}ms; animation-fill-mode: backwards;"
  >
    <div
      class="pointer-events-none absolute inset-0 bg-gradient-to-br {toneRing[tone]}"
    ></div>
    <div class="relative flex flex-col items-center gap-1">
      <p
        class="text-[9px] font-semibold tracking-[0.14em] text-(--color-fg-subtle) uppercase"
      >
        {label}
      </p>
      <p class="number-tabular font-display text-4xl font-medium {toneText[tone]}">
        {count}
      </p>
    </div>
  </div>
{/snippet}
