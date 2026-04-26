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
    await collection.refresh();
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
