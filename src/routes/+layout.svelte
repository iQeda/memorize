<script lang="ts">
  import "../app.css";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import PageTransition from "$lib/components/PageTransition.svelte";
  import Launcher from "$lib/components/Launcher.svelte";
  import { theme } from "$lib/stores/theme.svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import { sync } from "$lib/stores/sync.svelte";
  import { checkForAppUpdates } from "$lib/updater";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  let { children } = $props();
  let launcherOpen = $state(false);

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
    // テキスト入力中（NoteEditor の input/textarea など）は Launcher を
    // 含むグローバルショートカットを無効化。Cmd+, (Settings) は global
    // navigation なので編集中でも有効のままにする。
    if (isTextField(e.target)) return;
    // Cmd+F / Ctrl+K → quick deck launcher.
    // - Cmd+F は webview default の in-page find を override（memorize は
    //   in-page find UI を持たないため、no-op だと混乱するので Launcher へ）
    // - Ctrl+K は Emacs/terminal 系の command palette ショートカット
    // 逆組み合わせ (Ctrl+F, Cmd+K) は誤爆防止のため受け付けない。
    const isLauncherKey =
      (e.metaKey && !e.ctrlKey && (e.key === "f" || e.key === "F")) ||
      (e.ctrlKey && !e.metaKey && (e.key === "k" || e.key === "K"));
    if (isLauncherKey) {
      e.preventDefault();
      launcherOpen = true;
      return;
    }
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

    // Auto sync on startup. Refresh sync status first so we know whether
    // the user is signed in to AnkiWeb.
    await sync.refresh();
    void sync.tryAutoSync(collection.isOpen);

    // Auto sync on shutdown. Rust intercepts every close path (⌘Q, app
    // menu Quit, window X button) and emits "memorize://exit-requested".
    // We run the sync then call `confirm_exit`, which sets the Rust
    // latch and triggers the actual exit.
    try {
      const [{ listen }, { invoke }] = await Promise.all([
        import("@tauri-apps/api/event"),
        import("@tauri-apps/api/core"),
      ]);
      let exiting = false;
      await listen("memorize://exit-requested", async () => {
        if (exiting) return;
        exiting = true;
        try {
          await sync.tryAutoSync(collection.isOpen);
        } finally {
          await invoke("confirm_exit");
        }
      });
    } catch (e) {
      // Not running inside Tauri.
      console.warn("auto sync on shutdown not registered", e);
    }
  });
</script>

<svelte:window onkeydown={onKey} />

<div class="grid h-full grid-cols-[auto_1fr] grid-rows-1">
  <Sidebar />
  <div class="grid min-w-0 grid-rows-[auto_1fr]">
    <TitleBar />
    <main class="relative min-w-0 overflow-y-auto scroll-smooth bg-(--color-bg-base)">
      <PageTransition>
        {@render children()}
      </PageTransition>
    </main>
  </div>
</div>

<Launcher bind:open={launcherOpen} />
