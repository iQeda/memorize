<script lang="ts">
  import { Search, X } from "lucide-svelte";
  import { collection, type DeckSummary } from "$lib/stores/collection.svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { tick } from "svelte";
  import { fade } from "svelte/transition";
  import { t } from "$lib/i18n/index.svelte";

  let { open = $bindable(false) }: { open?: boolean } = $props();

  let query = $state("");
  let activeIndex = $state(0);
  let inputEl = $state<HTMLInputElement | null>(null);
  let listEl = $state<HTMLUListElement | null>(null);

  function deckShortName(name: string): string {
    return name.split("::").at(-1) ?? name;
  }

  function score(deck: DeckSummary, q: string): number {
    if (!q) return 0;
    const haystack = deck.name.toLowerCase();
    const needle = q.toLowerCase();
    const idx = haystack.indexOf(needle);
    if (idx < 0) return -1;
    // Earlier match in the short name beats earlier match deep in path.
    const short = deckShortName(haystack);
    const shortIdx = short.indexOf(needle);
    return shortIdx >= 0 ? 1000 - shortIdx : 100 - idx;
  }

  const filtered = $derived.by<DeckSummary[]>(() => {
    const q = query.trim();
    if (!q) return collection.decks;
    return collection.decks
      .map((d) => ({ d, s: score(d, q) }))
      .filter((x) => x.s >= 0)
      .sort((a, b) => b.s - a.s)
      .map((x) => x.d);
  });

  // Reset highlight whenever the visible list changes.
  $effect(() => {
    void filtered;
    activeIndex = 0;
  });

  // Auto-focus the search input when the launcher opens.
  $effect(() => {
    if (open) {
      query = "";
      activeIndex = 0;
      void tick().then(() => inputEl?.focus());
    }
  });

  function close() {
    open = false;
  }

  async function pick(deck: DeckSummary) {
    collection.selectedDeckId = deck.id;
    open = false;
    if ($page.url.pathname !== "/") {
      await goto("/");
    }
  }

  function moveBy(delta: number) {
    if (filtered.length === 0) return;
    const len = filtered.length;
    activeIndex = (activeIndex + delta + len) % len;
    void tick().then(() => {
      const el = listEl?.querySelector<HTMLElement>(`[data-idx="${activeIndex}"]`);
      el?.scrollIntoView({ block: "nearest" });
    });
  }

  function onInputKey(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      close();
      return;
    }
    // Arrow keys + Emacs-style Ctrl+N / Ctrl+P (familiar from terminals,
    // readline, fzf, Alfred, Raycast, etc).
    if (e.key === "ArrowDown" || (e.ctrlKey && (e.key === "n" || e.key === "N"))) {
      e.preventDefault();
      moveBy(1);
      return;
    }
    if (e.key === "ArrowUp" || (e.ctrlKey && (e.key === "p" || e.key === "P"))) {
      e.preventDefault();
      moveBy(-1);
      return;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      const target = filtered[activeIndex];
      if (target) void pick(target);
    }
  }

  type BadgeTone = "accent" | "warning" | "success";

  function deckBadges(d: DeckSummary): { count: number; tone: BadgeTone }[] {
    const out: { count: number; tone: BadgeTone }[] = [];
    if (d.new_count > 0) out.push({ count: d.new_count, tone: "accent" });
    if (d.learn_count > 0) out.push({ count: d.learn_count, tone: "warning" });
    if (d.review_count > 0) out.push({ count: d.review_count, tone: "success" });
    return out;
  }

  const badgeTone: Record<BadgeTone, string> = {
    accent: "bg-(--color-accent-500)/12 text-(--color-accent-500) ring-(--color-accent-500)/20",
    warning: "bg-(--color-warning)/12 text-(--color-warning) ring-(--color-warning)/20",
    success: "bg-(--color-success)/12 text-(--color-success) ring-(--color-success)/20",
  };
</script>

{#if open}
  <div
    role="dialog"
    aria-modal="true"
    aria-label={t("launcher.title")}
    class="fixed inset-0 z-50 flex items-start justify-center px-4 pt-[15vh]"
    transition:fade={{ duration: 120 }}
  >
    <button
      type="button"
      aria-label={t("launcher.close")}
      onclick={close}
      class="absolute inset-0 cursor-default bg-black/40 backdrop-blur-sm"
    ></button>

    <div
      class="relative flex w-full max-w-xl flex-col overflow-hidden rounded-(--radius-xl) border border-(--color-border-default) bg-(--color-bg-elevated) shadow-(--shadow-card)"
    >
      <div class="flex items-center gap-2 border-b border-(--color-border-default) px-3 py-2">
        <Search size={14} class="shrink-0 text-(--color-fg-subtle)" />
        <input
          bind:this={inputEl}
          bind:value={query}
          onkeydown={onInputKey}
          placeholder={t("launcher.placeholder")}
          class="min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-(--color-fg-subtle)"
        />
        <button
          type="button"
          onclick={close}
          aria-label={t("launcher.close")}
          class="grid h-5 w-5 place-items-center rounded text-(--color-fg-subtle) transition-colors hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)"
        >
          <X size={12} />
        </button>
      </div>

      <ul
        bind:this={listEl}
        class="max-h-[50vh] overflow-y-auto"
        role="listbox"
      >
        {#if filtered.length === 0}
          <li class="px-4 py-6 text-center text-xs text-(--color-fg-subtle)">
            {collection.decks.length === 0 ? t("launcher.noDecks") : t("launcher.empty")}
          </li>
        {:else}
          {#each filtered as deck, i (deck.id)}
            {@const active = i === activeIndex}
            {@const badges = deckBadges(deck)}
            <li
              role="option"
              aria-selected={active}
              data-idx={i}
            >
              <button
                type="button"
                onmouseenter={() => (activeIndex = i)}
                onclick={() => pick(deck)}
                class="flex w-full items-center justify-between gap-3 px-3 py-2 text-left text-sm transition-colors
                  {active
                  ? 'bg-(--color-accent-500)/12 text-(--color-fg-default)'
                  : 'text-(--color-fg-muted) hover:bg-(--color-bg-overlay)'}"
                style="padding-left: {0.75 + deck.level * 0.75}rem;"
              >
                <span class="min-w-0 flex-1 truncate text-(--color-fg-default)">
                  {deckShortName(deck.name)}
                </span>
                {#if badges.length > 0}
                  <span class="flex shrink-0 items-center gap-1">
                    {#each badges as b (b.tone)}
                      <span
                        class="number-tabular rounded-full px-1.5 py-0.5 text-[10px] font-semibold ring-1 ring-inset {badgeTone[b.tone]}"
                      >
                        {b.count}
                      </span>
                    {/each}
                  </span>
                {/if}
              </button>
            </li>
          {/each}
        {/if}
      </ul>

      <div
        class="flex items-center justify-between gap-3 border-t border-(--color-border-default) bg-(--color-bg-sunken) px-3 py-1.5 text-[10px] text-(--color-fg-subtle)"
      >
        <span class="flex items-center gap-2">
          <kbd class="rounded border border-(--color-border-default) bg-(--color-bg-base) px-1 py-0.5 font-mono">↑↓</kbd>
          <kbd class="rounded border border-(--color-border-default) bg-(--color-bg-base) px-1 py-0.5 font-mono">^P/^N</kbd>
          {t("launcher.hintNavigate")}
          <kbd class="ml-2 rounded border border-(--color-border-default) bg-(--color-bg-base) px-1 py-0.5 font-mono">↵</kbd>
          {t("launcher.hintSelect")}
        </span>
        <span class="flex items-center gap-1">
          <kbd class="rounded border border-(--color-border-default) bg-(--color-bg-base) px-1 py-0.5 font-mono">Esc</kbd>
          {t("launcher.hintClose")}
        </span>
      </div>
    </div>
  </div>
{/if}
