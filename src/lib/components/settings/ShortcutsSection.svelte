<script lang="ts">
  import { shortcuts, type Action } from "$lib/stores/shortcuts.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import SettingsSection from "./SettingsSection.svelte";

  const ratingShortcuts = $derived<{ action: Action; label: string }[]>([
    { action: "again", label: t("settings.shortcut.again") },
    { action: "hard", label: t("settings.shortcut.hard") },
    { action: "good", label: t("settings.shortcut.good") },
    { action: "easy", label: t("settings.shortcut.easy") },
    { action: "copy", label: t("settings.shortcut.copy") },
    { action: "speak", label: t("settings.shortcut.speak") },
    { action: "hide", label: t("settings.shortcut.hide") },
  ]);

  const fixedShortcuts = $derived([
    { keys: ["Space"], label: t("settings.shortcut.spaceLabel") },
    { keys: ["⌘", ","], label: t("settings.shortcut.openSettings") },
    { keys: ["⌘", "F / K"], label: t("settings.shortcut.openLauncher") },
    { keys: ["E"], label: t("settings.shortcut.editNote") },
    { keys: ["⇧", "L"], label: t("settings.shortcut.toggleHideDefault") },
  ]);

  let recordingFor = $state<Action | null>(null);

  function startRecord(action: Action) {
    recordingFor = action;
    const handler = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      window.removeEventListener("keydown", handler, true);
      if (e.key === "Escape") {
        recordingFor = null;
        return;
      }
      shortcuts.set(action, e.key);
      recordingFor = null;
    };
    window.addEventListener("keydown", handler, true);
  }
</script>

<SettingsSection id="shortcuts" title={t("settings.shortcuts")} first>
  <div
    class="overflow-hidden rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) shadow-(--shadow-subtle)"
  >
    {#each ratingShortcuts as s, i (s.action)}
      <div
        class="flex items-center justify-between gap-4 px-4 py-2.5 {i > 0
          ? 'border-t border-(--color-border-default)'
          : ''}"
      >
        <span class="text-sm text-(--color-fg-default)">{s.label}</span>
        <button
          type="button"
          onclick={() => startRecord(s.action)}
          class="rounded-(--radius-xs) border px-2 py-0.5 font-mono text-xs transition-colors
            {recordingFor === s.action
            ? 'border-(--color-accent-500) bg-(--color-accent-500)/10 text-(--color-accent-500) animate-pulse'
            : 'border-(--color-border-default) bg-(--color-bg-base) text-(--color-fg-muted) hover:border-(--color-border-strong) hover:text-(--color-fg-default)'}"
          title={recordingFor === s.action ? "Press a key… (Esc to cancel)" : "Click to rebind"}
        >
          {recordingFor === s.action ? "…" : shortcuts.label(s.action)}
        </button>
      </div>
    {/each}
    {#each fixedShortcuts as s (s.label)}
      <div
        class="flex items-center justify-between gap-4 border-t border-(--color-border-default) px-4 py-2.5"
      >
        <span class="text-sm text-(--color-fg-default)">{s.label}</span>
        <div class="flex gap-1">
          {#each s.keys as k (k)}
            <kbd
              class="rounded-(--radius-xs) border border-(--color-border-default) bg-(--color-bg-base) px-1.5 py-0.5 font-mono text-xs text-(--color-fg-muted)"
              >{k}</kbd
            >
          {/each}
        </div>
      </div>
    {/each}
    <div class="flex items-center justify-end gap-2 border-t border-(--color-border-default) px-4 py-2">
      <button
        type="button"
        onclick={() => shortcuts.reset()}
        class="text-[11px] text-(--color-fg-subtle) hover:text-(--color-fg-default)"
      >
        Reset to default (1/2/3/4 + n)
      </button>
    </div>
  </div>
</SettingsSection>
