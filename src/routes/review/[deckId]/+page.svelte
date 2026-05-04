<script lang="ts">
  import { ArrowLeft, RotateCcw, Eye, Pencil, Copy, X } from "lucide-svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { invoke } from "$lib/ipc";
  import CardFrame from "$lib/components/CardFrame.svelte";
  import NoteEditor from "$lib/components/NoteEditor.svelte";
  import { onMount } from "svelte";
  import { fade, scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { t } from "$lib/i18n/index.svelte";
  import { shortcuts } from "$lib/stores/shortcuts.svelte";
  import { sync } from "$lib/stores/sync.svelte";
  import { collection } from "$lib/stores/collection.svelte";

  type Counts = { new: number; learning: number; review: number };
  type StudyCard = {
    card_id: number;
    note_id: number;
    question_html: string;
    answer_html: string;
    css: string;
    remaining: Counts;
  };
  type NextCard =
    | {
        kind: "card";
        card_id: number;
        note_id: number;
        question_html: string;
        answer_html: string;
        css: string;
        remaining: Counts;
      }
    | { kind: "done"; new: number; learning: number; review: number };
  type RenderedCard = {
    question_html: string;
    answer_html: string;
    css: string;
  };

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
    await startSession();
  });

  // `totals` は `get_next_card` の `remaining` でしか更新されないため、
  // 解答中に Sync が完了して別端末からの変更が入っても、次の rating まで
  // バッジが古いままになる。Sync 完了 (busy: true → false) を検知して
  // `list_decks` の最新値で塗り直す。`get_next_card` の remaining は
  // 「現在カードを除く残り」なのに対し `list_decks` は全件を返すので、
  // 現在カードの種類に応じて 1 引いて差を吸収する。
  let prevSyncBusy = false;
  $effect(() => {
    const isBusy = sync.busy;
    if (prevSyncBusy && !isBusy && current) {
      void refreshTotalsAfterSync();
    }
    prevSyncBusy = isBusy;
  });

  async function refreshTotalsAfterSync() {
    await collection.refreshDecks();
    const d = collection.decks.find((x) => x.id === deckId);
    if (!d) return;
    const next = { new: d.new_count, learning: d.learn_count, review: d.review_count };
    // 現在カードの 1 枚分は remaining から除外したいが、その種類は
    // フロントエンドに渡っていない。totals の前回値との差分で推定する：
    // 前回 totals に存在し、かつ今回の next で 0 でないカテゴリを 1 減算。
    const adjusted = { ...next };
    if (totals.new > 0 && adjusted.new > 0) adjusted.new -= 1;
    else if (totals.learning > 0 && adjusted.learning > 0) adjusted.learning -= 1;
    else if (totals.review > 0 && adjusted.review > 0) adjusted.review -= 1;
    totals = adjusted;
    // ヘッダーの "X / Y" 分母も Sync で増減した分だけ追従させる。
    // cursor は既に答えた枚数、+1 は現在カード、残り 3 カテゴリの合計が今後分。
    initialTotal = cursor + adjusted.new + adjusted.learning + adjusted.review + 1;
  }

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
          note_id: r.note_id,
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

  let editing = $state(false);

  function openEditor() {
    if (!current) return;
    editing = true;
  }

  async function reloadCurrent() {
    if (!current) return;
    try {
      const r = await invoke<RenderedCard>("get_card_render", {
        cardId: current.card_id,
      });
      current = {
        ...current,
        question_html: r.question_html,
        answer_html: r.answer_html,
        css: r.css,
      };
    } catch (e) {
      console.error("get_card_render failed", e);
    }
  }

  function flip() {
    showingAnswer = !showingAnswer;
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
      // サイドバーのデッキバッジは collection.decks を参照している。get_next_card
      // の remaining はヘッダー用に header の totals だけ更新するが、collection
      // ストアには反映されないので、答えるたびに list_decks を fire-and-forget で
      // 投げて全デッキのカウントを塗り直す。次カード描画はブロックしない。
      void collection.refreshDecks();
    } catch (e) {
      error = String(e);
    }
  }

  let questionFrame = $state<HTMLIFrameElement | undefined>();
  let answerFrame = $state<HTMLIFrameElement | undefined>();
  let copyError = $state<string | null>(null);
  let copyInfo = $state<string | null>(null);
  let copyInfoTimer: ReturnType<typeof setTimeout> | null = null;

  // Anki テンプレートの answer 側は通常 `{{FrontSide}}<hr id=answer>{{Back}}` 構造で、
  // answer_html に質問部分が含まれる。フリップ後は答えだけ見せたいので hr#answer を
  // 境に前半 (質問) を削除した HTML を返す。hr#answer が無いカスタムテンプレートは
  // そのまま返す（破壊しない）。
  const answerOnlyHtml = $derived.by(() => {
    if (!current) return "";
    const html = current.answer_html;
    const doc = new DOMParser().parseFromString(html, "text/html");
    const hr = doc.querySelector('hr#answer, hr[id="answer"]');
    if (!hr || !hr.parentElement) return html;
    const parent = hr.parentElement;
    while (parent.firstChild && parent.firstChild !== hr) {
      parent.removeChild(parent.firstChild);
    }
    parent.removeChild(hr);
    return doc.body.innerHTML;
  });

  function flashInfo(msg: string) {
    copyInfo = msg;
    if (copyInfoTimer) clearTimeout(copyInfoTimer);
    copyInfoTimer = setTimeout(() => {
      copyInfo = null;
      copyInfoTimer = null;
    }, 4500);
  }

  async function copyErrorMessage() {
    if (!copyError) return;
    try {
      await navigator.clipboard.writeText(copyError);
      flashInfo("Copied");
    } catch (e) {
      console.error("clipboard copy failed", e);
    }
  }

  async function copyCardText() {
    const frame = showingAnswer ? answerFrame : questionFrame;
    const win = frame?.contentWindow;
    const doc = frame?.contentDocument;
    if (!win || !doc) return;
    const host = doc.querySelector(".memorize-card-host");
    if (!host) return;

    // iframe 内の本文 div を全範囲選択。removeAllRanges → addRange で
    // 既存の選択を上書きするだけで、その後の操作で解除はしない。
    const range = doc.createRange();
    range.selectNodeContents(host);
    const sel = win.getSelection();
    sel?.removeAllRanges();
    sel?.addRange(range);

    // iframe にフォーカスを移し selection を AX 的に「active」にする。
    // ユーザーが手動で Cmd+J を押した際に Nani.app が現在選択中のテキストを
    // 読み取れるのは、フォーカスが当たっている要素の selection だけなので
    // 必須。後続の c/1/2/3/4/Space などは srcdoc 内の key bridge が
    // parent window に再ディスパッチするため引き続き反応する。
    win.focus();

    const text = (host.textContent ?? "").trim().replace(/\s+/g, " ");
    if (!text) return;

    copyInfo = null;
    try {
      await navigator.clipboard.writeText(text);
      copyError = null;
      flashInfo("Copied");
    } catch (e) {
      console.error("clipboard write failed", e);
      copyError = e instanceof Error ? e.message : String(e);
    }
  }

  function onKey(e: KeyboardEvent) {
    // Auto-repeat would re-fire copyCardText mid-flight, or apply a rating
    // while the user is still holding a key from a previous action.
    if (e.repeat) return;
    // The editor mounts its own inputs/textarea; let it handle its own keys.
    if (editing) return;
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    // 修飾子付きのキーは OS / WebView の標準ショートカットに譲る。
    // 特に Cmd+J は Nani.app のグローバルホットキー — Copy が "j" にバインド
    // されている状態で preventDefault すると Nani に届かなくなる。同様に
    // Cmd+A/Cmd+S/Cmd+F なども rating キー (a/s/d/f) と衝突するため、修飾子
    // 付きは review 操作をすべて見送って素通しする。
    const hasModifier = e.metaKey || e.ctrlKey || e.altKey;
    // Esc — leave the review session and return to the deck overview.
    if (e.key === "Escape" && !hasModifier) {
      e.preventDefault();
      void goto("/");
      return;
    }
    if (hasModifier) return;
    // セッション終了画面 (current === null = "All cards reviewed") では
    // フリップやレーティング対象がないので、Enter は "Back to decks" として
    // ホームへ戻すだけにする (Esc と同等の動線)。それ以外のキーは無視。
    if (!current) {
      if (e.key === "Enter") {
        e.preventDefault();
        void goto("/");
      }
      return;
    }
    // `e` (no modifiers) — open the note editor for the current card.
    // Available regardless of question/answer side; mirrors Anki's E shortcut.
    if ((e.key === "e" || e.key === "E") && current) {
      e.preventDefault();
      openEditor();
      return;
    }
    // Copy works on either side — sometimes you need to look up a word
    // before flipping to the answer.
    if (shortcuts.isCopy(e.key)) {
      e.preventDefault();
      void copyCardText();
      return;
    }
    if (e.key === " " || e.key === "Enter") {
      e.preventDefault();
      flip();
      return;
    }
    if (showingAnswer) {
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
      <span class="ml-1 font-mono text-[10px] opacity-70">Esc</span>
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
    <div class="flex items-center gap-1">
      {#if current}
        <button
          type="button"
          onclick={openEditor}
          class="grid h-7 w-7 place-items-center rounded-(--radius-md) text-(--color-fg-muted) transition-colors hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)"
          aria-label={t("settings.shortcut.editNote")}
          title="{t('settings.shortcut.editNote')} (E)"
        >
          <Pencil size={14} />
        </button>
      {/if}
      <button
        type="button"
        onclick={startSession}
        class="grid h-7 w-7 place-items-center rounded-(--radius-md) text-(--color-fg-muted) transition-colors hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)"
        aria-label={t("reviewer.reload")}
      >
        <RotateCcw size={14} />
      </button>
    </div>
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
          class="mt-6 inline-flex items-center gap-2 rounded-(--radius-md) border border-(--color-border-strong) px-4 py-1.5 text-sm transition-colors hover:bg-(--color-bg-overlay)"
        >
          {t("reviewer.backToDecks")}
          <span class="font-mono text-[10px] opacity-70">↵</span>
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
            style="height: 420px; min-height: 420px; max-height: 420px; perspective: 2000px;"
            class="block w-full shrink-0"
          >
            <div
              class="relative h-full w-full transition-transform duration-500 ease-out"
              style="transform-style: preserve-3d; transform: rotateY({showingAnswer ? 180 : 0}deg);"
            >
              <div
                style="backface-visibility: hidden; -webkit-backface-visibility: hidden;"
                class="absolute inset-0 overflow-hidden rounded-(--radius-xl) border bg-(--color-bg-elevated) shadow-(--shadow-card) transition-[border-color,box-shadow] duration-200 {showingAnswer
                  ? 'border-(--color-success)/50 shadow-(--shadow-glow)'
                  : 'border-(--color-border-default)'}"
              >
                <CardFrame
                  bind:iframeEl={questionFrame}
                  html={current.question_html}
                  css={current.css}
                  side="question"
                />
              </div>
              <div
                style="backface-visibility: hidden; -webkit-backface-visibility: hidden; transform: rotateY(180deg);"
                class="absolute inset-0 overflow-hidden rounded-(--radius-xl) border bg-(--color-bg-elevated) shadow-(--shadow-card) transition-[border-color,box-shadow] duration-200 {showingAnswer
                  ? 'border-(--color-success)/50 shadow-(--shadow-glow)'
                  : 'border-(--color-border-default)'}"
              >
                <CardFrame
                  bind:iframeEl={answerFrame}
                  html={answerOnlyHtml}
                  css={current.css}
                  side="answer"
                />
              </div>
            </div>
          </article>
        {/key}
      </div>

      <div
        class="mt-8 flex w-full shrink-0 flex-col items-center gap-3"
      >
        {#if !showingAnswer}
          <div class="flex items-center justify-center gap-3">
            <button
              type="button"
              onclick={copyCardText}
              in:fade={{ duration: 160, easing: cubicOut }}
              class="flex h-16 w-32 flex-col items-center justify-center gap-0.5 rounded-(--radius-md) border border-(--color-border-strong) bg-(--color-bg-elevated) px-5 py-2.5 text-(--color-fg-default) shadow-(--shadow-card) transition-all hover:-translate-y-0.5 hover:bg-(--color-bg-overlay) hover:shadow-(--shadow-glow) active:translate-y-0 active:scale-[0.97]"
              title="Copy"
            >
              <span class="flex items-center gap-1.5 text-sm font-medium">
                <Copy size={14} strokeWidth={2.25} />
                Copy
              </span>
              <span class="font-mono text-[10px] opacity-70">{shortcuts.label("copy")}</span>
            </button>
            <button
              type="button"
              onclick={flip}
              in:fade={{ duration: 160, easing: cubicOut }}
              class="flex h-16 w-32 flex-col items-center justify-center gap-0.5 rounded-(--radius-md) border border-(--color-border-strong) bg-(--color-bg-elevated) px-5 py-2.5 text-(--color-fg-default) shadow-(--shadow-card) transition-all hover:-translate-y-0.5 hover:bg-(--color-bg-overlay) hover:shadow-(--shadow-glow) active:translate-y-0 active:scale-[0.97]"
            >
              <span class="flex items-center gap-1.5 text-sm font-medium">
                <Eye size={14} strokeWidth={2.25} />
                {t("reviewer.showAnswer")}
              </span>
              <span class="font-mono text-[10px] opacity-70">Space</span>
            </button>
          </div>
        {:else}
          <div class="flex items-center justify-center gap-3">
            <button
              type="button"
              onclick={copyCardText}
              in:fade={{ duration: 200, easing: cubicOut }}
              class="flex h-16 w-32 flex-col items-center justify-center gap-0.5 rounded-(--radius-md) border border-(--color-border-strong) bg-(--color-bg-elevated) px-5 py-2.5 text-(--color-fg-default) shadow-(--shadow-card) transition-all hover:-translate-y-0.5 hover:bg-(--color-bg-overlay) hover:shadow-(--shadow-glow) active:translate-y-0 active:scale-[0.97]"
              title="Copy"
            >
              <span class="flex items-center gap-1.5 text-sm font-medium">
                <Copy size={14} strokeWidth={2.25} />
                Copy
              </span>
              <span class="font-mono text-[10px] opacity-70">{shortcuts.label("copy")}</span>
            </button>
            <button
              type="button"
              onclick={flip}
              in:fade={{ duration: 200, easing: cubicOut }}
              class="flex h-16 w-32 flex-col items-center justify-center gap-0.5 rounded-(--radius-md) border border-(--color-border-strong) bg-(--color-bg-elevated) px-5 py-2.5 text-(--color-fg-default) shadow-(--shadow-card) transition-all hover:-translate-y-0.5 hover:bg-(--color-bg-overlay) hover:shadow-(--shadow-glow) active:translate-y-0 active:scale-[0.97]"
            >
              <span class="flex items-center gap-1.5 text-sm font-medium">
                <RotateCcw size={14} strokeWidth={2.25} />
                {t("reviewer.showQuestion")}
              </span>
              <span class="font-mono text-[10px] opacity-70">Space</span>
            </button>
          </div>
          <div class="flex items-center justify-center gap-3">
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
          </div>
        {/if}
      </div>
    {/if}
  </div>
  {#if copyError}
    <div
      role="alert"
      class="pointer-events-auto fixed bottom-6 left-1/2 z-20 flex max-w-lg -translate-x-1/2 gap-2 rounded-(--radius-md) border border-(--color-danger)/40 bg-(--color-danger)/10 px-4 py-2.5 text-xs text-(--color-danger) shadow-(--shadow-card) select-text"
    >
      <div class="min-w-0 flex-1">
        <p class="font-medium">Copy failed</p>
        <p class="mt-0.5 font-mono text-[11px] break-all opacity-90 select-all">{copyError}</p>
      </div>
      <div class="flex shrink-0 flex-col gap-1">
        <button
          type="button"
          onclick={copyErrorMessage}
          aria-label="Copy error"
          title="Copy error"
          class="grid h-5 w-5 place-items-center rounded text-(--color-danger) transition-colors hover:bg-(--color-danger)/20"
        >
          <Copy size={12} />
        </button>
        <button
          type="button"
          onclick={() => (copyError = null)}
          aria-label="Dismiss"
          title="Dismiss"
          class="grid h-5 w-5 place-items-center rounded text-(--color-danger) transition-colors hover:bg-(--color-danger)/20"
        >
          <X size={12} />
        </button>
      </div>
    </div>
  {:else if copyInfo}
    <div
      role="status"
      class="pointer-events-auto fixed bottom-6 left-1/2 z-20 -translate-x-1/2 rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-elevated) px-3 py-1.5 text-xs text-(--color-fg-default) shadow-(--shadow-card) select-text"
    >
      {copyInfo}
    </div>
  {/if}

</div>

<svelte:window onkeydown={onKey} />

{#if editing && current}
  <NoteEditor
    mode="edit"
    noteId={current.note_id}
    onClose={() => (editing = false)}
    onSaved={() => void reloadCurrent()}
  />
{/if}
