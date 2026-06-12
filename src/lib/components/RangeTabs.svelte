<script lang="ts" generics="T extends string | number">
  // 統計パネル右上のレンジ切替タブ。クラスはホームの旧 rangeBtn /
  // hRangeBtn / bRangeBtn スニペットから逐語コピー。
  type Props = {
    options: { value: T; label: string }[];
    value: T;
    onSelect: (v: T) => void;
  };
  let { options, value, onSelect }: Props = $props();
</script>

<div class="flex gap-1 text-[11px]">
  {#each options as opt, i (i)}
    <button
      type="button"
      onclick={() => onSelect(opt.value)}
      class="rounded-(--radius-sm) px-2 py-0.5 transition-colors
        {value === opt.value
        ? 'bg-(--color-accent-500) text-(--color-fg-onAccent)'
        : 'text-(--color-fg-muted) hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)'}"
    >
      {opt.label}
    </button>
  {/each}
</div>
