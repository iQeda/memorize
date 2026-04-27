<script lang="ts">
  import { Brain, Sparkles, Plus, FolderOpen, FilePlus } from "lucide-svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import { goto } from "$app/navigation";
  import NoteEditor from "$lib/components/NoteEditor.svelte";
  import FutureDueChart from "$lib/components/FutureDueChart.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { invoke } from "$lib/ipc";

  type DeckStats = {
    total_cards: number;
    total_notes: number;
    new_cards: number;
    learn_cards: number;
    review_cards: number;
    suspended: number;
    buried: number;
  };
  type TodayStats = {
    answer_count: number;
    answer_millis: number;
    correct_count: number;
    mature_count: number;
    mature_correct: number;
    learn_count: number;
    review_count: number;
    relearn_count: number;
  };
  type FutureDueBucket = { day: number; count: number };
  type DeckGraphStats = {
    today: TodayStats;
    future_due: FutureDueBucket[];
    future_due_total: number;
    future_due_avg_per_day: number;
    future_due_have_backlog: boolean;
    daily_load: number;
  };

  let stats = $state<DeckStats | null>(null);
  let statsDeckId = $state<number | null>(null);
  let graph = $state<DeckGraphStats | null>(null);
  let graphDays = $state<number>(31);

  $effect(() => {
    const dId = collection.selectedDeckId;
    const days = graphDays;
    if (dId === null || !collection.isOpen) {
      stats = null;
      graph = null;
      statsDeckId = null;
      return;
    }
    statsDeckId = dId;
    void (async () => {
      try {
        const [s, g] = await Promise.all([
          invoke<DeckStats>("deck_stats", { deckId: dId }),
          invoke<DeckGraphStats>("deck_graph_stats", { deckId: dId, days }),
        ]);
        stats = s;
        graph = g;
      } catch (e) {
        console.error("deck stats", e);
        stats = null;
        graph = null;
      }
    })();
  });

  function formatDuration(ms: number): string {
    if (ms < 60_000) return `${Math.round(ms / 1000)}s`;
    const mins = Math.round(ms / 60_000);
    if (mins < 60) return `${mins}m`;
    const hrs = Math.floor(mins / 60);
    return `${hrs}h ${mins % 60}m`;
  }

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
    if (statsDeckId !== null) {
      try {
        stats = await invoke<DeckStats>("deck_stats", { deckId: statsDeckId });
      } catch {}
    }
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
            {t("welcome.title")}
          </h1>
          <p class="text-sm leading-relaxed whitespace-pre-line text-(--color-fg-muted)">
            {t("welcome.body")}
          </p>
        </div>
        <div class="flex w-full flex-col gap-2">
          <button
            type="button"
            onclick={pickAndOpen}
            class="flex items-center justify-center gap-2 rounded-(--radius-md) bg-(--color-accent-500) px-5 py-2.5 text-sm font-medium whitespace-nowrap text-(--color-fg-onAccent) shadow-(--shadow-card) transition-all duration-200 hover:bg-(--color-accent-600) active:scale-[0.97]"
          >
            <FolderOpen size={16} strokeWidth={2.25} />
            {t("welcome.openExisting")}
          </button>
          <button
            type="button"
            onclick={createNew}
            class="flex items-center justify-center gap-2 rounded-(--radius-md) border border-(--color-border-strong) bg-(--color-bg-elevated) px-5 py-2.5 text-sm font-medium whitespace-nowrap text-(--color-fg-default) shadow-(--shadow-subtle) transition-all duration-200 hover:bg-(--color-bg-overlay) active:scale-[0.97]"
          >
            <FilePlus size={16} strokeWidth={2.25} />
            {t("welcome.createNew")}
          </button>
        </div>
        {#if collection.error}
          <p class="text-xs text-(--color-danger)">{collection.error}</p>
        {/if}
      </div>
    </div>
  {:else if selected}
    <div class="flex flex-col gap-12">
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
        </div>
        <button
          type="button"
          onclick={() => (showAddNote = true)}
          class="shrink-0 mt-1 flex items-center gap-1.5 rounded-(--radius-md) border border-(--color-border-strong) bg-(--color-bg-elevated) px-3 py-1.5 text-xs font-medium text-(--color-fg-default) transition-colors hover:bg-(--color-bg-overlay) active:scale-[0.98]"
        >
          <Plus size={12} strokeWidth={2.5} />
          {t("decks.addWord")}
        </button>
      </header>

      <div class="grid grid-cols-3 gap-4">
        {@render countCard(t("decks.new"), selected.new_count, "accent", 0)}
        {@render countCard(t("decks.learning"), selected.learn_count, "warning", 60)}
        {@render countCard(t("decks.review"), selected.review_count, "success", 120)}
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
        </button>
        <p class="text-xs text-(--color-fg-subtle) tabular-nums">
          {totalDue > 0
            ? t("decks.cardsWaiting", { count: totalDue })
            : t("decks.allDoneToday")}
        </p>
      </div>

      {#if stats}
        <section
          class="animate-count rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-5 shadow-(--shadow-subtle)"
          style="animation-delay: 200ms; animation-fill-mode: backwards;"
        >
          <h2 class="text-[11px] font-semibold tracking-[0.16em] text-(--color-fg-subtle) uppercase">
            {t("decks.stats")}
          </h2>
          <dl class="mt-4 grid grid-cols-2 gap-x-6 gap-y-3 text-sm sm:grid-cols-4">
            {@render stat(t("decks.totalCards"), stats.total_cards)}
            {@render stat(t("decks.totalNotes"), stats.total_notes)}
            {@render stat(t("decks.suspended"), stats.suspended, stats.suspended > 0 ? "warning" : undefined)}
            {@render stat(t("decks.buried"), stats.buried)}
          </dl>
        </section>
      {/if}

      {#if graph}
        <section
          class="animate-count rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-5 shadow-(--shadow-subtle)"
          style="animation-delay: 240ms; animation-fill-mode: backwards;"
        >
          <h2 class="text-[11px] font-semibold tracking-[0.16em] text-(--color-fg-subtle) uppercase">
            {t("decks.today")}
          </h2>
          {#if graph.today.answer_count === 0}
            <p class="mt-3 text-sm text-(--color-fg-muted)">
              {t("decks.todayEmpty")}
            </p>
          {:else}
            <p class="mt-3 text-sm text-(--color-fg-default)">
              {t("decks.todayCount", {
                count: graph.today.answer_count,
                minutes: formatDuration(graph.today.answer_millis),
              })}
            </p>
          {/if}
        </section>

        <section
          class="animate-count rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-5 shadow-(--shadow-subtle)"
          style="animation-delay: 280ms; animation-fill-mode: backwards;"
        >
          <div class="flex flex-wrap items-baseline justify-between gap-3">
            <h2 class="text-[11px] font-semibold tracking-[0.16em] text-(--color-fg-subtle) uppercase">
              {t("decks.futureDue")}
            </h2>
            <div class="flex gap-1 text-[11px]">
              {@render rangeBtn(31, t("decks.range1m"))}
              {@render rangeBtn(92, t("decks.range3m"))}
              {@render rangeBtn(365, t("decks.range1y"))}
            </div>
          </div>
          <p class="mt-1 text-xs text-(--color-fg-subtle)">
            {t("decks.futureDueDesc", { days: graphDays })}
          </p>
          <div class="mt-3 text-(--color-fg-muted)">
            <FutureDueChart buckets={graph.future_due} days={graphDays} />
          </div>
          <div class="mt-1 flex justify-between text-[11px] text-(--color-fg-subtle) tabular-nums">
            <span>{t("decks.futureDueTotal", { count: graph.future_due_total })}</span>
            <span>{t("decks.futureDueAvg", { avg: graph.future_due_avg_per_day.toFixed(1) })}</span>
          </div>
        </section>
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

{#snippet rangeBtn(days: number, label: string)}
  {@const active = graphDays === days}
  <button
    type="button"
    onclick={() => (graphDays = days)}
    class="rounded-(--radius-sm) px-2 py-0.5 transition-colors
      {active
      ? 'bg-(--color-accent-500) text-(--color-fg-onAccent)'
      : 'text-(--color-fg-muted) hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)'}"
  >
    {label}
  </button>
{/snippet}

{#snippet stat(label: string, value: number, tone: "warning" | undefined = undefined)}
  <div class="flex flex-col gap-0.5">
    <dt class="text-[10px] tracking-[0.12em] text-(--color-fg-subtle) uppercase">
      {label}
    </dt>
    <dd
      class="number-tabular text-lg font-medium {tone === 'warning'
        ? 'text-(--color-warning)'
        : 'text-(--color-fg-default)'}"
    >
      {value}
    </dd>
  </div>
{/snippet}

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
