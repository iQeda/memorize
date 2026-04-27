<script lang="ts">
  import "../app.css";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import PageTransition from "$lib/components/PageTransition.svelte";
  import { theme } from "$lib/stores/theme.svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import { checkForAppUpdates } from "$lib/updater";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  let { children } = $props();

  function isTextField(target: EventTarget | null): boolean {
    const el = target as HTMLElement | null;
    if (!el) return false;
    const tag = el.tagName;
    return (
      tag === "INPUT" ||
      tag === "TEXTAREA" ||
      tag === "SELECT" ||
      el.isContentEditable
    );
  }

  function onKey(e: KeyboardEvent) {
    // ⌘, (macOS) / Ctrl+, (other) → Settings
    if ((e.metaKey || e.ctrlKey) && e.key === ",") {
      e.preventDefault();
      void goto("/settings/");
      return;
    }
    if (isTextField(e.target)) return;
    // Plain shortcuts (only when not in a text field)
    if (e.key === "?" && (e.metaKey || e.shiftKey)) {
      // Cmd+? or Shift+? — could open shortcuts help in the future
    }
  }

  // Touch the theme store so its constructor runs and applies the class.
  $effect(() => {
    void theme.resolved;
  });

  onMount(async () => {
    await collection.refresh();
    void checkForAppUpdates();
  });
</script>

<svelte:window onkeydown={onKey} />

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
