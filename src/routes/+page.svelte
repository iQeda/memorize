<script lang="ts">
  import { Brain, Sparkles, Plus, FolderOpen, FilePlus } from "lucide-svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import { goto } from "$app/navigation";
  import NoteEditor from "$lib/components/NoteEditor.svelte";
  import FutureDueChart from "$lib/components/charts/FutureDueChart.svelte";
  import CardCountsPie from "$lib/components/charts/CardCountsPie.svelte";
  import HistogramChart from "$lib/components/charts/HistogramChart.svelte";
  import StackedBarChart from "$lib/components/charts/StackedBarChart.svelte";
  import HoursChart from "$lib/components/charts/HoursChart.svelte";
  import ButtonsChart from "$lib/components/charts/ButtonsChart.svelte";
  import RetentionTable from "$lib/components/charts/RetentionTable.svelte";
  import CalendarHeatmap from "$lib/components/charts/CalendarHeatmap.svelte";
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
  type Bucket<K> = { key: K; value: number };
  type CardCountsBreakdown = {
    new_cards: number;
    learn: number;
    relearn: number;
    young: number;
    mature: number;
    suspended: number;
    buried: number;
  };
  type ReviewsBucket = {
    day: number;
    learn: number;
    relearn: number;
    young: number;
    mature: number;
    filtered: number;
  };
  type ButtonsCounts = { learning: number[]; young: number[]; mature: number[] };
  type ButtonsByRange = {
    one_month: ButtonsCounts;
    three_months: ButtonsCounts;
    one_year: ButtonsCounts;
  };
  type HourBucket = { hour: number; total: number; correct: number };
  type HoursByRange = {
    one_month: HourBucket[];
    three_months: HourBucket[];
    one_year: HourBucket[];
  };
  type TrueRetention = {
    young_passed: number;
    young_failed: number;
    mature_passed: number;
    mature_failed: number;
  };
  type RetentionStats = {
    today: TrueRetention;
    yesterday: TrueRetention;
    week: TrueRetention;
    month: TrueRetention;
    year: TrueRetention;
    all_time: TrueRetention;
  };
  type DeckGraphStats = {
    today: TodayStats;
    future_due: Bucket<number>[];
    future_due_total: number;
    future_due_avg_per_day: number;
    future_due_have_backlog: boolean;
    daily_load: number;
    card_counts_separate: CardCountsBreakdown;
    card_counts_combined: CardCountsBreakdown;
    intervals: Bucket<number>[];
    eases: Bucket<number>[];
    eases_average: number;
    reviews: ReviewsBucket[];
    added: Bucket<number>[];
    buttons: ButtonsByRange;
    hours: HoursByRange;
    retention: RetentionStats;
  };

  let separateInactive = $state(true);
  let buttonsRange = $state<"one_month" | "three_months" | "one_year">("one_month");
  let hoursRange = $state<"one_month" | "three_months" | "one_year">("one_month");

  let stats = $state<DeckStats | null>(null);
  let statsDeckId = $state<number | null>(null);
  let graph = $state<DeckGraphStats | null>(null);
  let graphDays = $state<number>(31);
  let graphError = $state<string | null>(null);

  $effect(() => {
    const dId = collection.selectedDeckId;
    const days = graphDays;
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

  const pickAndOpen = () => collection.pickAndOpen();
  const createNew = () => collection.createNew();

  function startStudy() {
    if (selected && totalDue > 0) goto(`/review/${selected.id}/`);
  }

  function onKey(e: KeyboardEvent) {
    if (e.repeat || e.defaultPrevented) return;
    // Skip when the user is typing in a form field (deck rename, note editor,
    // launcher search, etc).
    const target = e.target as HTMLElement | null;
    if (
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target instanceof HTMLSelectElement ||
      target?.isContentEditable
    ) {
      return;
    }
    if (e.key === "Enter" && selected && totalDue > 0) {
      e.preventDefault();
      startStudy();
      return;
    }
    if (
      (e.key === "n" || e.key === "N") &&
      !e.metaKey &&
      !e.ctrlKey &&
      !e.altKey &&
      collection.isOpen
    ) {
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
          <span class="ml-1 font-mono text-[10px] opacity-70">↵</span>
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
        {@const futureDueChartBuckets = graph!.future_due.map((b) => ({ day: b.key, count: b.value }))}
        {@const reviewsCols = (() => {
          // Map day (0 today, negative past) to columns from min to 0
          const minDay = Math.min(0, ...graph!.reviews.map((r) => r.day));
          const cols = -minDay + 1;
          const series = [
            { label: "Learn", color: "#f5a623", values: Array(cols).fill(0) },
            { label: "Relearn", color: "#e26d6d", values: Array(cols).fill(0) },
            { label: "Young", color: "#7ed87a", values: Array(cols).fill(0) },
            { label: "Mature", color: "#3aa050", values: Array(cols).fill(0) },
            { label: "Filtered", color: "#7c8aff", values: Array(cols).fill(0) },
          ];
          for (const r of graph!.reviews) {
            const i = r.day - minDay;
            if (i < 0 || i >= cols) continue;
            series[0].values[i] = r.learn;
            series[1].values[i] = r.relearn;
            series[2].values[i] = r.young;
            series[3].values[i] = r.mature;
            series[4].values[i] = r.filtered;
          }
          return { cols, series, minDay };
        })()}
        {@const calendarPerDay = graph!.reviews.map((r) => ({ day: r.day, total: r.learn + r.relearn + r.young + r.mature + r.filtered }))}
        {@const addedCols = (() => {
          const minDay = Math.min(0, ...graph!.added.map((b) => b.key));
          const cols = -minDay + 1;
          const values = Array(cols).fill(0);
          for (const b of graph!.added) {
            const i = b.key - minDay;
            if (i >= 0 && i < cols) values[i] = b.value;
          }
          return { cols, values, minDay };
        })()}

        <section class="grid auto-rows-[260px] grid-cols-1 gap-3 lg:grid-cols-2">
          <!-- Today -->
          {@render panel(t("decks.today"), null, todayBlock)}

          <!-- Future Due -->
          {@render panel(t("decks.futureDue"), futureDueRange, futureDueBlock)}

          <!-- Calendar -->
          {@render panel(t("decks.calendar"), null, calendarBlock)}

          <!-- Reviews -->
          {@render panel(t("decks.reviews"), null, reviewsBlock)}

          <!-- Card Counts -->
          {@render panel(t("decks.cardCounts"), cardCountsToggle, cardCountsBlock)}

          <!-- Review Intervals -->
          {@render panel(t("decks.intervals"), null, intervalsBlock)}

          <!-- Card Ease -->
          {@render panel(t("decks.cardEase"), null, easeBlock)}

          <!-- Retention -->
          {@render panel(t("decks.retention"), null, retentionBlock)}

          <!-- Hourly -->
          {@render panel(t("decks.hourly"), hoursRangeBtns, hoursBlock)}

          <!-- Answer Buttons -->
          {@render panel(t("decks.answerButtons"), buttonsRangeBtns, buttonsBlock)}

          <!-- Added -->
          {@render panel(t("decks.added"), null, addedBlock)}
        </section>

        {#snippet todayBlock()}
          {#if graph!.today.answer_count === 0}
            <p class="mt-2 text-sm text-(--color-fg-muted)">{t("decks.todayEmpty")}</p>
          {:else}
            <p class="mt-2 text-sm text-(--color-fg-default)">
              {t("decks.todayCount", {
                count: graph!.today.answer_count,
                minutes: formatDuration(graph!.today.answer_millis),
              })}
            </p>
          {/if}
        {/snippet}

        {#snippet futureDueRange()}
          <div class="flex gap-1 text-[11px]">
            {@render rangeBtn(31, t("decks.range1m"))}
            {@render rangeBtn(92, t("decks.range3m"))}
            {@render rangeBtn(365, t("decks.range1y"))}
          </div>
        {/snippet}

        {#snippet futureDueBlock()}
          <p class="mt-1 text-xs text-(--color-fg-subtle)">
            {t("decks.futureDueDesc", { days: graphDays })}
          </p>
          <div class="mt-2 text-(--color-fg-muted)">
            <FutureDueChart buckets={futureDueChartBuckets} days={graphDays} />
          </div>
          <div class="mt-1 flex justify-between text-[11px] text-(--color-fg-subtle) tabular-nums">
            <span>{t("decks.futureDueTotal", { count: graph!.future_due_total })}</span>
            <span>{t("decks.futureDueAvg", { avg: graph!.future_due_avg_per_day.toFixed(1) })}</span>
          </div>
        {/snippet}

        {#snippet calendarBlock()}
          <div class="mt-2 overflow-x-auto text-(--color-fg-muted)">
            <CalendarHeatmap perDay={calendarPerDay} />
          </div>
        {/snippet}

        {#snippet reviewsBlock()}
          <p class="mt-1 text-xs text-(--color-fg-subtle)">{t("decks.reviewsDesc")}</p>
          <div class="mt-2">
            <StackedBarChart
              columns={reviewsCols.cols}
              series={reviewsCols.series}
              minCol={reviewsCols.minDay}
              xFormat={(c) => `${c}`}
            />
          </div>
        {/snippet}

        {#snippet cardCountsToggle()}
          <label class="flex items-center gap-1.5 text-[11px] text-(--color-fg-muted)">
            <input
              type="checkbox"
              bind:checked={separateInactive}
              class="h-3 w-3 accent-(--color-accent-500)"
            />
            {t("decks.separateInactive")}
          </label>
        {/snippet}

        {#snippet cardCountsBlock()}
          <div class="mt-3">
            <CardCountsPie
              counts={separateInactive ? graph!.card_counts_separate : graph!.card_counts_combined}
            />
          </div>
        {/snippet}

        {#snippet intervalsBlock()}
          <p class="mt-1 text-xs text-(--color-fg-subtle)">{t("decks.intervalsDesc")}</p>
          <div class="mt-2 text-(--color-fg-muted)">
            <HistogramChart
              buckets={graph!.intervals}
              xFormat={(v) => `${v}d`}
              color="var(--color-accent-500)"
            />
          </div>
        {/snippet}

        {#snippet easeBlock()}
          <p class="mt-1 text-xs text-(--color-fg-subtle)">{t("decks.cardEaseDesc")}</p>
          <div class="mt-2 text-(--color-fg-muted)">
            <HistogramChart
              buckets={graph!.eases.map((b) => ({ key: Math.round(b.key / 10), value: b.value }))}
              xFormat={(v) => `${v}%`}
              color="var(--color-success)"
            />
          </div>
          {#if graph!.eases_average > 0}
            <p class="mt-1 text-center text-[11px] text-(--color-fg-subtle)">
              Median ease: {(graph!.eases_average / 10).toFixed(0)}%
            </p>
          {/if}
        {/snippet}

        {#snippet retentionBlock()}
          <p class="mt-1 text-xs text-(--color-fg-subtle)">{t("decks.retentionDesc")}</p>
          <div class="mt-3">
            <RetentionTable {...graph!.retention} />
          </div>
        {/snippet}

        {#snippet hoursRangeBtns()}
          <div class="flex gap-1 text-[11px]">
            {@render hRangeBtn("one_month", t("decks.range1m"))}
            {@render hRangeBtn("three_months", t("decks.range3m"))}
            {@render hRangeBtn("one_year", t("decks.range1y"))}
          </div>
        {/snippet}

        {#snippet hoursBlock()}
          <p class="mt-1 text-xs text-(--color-fg-subtle)">{t("decks.hourlyDesc")}</p>
          <div class="mt-2 text-(--color-fg-muted)">
            <HoursChart hours={graph!.hours[hoursRange]} />
          </div>
        {/snippet}

        {#snippet buttonsRangeBtns()}
          <div class="flex gap-1 text-[11px]">
            {@render bRangeBtn("one_month", t("decks.range1m"))}
            {@render bRangeBtn("three_months", t("decks.range3m"))}
            {@render bRangeBtn("one_year", t("decks.range1y"))}
          </div>
        {/snippet}

        {#snippet buttonsBlock()}
          <p class="mt-1 text-xs text-(--color-fg-subtle)">{t("decks.answerButtonsDesc")}</p>
          <div class="mt-2 text-(--color-fg-muted)">
            <ButtonsChart counts={graph!.buttons[buttonsRange]} />
          </div>
        {/snippet}

        {#snippet addedBlock()}
          <p class="mt-1 text-xs text-(--color-fg-subtle)">{t("decks.addedDesc")}</p>
          <div class="mt-2 text-(--color-fg-muted)">
            <StackedBarChart
              columns={addedCols.cols}
              series={[{ label: "Added", color: "var(--color-accent-500)", values: addedCols.values }]}
              minCol={addedCols.minDay}
              xFormat={(c) => `${c}`}
            />
          </div>
        {/snippet}
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

{#snippet panel(title: string, controls: import("svelte").Snippet | null, body: import("svelte").Snippet)}
  <section class="flex h-full flex-col overflow-hidden rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-3.5 shadow-(--shadow-subtle)">
    <div class="flex flex-wrap items-baseline justify-between gap-2">
      <h2 class="text-[10px] font-semibold tracking-[0.16em] text-(--color-fg-subtle) uppercase">
        {title}
      </h2>
      {#if controls}{@render controls()}{/if}
    </div>
    <div class="min-h-0 flex-1 overflow-hidden">
      {@render body()}
    </div>
  </section>
{/snippet}

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

{#snippet hRangeBtn(range: "one_month" | "three_months" | "one_year", label: string)}
  {@const active = hoursRange === range}
  <button
    type="button"
    onclick={() => (hoursRange = range)}
    class="rounded-(--radius-sm) px-2 py-0.5 transition-colors
      {active
      ? 'bg-(--color-accent-500) text-(--color-fg-onAccent)'
      : 'text-(--color-fg-muted) hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)'}"
  >
    {label}
  </button>
{/snippet}

{#snippet bRangeBtn(range: "one_month" | "three_months" | "one_year", label: string)}
  {@const active = buttonsRange === range}
  <button
    type="button"
    onclick={() => (buttonsRange = range)}
    class="rounded-(--radius-sm) px-2 py-0.5 transition-colors
      {active
      ? 'bg-(--color-accent-500) text-(--color-fg-onAccent)'
      : 'text-(--color-fg-muted) hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)'}"
  >
    {label}
  </button>
{/snippet}

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
