<script lang="ts">
  // Nani / Speak / Hide-Reveal / Show Question / Show Answer で同一 class
  // だったアクションボタン。size="wide" は front の Show Answer 用。
  import type { ComponentType, Snippet, SvelteComponent } from "svelte";

  type Props = {
    icon: ComponentType<SvelteComponent<{ size?: number | string; strokeWidth?: number | string }>>;
    label: string;
    hotkey: string;
    onclick: () => void;
    title?: string;
    size?: "normal" | "wide";
  };
  let { icon: Icon, label, hotkey, onclick, title, size = "normal" }: Props = $props();
</script>

<button
  type="button"
  {onclick}
  title={title ?? label}
  class="flex h-16 flex-col items-center justify-center gap-0.5 rounded-(--radius-md) border border-(--color-border-strong) bg-(--color-bg-elevated) px-5 py-2.5 text-(--color-fg-default) shadow-(--shadow-card) transition-all hover:-translate-y-0.5 hover:bg-(--color-bg-overlay) hover:shadow-(--shadow-glow) active:translate-y-0 active:scale-[0.97] {size ===
  'wide'
    ? 'w-[420px]'
    : 'w-32'}"
>
  <span class="flex items-center gap-1.5 font-medium {size === 'wide' ? 'text-base' : 'text-sm'}">
    <Icon size={size === "wide" ? 16 : 14} strokeWidth={2.25} />
    {label}
  </span>
  <span class="font-mono text-[10px] opacity-70">{hotkey}</span>
</button>
