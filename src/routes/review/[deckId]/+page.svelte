<script lang="ts">
  import { ArrowLeft, RotateCcw, BookOpen } from "lucide-svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { invoke } from "$lib/ipc";
  import CardFrame from "$lib/components/CardFrame.svelte";
  import { onMount, onDestroy } from "svelte";
  import { fade, scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { t } from "$lib/i18n/index.svelte";
  import { shortcuts } from "$lib/stores/shortcuts.svelte";

  type Counts = { new: number; learning: number; review: number };
  type StudyCard = {
    card_id: number;
    question_html: string;
    answer_html: string;
    css: string;
    remaining: Counts;
  };
  type NextCard =
    | { kind: "card"; card_id: number; question_html: string; answer_html: string; css: string; remaining: Counts }
    | { kind: "done"; new: number; learning: number; review: number };

  const deckId = $derived(Number($page.params.deckId));

  let current = $state<StudyCard | null>(null);
  let totals = $state<Counts>({ new: 0, learning: 0, review: 0 });
  let initialTotal = $state(0);
  let cursor = $state(0);
  let showingAnswer = $state(false);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let shownAt = $state<number>(0);

  const totalDue = $derived(totals.new + totals.learning + totals.review);
  const progress = $derived(
    initialTotal > 0 ? (cursor / initialTotal) * 100 : 0,
  );

  onMount(async () => {
    window.addEventListener("keydown", onKey);
    await startSession();
  });

  onDestroy(() => {
    window.removeEventListener("keydown", onKey);
  });

  async function startSession() {
    loading = true;
    error = null;
    cursor = 0;
    try {
      await invoke("start_study", { deckId });
      await loadNext(/* isFirst */ true);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function loadNext(isFirst = false) {
    showingAnswer = false;
    try {
      const r = await invoke<NextCard>("get_next_card");
      if (r.kind === "card") {
        current = {
          card_id: r.card_id,
          question_html: r.question_html,
          answer_html: r.answer_html,
          css: r.css,
          remaining: r.remaining,
        };
        totals = r.remaining;
        if (isFirst) initialTotal = totals.new + totals.learning + totals.review + 1;
        shownAt = performance.now();
      } else {
        current = null;
        totals = { new: r.new, learning: r.learning, review: r.review };
      }
    } catch (e) {
      error = String(e);
    }
  }

  function flip() {
    showingAnswer = true;
  }

  async function answer(rating: "again" | "hard" | "good" | "easy") {
    if (!current) return;
    const ms = Math.min(60_000, Math.round(performance.now() - shownAt));
    try {
      await invoke("answer_card_now", {
        rating,
        millisecondsTaken: ms,
      });
      cursor += 1;
      await loadNext();
    } catch (e) {
      error = String(e);
    }
  }

  let naniInput = $state<HTMLInputElement | null>(null);
  let naniBusy = $state(false);
  let naniError = $state<string | null>(null);

  function frontWord(): string {
    if (!current) return "";
    // DOMParser parses without attaching the result to the live document,
    // so inline event handlers (e.g. <img onerror="…">) inside the card
    // template never fire — safer than `div.innerHTML = …`.
    const doc = new DOMParser().parseFromString(current.question_html, "text/html");
    return (doc.body.textContent ?? "").trim().replace(/\s+/g, " ");
  }

  async function naniLookup() {
    if (naniBusy) return;
    const word = frontWord();
    if (!word || !naniInput) return;
    naniBusy = true;
    naniError = null;
    // A real <input> is exposed to macOS Accessibility / Services as a
    // text container with a live selection — that's what Nani reads when
    // its global Cmd+J fires. Selecting in the iframe / arbitrary spans
    // does not reliably surface to the OS, so we route through this input.
    // Do NOT add `aria-hidden` to the input element — that would remove
    // it from the AX tree and Nani would no longer see the selection.
    naniInput.value = word;
    naniInput.focus();
    naniInput.setSelectionRange(0, word.length);
    try {
      await invoke("nani_lookup", { word });
    } catch (e) {
      console.error("nani_lookup failed", e);
      naniError = e instanceof Error ? e.message : String(e);
    } finally {
      // Nani has already read the selection by now (osascript Cmd+J ran
      // synchronously inside the await). Blur so subsequent rating keys
      // (1/2/3/4) don't get swallowed by the input element — onKey skips
      // events whose target is an <input>.
      naniInput.blur();
      naniBusy = false;
    }
  }

  function onKey(e: KeyboardEvent) {
    // Auto-repeat would re-fire naniLookup mid-flight, or trigger a
    // rating right after Nani returns when the user is still holding a key.
    if (e.repeat) return;
    // While Nani is in flight (osascript / Cmd+J / Nani app focus switch)
    // any rating key would race with naniLookup and progress the deck
    // before the user even sees the lookup result.
    if (naniBusy) return;
    if (e.target instanceof HTMLInputElement) return;
    if (!showingAnswer && (e.key === " " || e.key === "Enter")) {
      e.preventDefault();
      flip();
      return;
    }
    if (showingAnswer) {
      if (shortcuts.isNani(e.key)) {
        e.preventDefault();
        void naniLookup();
        return;
      }
      const rating = shortcuts.ratingFor(e.key);
      if (rating) {
        e.preventDefault();
        void answer(rating);
      }
    }
  }

  type Tone = "danger" | "warning" | "accent" | "success";
  type Rating = "again" | "hard" | "good" | "easy";
  const buttons = $derived<
    { rating: Rating; label: string; tone: Tone }[]
  >([
    { rating: "again", label: t("reviewer.again"), tone: "danger" },
    { rating: "hard", label: t("reviewer.hard"), tone: "warning" },
    { rating: "good", label: t("reviewer.good"), tone: "accent" },
    { rating: "easy", label: t("reviewer.easy"), tone: "success" },
  ]);

  const toneBg: Record<Tone, string> = {
    accent:
      "bg-(--color-accent-500) text-(--color-fg-onAccent) hover:bg-(--color-accent-600)",
    success:
      "bg-(--color-success) text-(--color-fg-onAccent) hover:brightness-110",
    warning:
      "bg-(--color-warning) text-(--color-bg-base) hover:brightness-105",
    danger:
      "bg-(--color-danger) text-(--color-fg-onAccent) hover:brightness-110",
  };
</script>

<div
  class="relative flex h-full flex-col transition-colors duration-300"
  class:bg-answer={showingAnswer}
>
  <div class="flex items-center justify-between px-6 py-3">
    <button
      type="button"
      onclick={() => goto("/")}
      class="flex items-center gap-1.5 rounded-(--radius-md) px-2 py-1 text-sm text-(--color-fg-muted) transition-colors hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)"
    >
      <ArrowLeft size={14} />
      {t("reviewer.back")}
    </button>
    <p class="flex items-center gap-6 text-xs text-(--color-fg-subtle)">
      <span class="number-tabular">
        {cursor + (current ? 1 : 0)} / {initialTotal || totalDue || "—"}
      </span>
      <span class="hidden items-center gap-1.5 sm:flex">
        <span
          class="inline-flex items-center gap-1.5 rounded-full border border-(--color-border-default) bg-(--color-bg-elevated) px-2 py-0.5"
        >
          <span class="text-[10px] font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
            {t("decks.new")}
          </span>
          <span class="number-tabular text-xs font-medium text-(--color-fg-default)">{totals.new}</span>
        </span>
        <span
          class="inline-flex items-center gap-1.5 rounded-full border border-(--color-border-default) bg-(--color-bg-elevated) px-2 py-0.5"
        >
          <span class="text-[10px] font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
            {t("decks.learning")}
          </span>
          <span class="number-tabular text-xs font-medium text-(--color-fg-default)">{totals.learning}</span>
        </span>
        <span
          class="inline-flex items-center gap-1.5 rounded-full border border-(--color-border-default) bg-(--color-bg-elevated) px-2 py-0.5"
        >
          <span class="text-[10px] font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
            {t("decks.review")}
          </span>
          <span class="number-tabular text-xs font-medium text-(--color-fg-default)">{totals.review}</span>
        </span>
      </span>
    </p>
    <button
      type="button"
      onclick={startSession}
      class="grid h-7 w-7 place-items-center rounded-(--radius-md) text-(--color-fg-muted) transition-colors hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)"
      aria-label={t("reviewer.reload")}
    >
      <RotateCcw size={14} />
    </button>
  </div>

  <div
    class="bg-(--color-bg-overlay) transition-[height] duration-300 {showingAnswer
      ? 'h-1'
      : 'h-0.5'}"
  >
    <div
      class="h-full transition-all duration-500 {showingAnswer
        ? 'bg-(--color-success)'
        : 'bg-(--color-accent-500)'}"
      style="width: {progress}%; transition-timing-function: var(--ease-out-expo);"
    ></div>
  </div>

  <div class="flex flex-1 flex-col items-center px-6 pt-12 pb-6">
    {#if loading}
      <p class="text-sm text-(--color-fg-muted)">{t("reviewer.loading")}</p>
    {:else if error}
      <p class="text-sm text-(--color-danger)">{error}</p>
    {:else if !current}
      <div in:scale={{ duration: 240, start: 0.92, easing: cubicOut }} class="text-center">
        <p class="font-display text-3xl font-medium">{t("reviewer.done")}</p>
        <p class="mt-2 text-sm text-(--color-fg-muted)">{t("reviewer.sessionFinished")}</p>
        <button
          type="button"
          onclick={() => goto("/")}
          class="mt-6 rounded-(--radius-md) border border-(--color-border-strong) px-4 py-1.5 text-sm transition-colors hover:bg-(--color-bg-overlay)"
        >
          {t("reviewer.backToDecks")}
        </button>
      </div>
    {:else}
      <div class="relative w-full">
        <div
          class="absolute -top-3.5 left-1/2 z-10 flex -translate-x-1/2 items-center gap-1.5 rounded-full px-3 py-1 text-[10px] font-semibold tracking-[0.18em] uppercase shadow-(--shadow-subtle) transition-all duration-200 {showingAnswer
            ? 'bg-(--color-success) text-(--color-fg-onAccent) ring-1 ring-(--color-success)/30 ring-offset-2 ring-offset-(--color-bg-base)'
            : 'bg-(--color-bg-overlay) text-(--color-fg-muted)'}"
        >
          <span
            class="h-1.5 w-1.5 rounded-full {showingAnswer
              ? 'bg-(--color-fg-onAccent)/80'
              : 'bg-(--color-accent-500)'}"
          ></span>
          {showingAnswer ? t("reviewer.answer") : t("reviewer.question")}
        </div>
        {#key current.card_id}
          <article
            in:fade={{ duration: 220, easing: cubicOut, delay: 60 }}
            style="height: 420px; min-height: 420px; max-height: 420px;"
            class="block w-full shrink-0 overflow-hidden rounded-(--radius-xl) border bg-(--color-bg-elevated) shadow-(--shadow-card) transition-[border-color,box-shadow] duration-200 {showingAnswer
              ? 'border-(--color-success)/50 shadow-(--shadow-glow)'
              : 'border-(--color-border-default)'}"
          >
            <CardFrame
              html={showingAnswer ? current.answer_html : current.question_html}
              css={current.css}
              side={showingAnswer ? "answer" : "question"}
            />
          </article>
        {/key}
      </div>

      <div
        class="mt-8 flex h-16 w-full shrink-0 items-center justify-center gap-3"
      >
        {#if !showingAnswer}
          <button
            type="button"
            onclick={flip}
            in:fade={{ duration: 160, easing: cubicOut }}
            class="rounded-(--radius-md) bg-(--color-accent-500) px-8 py-2.5 text-sm font-medium text-(--color-fg-onAccent) shadow-(--shadow-card) transition-all hover:bg-(--color-accent-600) hover:shadow-(--shadow-glow) active:scale-[0.97]"
          >
            {t("reviewer.showAnswer")}
            <span class="ml-2 font-mono text-[10px] opacity-70">Space</span>
          </button>
        {:else}
          <button
            type="button"
            onclick={naniLookup}
            in:fade={{ duration: 200, easing: cubicOut }}
            class="flex min-w-[88px] flex-col items-center gap-0.5 rounded-(--radius-md) border border-(--color-border-strong) bg-(--color-bg-elevated) px-5 py-2.5 text-(--color-fg-default) shadow-(--shadow-card) transition-all hover:-translate-y-0.5 hover:bg-(--color-bg-overlay) hover:shadow-(--shadow-glow) active:translate-y-0 active:scale-[0.97]"
            title="Nani Search"
          >
            <span class="flex items-center gap-1.5 text-sm font-medium">
              <BookOpen size={14} strokeWidth={2.25} />
              Nani
            </span>
            <span class="font-mono text-[10px] opacity-70">{shortcuts.label("nani")}</span>
          </button>
          {#each buttons as b, i (b.rating)}
            <button
              type="button"
              onclick={() => answer(b.rating)}
              in:fade={{ duration: 200, delay: 40 + i * 30, easing: cubicOut }}
              class="flex min-w-[88px] flex-col items-center gap-0.5 rounded-(--radius-md) px-5 py-2.5 shadow-(--shadow-card) transition-all hover:-translate-y-0.5 hover:shadow-(--shadow-glow) active:translate-y-0 active:scale-[0.97] {toneBg[b.tone]}"
            >
              <span class="text-sm font-medium">{b.label}</span>
              <span class="font-mono text-[10px] opacity-70">{shortcuts.label(b.rating)}</span>
            </button>
          {/each}
        {/if}
      </div>
    {/if}
  </div>
  {#if naniError}
    <div
      role="alert"
      class="pointer-events-auto fixed bottom-6 left-1/2 z-20 max-w-md -translate-x-1/2 rounded-(--radius-md) border border-(--color-danger)/40 bg-(--color-danger)/10 px-4 py-2 text-xs text-(--color-danger) shadow-(--shadow-card)"
    >
      <p class="font-medium">Nani lookup failed</p>
      <p class="mt-0.5 break-all opacity-80">{naniError}</p>
      <p class="mt-1 text-[10px] opacity-70">
        macOS &gt; Privacy &amp; Security &gt; Accessibility で memorize に権限を付与してください。
      </p>
    </div>
  {/if}

  <input
    bind:this={naniInput}
    type="text"
    readonly
    tabindex="-1"
    class="pointer-events-none fixed top-0 left-[-10000px] h-4 w-px"
  />
</div>
