<script lang="ts">
  // ホームの統計パネルグリッド (11 パネル)。+page.svelte からの純移動。
  // パネルは 260px 固定高の 2 カラムグリッド (auto-rows-[260px])。
  import FutureDueChart from "$lib/components/charts/FutureDueChart.svelte";
  import CardCountsPie from "$lib/components/charts/CardCountsPie.svelte";
  import HistogramChart from "$lib/components/charts/HistogramChart.svelte";
  import StackedBarChart from "$lib/components/charts/StackedBarChart.svelte";
  import HoursChart from "$lib/components/charts/HoursChart.svelte";
  import ButtonsChart from "$lib/components/charts/ButtonsChart.svelte";
  import RetentionTable from "$lib/components/charts/RetentionTable.svelte";
  import CalendarHeatmap from "$lib/components/charts/CalendarHeatmap.svelte";
  import RangeTabs from "$lib/components/RangeTabs.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import type { DeckGraphStats } from "$lib/stats/types";
  import {
    buildAddedCols,
    buildCalendarPerDay,
    buildReviewsCols,
    formatDuration,
  } from "$lib/stats/transform";

  type Props = {
    graph: DeckGraphStats;
    graphDays: number;
    onDaysChange: (days: number) => void;
  };
  let { graph, graphDays, onDaysChange }: Props = $props();

  type StatRange = "one_month" | "three_months" | "one_year";
  let separateInactive = $state(true);
  let hoursRange = $state<StatRange>("one_month");
  let buttonsRange = $state<StatRange>("one_month");

  const dayRangeOptions = $derived([
    { value: 31, label: t("decks.range1m") },
    { value: 92, label: t("decks.range3m") },
    { value: 365, label: t("decks.range1y") },
  ]);
  const statRangeOptions = $derived([
    { value: "one_month", label: t("decks.range1m") },
    { value: "three_months", label: t("decks.range3m") },
    { value: "one_year", label: t("decks.range1y") },
  ] as { value: StatRange; label: string }[]);

  const futureDueChartBuckets = $derived(
    graph.future_due.map((b) => ({ day: b.key, count: b.value })),
  );
  const reviewsCols = $derived(buildReviewsCols(graph.reviews));
  const calendarPerDay = $derived(buildCalendarPerDay(graph.reviews));
  const addedCols = $derived(buildAddedCols(graph.added));
</script>

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

{#snippet todayBlock()}
  {#if graph.today.answer_count === 0}
    <p class="mt-2 text-sm text-(--color-fg-muted)">{t("decks.todayEmpty")}</p>
  {:else}
    <p class="mt-2 text-sm text-(--color-fg-default)">
      {t("decks.todayCount", {
        count: graph.today.answer_count,
        minutes: formatDuration(graph.today.answer_millis),
      })}
    </p>
  {/if}
{/snippet}

{#snippet futureDueRange()}
  <RangeTabs options={dayRangeOptions} value={graphDays} onSelect={onDaysChange} />
{/snippet}

{#snippet futureDueBlock()}
  <p class="mt-1 text-xs text-(--color-fg-subtle)">
    {t("decks.futureDueDesc", { days: graphDays })}
  </p>
  <div class="mt-2 text-(--color-fg-muted)">
    <FutureDueChart buckets={futureDueChartBuckets} days={graphDays} />
  </div>
  <div class="mt-1 flex justify-between text-[11px] text-(--color-fg-subtle) tabular-nums">
    <span>{t("decks.futureDueTotal", { count: graph.future_due_total })}</span>
    <span>{t("decks.futureDueAvg", { avg: graph.future_due_avg_per_day.toFixed(1) })}</span>
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
      counts={separateInactive ? graph.card_counts_separate : graph.card_counts_combined}
    />
  </div>
{/snippet}

{#snippet intervalsBlock()}
  <p class="mt-1 text-xs text-(--color-fg-subtle)">{t("decks.intervalsDesc")}</p>
  <div class="mt-2 text-(--color-fg-muted)">
    <HistogramChart
      buckets={graph.intervals}
      xFormat={(v) => `${v}d`}
      color="var(--color-accent-500)"
    />
  </div>
{/snippet}

{#snippet easeBlock()}
  <p class="mt-1 text-xs text-(--color-fg-subtle)">{t("decks.cardEaseDesc")}</p>
  <div class="mt-2 text-(--color-fg-muted)">
    <HistogramChart
      buckets={graph.eases.map((b) => ({ key: Math.round(b.key / 10), value: b.value }))}
      xFormat={(v) => `${v}%`}
      color="var(--color-success)"
    />
  </div>
  {#if graph.eases_average > 0}
    <p class="mt-1 text-center text-[11px] text-(--color-fg-subtle)">
      Median ease: {(graph.eases_average / 10).toFixed(0)}%
    </p>
  {/if}
{/snippet}

{#snippet retentionBlock()}
  <p class="mt-1 text-xs text-(--color-fg-subtle)">{t("decks.retentionDesc")}</p>
  <div class="mt-3">
    <RetentionTable {...graph.retention} />
  </div>
{/snippet}

{#snippet hoursRangeBtns()}
  <RangeTabs options={statRangeOptions} value={hoursRange} onSelect={(v) => (hoursRange = v)} />
{/snippet}

{#snippet hoursBlock()}
  <p class="mt-1 text-xs text-(--color-fg-subtle)">{t("decks.hourlyDesc")}</p>
  <div class="mt-2 text-(--color-fg-muted)">
    <HoursChart hours={graph.hours[hoursRange]} />
  </div>
{/snippet}

{#snippet buttonsRangeBtns()}
  <RangeTabs options={statRangeOptions} value={buttonsRange} onSelect={(v) => (buttonsRange = v)} />
{/snippet}

{#snippet buttonsBlock()}
  <p class="mt-1 text-xs text-(--color-fg-subtle)">{t("decks.answerButtonsDesc")}</p>
  <div class="mt-2 text-(--color-fg-muted)">
    <ButtonsChart counts={graph.buttons[buttonsRange]} />
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
