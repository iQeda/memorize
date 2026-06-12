<script lang="ts">
  // 解答後の 4 トーン採点ボタン列。
  import { fade } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { t } from "$lib/i18n/index.svelte";
  import { shortcuts, type Rating } from "$lib/stores/shortcuts.svelte";

  type Props = { onRate: (rating: Rating) => void };
  let { onRate }: Props = $props();

  type Tone = "danger" | "warning" | "accent" | "success";
  const buttons = $derived<{ rating: Rating; label: string; tone: Tone }[]>([
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

<div class="flex items-center justify-center gap-3">
  {#each buttons as b, i (b.rating)}
    <button
      type="button"
      onclick={() => onRate(b.rating)}
      in:fade={{ duration: 200, delay: 40 + i * 30, easing: cubicOut }}
      class="flex min-w-[88px] flex-col items-center gap-0.5 rounded-(--radius-md) px-5 py-2.5 shadow-(--shadow-card) transition-all hover:-translate-y-0.5 hover:shadow-(--shadow-glow) active:translate-y-0 active:scale-[0.97] {toneBg[b.tone]}"
    >
      <span class="text-sm font-medium">{b.label}</span>
      <span class="font-mono text-[10px] opacity-70">{shortcuts.label(b.rating)}</span>
    </button>
  {/each}
</div>
