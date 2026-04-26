<script lang="ts">
  import "../app.css";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import PageTransition from "$lib/components/PageTransition.svelte";
  import { theme } from "$lib/stores/theme.svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import { onMount } from "svelte";

  let { children } = $props();

  // Touch the theme store so its constructor runs and applies the class.
  $effect(() => {
    void theme.resolved;
  });

  onMount(async () => {
    // Try to auto-open a default profile if not yet open.
    if (collection.isOpen) return;
    const home =
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (globalThis as any).process?.env?.HOME ?? "";
    if (!home) return;
    // No-op for now: profile opening flow comes in Phase 1 wire-up.
  });
</script>

<div class="grid h-full grid-cols-[auto_1fr] grid-rows-1">
  <Sidebar />
  <div class="grid min-w-0 grid-rows-[auto_1fr]">
    <TitleBar />
    <main class="relative min-w-0 overflow-y-auto bg-(--color-bg-base)">
      <PageTransition>
        {@render children()}
      </PageTransition>
    </main>
  </div>
</div>
