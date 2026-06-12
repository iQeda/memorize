<script lang="ts">
  // settings の章 chrome: アンカー付き section + 見出し。カード枠の
  // padding はセクションごとに違う (p-1 / p-4 / p-5 / なし) ため children 側。
  import type { Snippet } from "svelte";

  type Props = {
    id: string;
    title: string;
    /** グループ見出し直後の最初のセクションは mt-4、それ以外は mt-10。 */
    first?: boolean;
    children: Snippet;
  };
  let { id, title, first = false, children }: Props = $props();
</script>

<section {id} class="scroll-mt-20 space-y-3 {first ? 'mt-4' : 'mt-10'}">
  <h3 class="text-xs font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
    {title}
  </h3>
  {@render children()}
</section>
