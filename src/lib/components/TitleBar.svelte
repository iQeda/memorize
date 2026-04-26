<script lang="ts">
  import { Sun, Moon } from "lucide-svelte";
  import { theme } from "$lib/stores/theme.svelte";
  import { draggable } from "$lib/actions/draggable";
</script>

<header
  use:draggable
  class="flex h-11 shrink-0 items-center justify-end border-b border-(--color-border-default) bg-(--color-bg-elevated) px-3"
>
  <button
    type="button"
    onclick={() => theme.toggle()}
    class="grid h-7 w-7 place-items-center rounded-md text-(--color-fg-muted) transition-colors hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default) active:scale-[0.96]"
    aria-label="Toggle theme"
  >
    {#if theme.resolved === "dark"}
      <Sun size={15} strokeWidth={2} />
    {:else}
      <Moon size={15} strokeWidth={2} />
    {/if}
  </button>
</header>
