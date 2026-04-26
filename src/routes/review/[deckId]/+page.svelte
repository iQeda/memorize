<script lang="ts">
  import { ArrowLeft, RotateCcw } from "lucide-svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { invoke } from "$lib/ipc";
  import CardFrame from "$lib/components/CardFrame.svelte";
  import { onMount, onDestroy } from "svelte";
  import { fade, scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  type CardSummary = {
    id: number;
    note_id: number;
    deck_id: number;
    template_idx: number;
  };
  type StudyQueue = {
    cards: CardSummary[];
    new_count: number;
    learning_count: number;
    review_count: number;
  };
  type Rendered = { question_html: string; answer_html: string; css: string };

  const deckId = $derived(Number($page.params.deckId));

  let queue = $state<CardSummary[]>([]);
  let cursor = $state(0);
  let totals = $state({ new_count: 0, learning_count: 0, review_count: 0 });
  let rendered = $state<Rendered | null>(null);
  let showingAnswer = $state(false);
  let loading = $state(false);
  let error = $state<string | null>(null);

  const current = $derived(queue[cursor] ?? null);
  const remaining = $derived(Math.max(0, queue.length - cursor));
  const totalDue = $derived(
    totals.new_count + totals.learning_count + totals.review_count,
  );
  const progress = $derived(totalDue > 0 ? (cursor / totalDue) * 100 : 0);

  onMount(async () => {
    window.addEventListener("keydown", onKey);
    await loadQueue();
  });

  onDestroy(() => {
    window.removeEventListener("keydown", onKey);
  });

  async function loadQueue() {
    loading = true;
    error = null;
    try {
      const q = await invoke<StudyQueue>("get_study_queue", {
        deckId,
        limit: 200,
      });
      queue = q.cards;
      totals = {
        new_count: q.new_count,
        learning_count: q.learning_count,
        review_count: q.review_count,
      };
      cursor = 0;
      await loadCurrent();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function loadCurrent() {
    showingAnswer = false;
    if (!current) {
      rendered = null;
      return;
    }
    try {
      rendered = await invoke<Rendered>("get_card_render", {
        cardId: current.id,
      });
    } catch (e) {
      error = String(e);
    }
  }

  function flip() {
    showingAnswer = true;
  }

  async function answer(_grade: 1 | 2 | 3 | 4) {
    // Phase 1 では実際の解答送信は行わず、次のカードへ進むだけ
    cursor = Math.min(cursor + 1, queue.length);
    await loadCurrent();
  }

  function onKey(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement) return;
    if (!showingAnswer && (e.key === " " || e.key === "Enter")) {
      e.preventDefault();
      flip();
      return;
    }
    if (showingAnswer) {
      const grade = ({ "1": 1, "2": 2, "3": 3, "4": 4, " ": 3 } as const)[
        e.key
      ];
      if (grade) {
        e.preventDefault();
        void answer(grade as 1 | 2 | 3 | 4);
      }
    }
  }

  type Tone = "danger" | "warning" | "accent" | "success";
  const buttons: { grade: 1 | 2 | 3 | 4; label: string; tone: Tone }[] = [
    { grade: 1, label: "Again", tone: "danger" },
    { grade: 2, label: "Hard", tone: "warning" },
    { grade: 3, label: "Good", tone: "accent" },
    { grade: 4, label: "Easy", tone: "success" },
  ];

  // Background tint applied to the whole reviewer when answer is showing.
  // Defined inline so HMR picks it up immediately.
  // (Tailwind v4 utility classes don't compose dynamic colors well, so we
  // use a CSS class declared in app.css.)
  const _toneBgPlaceholder: undefined = undefined;

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
      Back
    </button>
    <p class="flex items-center gap-6 text-xs text-(--color-fg-subtle)">
      <span class="number-tabular">
        {Math.min(cursor + 1, Math.max(totalDue, 1))} / {totalDue || "—"}
      </span>
      <span class="hidden items-center gap-3 sm:flex">
        <span class="flex items-center gap-1.5">
          <span class="h-1.5 w-1.5 rounded-full bg-(--color-accent-500)"></span>
          <span class="number-tabular">{totals.new_count}</span>
        </span>
        <span class="flex items-center gap-1.5">
          <span class="h-1.5 w-1.5 rounded-full bg-(--color-warning)"></span>
          <span class="number-tabular">{totals.learning_count}</span>
        </span>
        <span class="flex items-center gap-1.5">
          <span class="h-1.5 w-1.5 rounded-full bg-(--color-success)"></span>
          <span class="number-tabular">{totals.review_count}</span>
        </span>
      </span>
    </p>
    <button
      type="button"
      onclick={loadQueue}
      class="grid h-7 w-7 place-items-center rounded-(--radius-md) text-(--color-fg-muted) transition-colors hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)"
      aria-label="Reload"
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
      <p class="text-sm text-(--color-fg-muted)">読み込み中…</p>
    {:else if error}
      <p class="text-sm text-(--color-danger)">{error}</p>
    {:else if !current}
      <div in:scale={{ duration: 240, start: 0.92, easing: cubicOut }} class="text-center">
        <p class="font-display text-3xl font-medium">完了 ✦</p>
        <p class="mt-2 text-sm text-(--color-fg-muted)">
          このセッションのカードは終わりました
        </p>
        <button
          type="button"
          onclick={() => goto("/")}
          class="mt-6 rounded-(--radius-md) border border-(--color-border-strong) px-4 py-1.5 text-sm transition-colors hover:bg-(--color-bg-overlay)"
        >
          デッキ一覧へ
        </button>
      </div>
    {:else}
      <div class="relative w-full max-w-[720px]">
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
          {showingAnswer ? "解答" : "問題"}
        </div>
        {#key current.id}
          <article
            in:fade={{ duration: 220, easing: cubicOut, delay: 60 }}
            style="height: 420px; min-height: 420px; max-height: 420px;"
            class="block w-full shrink-0 overflow-hidden rounded-(--radius-xl) border bg-(--color-bg-elevated) shadow-(--shadow-card) transition-[border-color,box-shadow] duration-200 {showingAnswer
              ? 'border-(--color-success)/50 shadow-(--shadow-glow)'
              : 'border-(--color-border-default)'}"
          >
            {#if rendered}
              <CardFrame
                html={showingAnswer ? rendered.answer_html : rendered.question_html}
                css={rendered.css}
                side={showingAnswer ? "answer" : "question"}
              />
            {/if}
          </article>
        {/key}
      </div>

      <div
        class="mt-8 flex h-16 w-full max-w-[720px] shrink-0 items-center justify-center gap-3"
      >
        {#if !showingAnswer}
          <button
            type="button"
            onclick={flip}
            in:fade={{ duration: 160, easing: cubicOut }}
            class="rounded-(--radius-md) bg-(--color-accent-500) px-8 py-2.5 text-sm font-medium text-(--color-fg-onAccent) shadow-(--shadow-card) transition-all hover:bg-(--color-accent-600) hover:shadow-(--shadow-glow) active:scale-[0.97]"
          >
            解答を表示
            <span class="ml-2 font-mono text-[10px] opacity-70">Space</span>
          </button>
        {:else}
          {#each buttons as b, i (b.grade)}
            <button
              type="button"
              onclick={() => answer(b.grade)}
              in:fade={{ duration: 200, delay: 40 + i * 30, easing: cubicOut }}
              class="flex min-w-[88px] flex-col items-center gap-0.5 rounded-(--radius-md) px-5 py-2.5 shadow-(--shadow-card) transition-all hover:-translate-y-0.5 hover:shadow-(--shadow-glow) active:translate-y-0 active:scale-[0.97] {toneBg[b.tone]}"
            >
              <span class="text-sm font-medium">{b.label}</span>
              <span class="font-mono text-[10px] opacity-70">{b.grade}</span>
            </button>
          {/each}
        {/if}
      </div>
    {/if}
  </div>
</div>
