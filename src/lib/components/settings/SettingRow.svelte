<script lang="ts">
  // settings の「アイコン + ラベル/サブタイトル + 右端アクション」行。
  // divider=true で直前行との区切り線 (mt-4 border-t pt-4) を付ける。
  import type { ComponentType, Snippet, SvelteComponent } from "svelte";

  type Props = {
    /** lucide-svelte のアイコン (Svelte 4 形式クラスコンポーネント)。 */
    icon?: ComponentType<SvelteComponent<{ size?: number | string; class?: string }>>;
    iconClass?: string;
    label: string;
    subtitle?: string;
    divider?: boolean;
    action: Snippet;
  };
  let {
    icon: Icon,
    iconClass = "text-(--color-accent-500)",
    label,
    subtitle,
    divider = false,
    action,
  }: Props = $props();
</script>

<div
  class="flex items-center justify-between gap-4 {divider
    ? 'mt-4 border-t border-(--color-border-default) pt-4'
    : ''}"
>
  <div class="flex items-center gap-2.5">
    {#if Icon}
      <Icon size={16} class={iconClass} />
    {/if}
    <div class="text-sm">
      <p class="text-(--color-fg-default)">{label}</p>
      {#if subtitle}
        <p class="mt-0.5 text-xs text-(--color-fg-subtle)">{subtitle}</p>
      {/if}
    </div>
  </div>
  {@render action()}
</div>
