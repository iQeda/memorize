<script lang="ts">
  import { page } from "$app/stores";
  import {
    Library,
    Search,
    Settings as SettingsIcon,
    Brain,
  } from "lucide-svelte";
  import { collection, type DeckSummary } from "$lib/stores/collection.svelte";
  import { draggable } from "$lib/actions/draggable";

  const navItems = [
    { href: "/", label: "Decks", icon: Library },
    { href: "/browse/", label: "Browse", icon: Search },
    { href: "/settings/", label: "Settings", icon: SettingsIcon },
  ];

  function isActive(href: string): boolean {
    if (href === "/") return $page.url.pathname === "/";
    return $page.url.pathname.startsWith(href);
  }

  function deckShortName(name: string): string {
    return name.split("::").at(-1) ?? name;
  }

  function deckBadge(d: DeckSummary): { count: number; tone: "accent" | "warning" | "success" } | null {
    if (d.new_count > 0) return { count: d.new_count, tone: "accent" };
    if (d.learn_count > 0) return { count: d.learn_count, tone: "warning" };
    if (d.review_count > 0) return { count: d.review_count, tone: "success" };
    return null;
  }

  const badgeTone = {
    accent:
      "bg-(--color-accent-500)/12 text-(--color-accent-500) ring-(--color-accent-500)/20",
    warning:
      "bg-(--color-warning)/12 text-(--color-warning) ring-(--color-warning)/20",
    success:
      "bg-(--color-success)/12 text-(--color-success) ring-(--color-success)/20",
  } as const;
</script>

<aside
  class="flex h-full w-60 shrink-0 flex-col border-r border-(--color-border-default) bg-(--color-bg-sunken)"
>
  <div
    use:draggable
    class="flex h-11 items-center pr-4 pl-[78px]"
  >
    <div class="flex items-center gap-2">
      <div
        class="grid h-6 w-6 place-items-center rounded-md bg-(--color-accent-500) text-(--color-fg-onAccent) shadow-(--shadow-subtle)"
      >
        <Brain size={14} strokeWidth={2.5} />
      </div>
      <span class="font-display text-sm font-medium tracking-tight">
        memorize
      </span>
    </div>
  </div>

  <nav class="flex flex-col gap-0.5 px-2 pt-1">
    {#each navItems as item (item.href)}
      {@const active = isActive(item.href)}
      <a
        href={item.href}
        data-sveltekit-preload-data="hover"
        class="flex items-center gap-2.5 rounded-md px-2.5 py-1.5 text-sm transition-all duration-150
          {active
          ? 'bg-(--color-bg-elevated) text-(--color-fg-default) shadow-(--shadow-subtle)'
          : 'text-(--color-fg-muted) hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)'}"
      >
        <item.icon size={16} strokeWidth={2} />
        {item.label}
      </a>
    {/each}
  </nav>

  {#if collection.isOpen && collection.decks.length > 0}
    <div class="mt-5 flex-1 overflow-y-auto px-2 pb-3">
      <div
        class="mb-1 flex items-center justify-between px-2.5"
      >
        <span
          class="text-[10px] font-semibold tracking-[0.14em] text-(--color-fg-subtle) uppercase"
        >
          デッキ
        </span>
        <span class="number-tabular text-[10px] text-(--color-fg-subtle)">
          {collection.decks.length}
        </span>
      </div>
      {#each collection.decks as deck (deck.id)}
        {@const active = collection.selectedDeckId === deck.id}
        {@const badge = deckBadge(deck)}
        <button
          type="button"
          onclick={() => (collection.selectedDeckId = deck.id)}
          class="group flex w-full items-center justify-between gap-2 rounded-md py-1 pr-2 text-left text-sm transition-colors
            {active
            ? 'bg-(--color-bg-elevated) text-(--color-fg-default) shadow-(--shadow-subtle)'
            : 'text-(--color-fg-muted) hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)'}"
          style="padding-left: {0.625 + deck.level * 0.75}rem;"
        >
          <span class="flex min-w-0 items-center gap-2">
            <span
              class="h-1.5 w-1.5 shrink-0 rounded-full transition-colors
                {badge ? (badge.tone === 'accent' ? 'bg-(--color-accent-500)' : badge.tone === 'warning' ? 'bg-(--color-warning)' : 'bg-(--color-success)') : 'bg-(--color-border-strong) group-hover:bg-(--color-fg-subtle)'}"
            ></span>
            <span class="truncate">{deckShortName(deck.name)}</span>
          </span>
          {#if badge}
            <span
              class="number-tabular shrink-0 rounded-full px-1.5 py-0.5 text-[10px] font-semibold ring-1 ring-inset {badgeTone[badge.tone]}"
            >
              {badge.count}
            </span>
          {/if}
        </button>
      {/each}
    </div>
  {:else}
    <div class="flex-1"></div>
  {/if}
</aside>
