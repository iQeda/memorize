<script lang="ts">
  import "../app.css";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import PageTransition from "$lib/components/PageTransition.svelte";
  import Launcher from "$lib/components/Launcher.svelte";
  import ImportPreviewModal from "$lib/components/ImportPreviewModal.svelte";
  import { theme } from "$lib/stores/theme.svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import { sync } from "$lib/stores/sync.svelte";
  import { checkForAppUpdates } from "$lib/updater";
  import { isTextFieldTarget } from "$lib/utils/keyboard";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount } from "svelte";

  let { children } = $props();
  let launcherOpen = $state(false);
  let mainEl = $state<HTMLElement | undefined>();

  // デッキを切り替えたら main コンテナの縦スクロールを先頭に戻す。
  // ホームの stat panel グリッドや review / browse の中身が入れ替わっても
  // スクロール位置は前デッキのまま残るので、別デッキの異なる長さのコンテンツ
  // を途中位置で開かないようにする。scroll-smooth クラスで scroll-behavior:
  // smooth が CSS 経由で効くため、scrollTo の behavior 指定は不要。
  $effect(() => {
    void collection.selectedDeckId;
    mainEl?.scrollTo({ top: 0 });
  });

  // ルート遷移 (settings → home, home → browse など) でも main コンテナ
  // 自前のスクロールを top に戻す。SvelteKit は window スクロールはリセット
  // するが、layout が持つ overflow:auto コンテナの scrollTop はそのまま
  // 残るため (sidebar + main 構造で main 側のみスクロール)、明示的に戻す。
  $effect(() => {
    void $page.url.pathname;
    mainEl?.scrollTo({ top: 0 });
  });

  function onKey(e: KeyboardEvent) {
    // ⌘, → Settings (macOS のみサポート、Ctrl 修飾子は受け付けない)
    if (e.metaKey && !e.ctrlKey && e.key === ",") {
      e.preventDefault();
      void goto("/settings/");
      return;
    }
    // ⌘S → sync now (TitleBar の Sync ボタンと同等)。memorize には
    // form 保存動作がないので Cmd+S は WebView 内で衝突せず、Cmd+, と同じく
    // テキスト入力中でも発火させる global action として扱う。
    if (e.metaKey && !e.ctrlKey && (e.key === "s" || e.key === "S")) {
      e.preventDefault();
      if (collection.isOpen) void sync.syncNow();
      return;
    }
    // テキスト入力中（NoteEditor の input/textarea など）は Launcher を
    // 含むグローバルショートカットを無効化。Cmd+, (Settings) は global
    // navigation なので編集中でも有効のままにする。
    if (isTextFieldTarget(e.target)) return;
    // Cmd+F / Cmd+K → quick deck launcher.
    // - Cmd+F は webview default の in-page find を override（memorize は
    //   in-page find UI を持たないため、no-op だと混乱するので Launcher へ）
    // - Cmd+K は他アプリの command palette と同じ慣習
    // Ctrl 修飾子は受け付けない (誤爆防止)。
    const isLauncherKey =
      e.metaKey && !e.ctrlKey && (e.key === "f" || e.key === "F" || e.key === "k" || e.key === "K");
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
          await invoke<void>("confirm_exit");
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
    <main bind:this={mainEl} class="relative min-w-0 overflow-y-auto scroll-smooth bg-(--color-bg-base)">
      <PageTransition>
        {@render children()}
      </PageTransition>
    </main>
  </div>
</div>

<Launcher bind:open={launcherOpen} />
<ImportPreviewModal />
