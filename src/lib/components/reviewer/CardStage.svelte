<script lang="ts">
  // カードの 3D フリップステージ。Question/Answer の CardFrame ×2 と
  // 上部の面バッジ。iframe は親が speech / hide 操作に使うため $bindable。
  import { fade } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import CardFrame from "$lib/components/CardFrame.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import type { StudyCard } from "$lib/reviewer/types";

  type Props = {
    card: StudyCard;
    answerHtml: string;
    showingAnswer: boolean;
    questionFrame?: HTMLIFrameElement;
    answerFrame?: HTMLIFrameElement;
  };
  let {
    card,
    answerHtml,
    showingAnswer,
    questionFrame = $bindable(),
    answerFrame = $bindable(),
  }: Props = $props();
</script>

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
  {#key card.card_id}
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
            html={card.question_html}
            css={card.css}
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
            html={answerHtml}
            css={card.css}
            side="answer"
          />
        </div>
      </div>
    </article>
  {/key}
</div>
