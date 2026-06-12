<script lang="ts">
  // settings 左サイドの目次 nav。
  import { t } from "$lib/i18n/index.svelte";

  export type TocGroup = {
    id: string;
    title: string;
    items: { id: string; label: string }[];
  };

  type Props = { groups: TocGroup[] };
  let { groups }: Props = $props();
</script>

<aside class="md:sticky md:top-6 md:self-start">
  <nav aria-labelledby="toc-heading">
    <p
      id="toc-heading"
      class="mb-3 text-[10px] font-semibold tracking-[0.18em] text-(--color-fg-subtle) uppercase"
    >
      {t("settings.contents")}
    </p>
    <div class="space-y-4">
      {#each groups as group (group.id)}
        <section aria-labelledby="toc-group-{group.id}">
          <h3
            id="toc-group-{group.id}"
            class="mb-1 text-[10px] font-semibold tracking-[0.18em] text-(--color-fg-default) uppercase"
          >
            {group.title}
          </h3>
          <ul class="space-y-0.5">
            {#each group.items as item (item.id)}
              <li>
                <a
                  href="#{item.id}"
                  class="block rounded-(--radius-sm) px-2 py-1 text-xs text-(--color-fg-muted) transition-colors hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)"
                >
                  {item.label}
                </a>
              </li>
            {/each}
          </ul>
        </section>
      {/each}
    </div>
  </nav>
</aside>
