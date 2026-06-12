<script lang="ts">
  // Settings の composition root。各セクションの実装は
  // $lib/components/settings/ 配下 (Phase 6 で 1308 行から分割)。
  import { onMount } from "svelte";
  import { sync } from "$lib/stores/sync.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import SettingsToc, { type TocGroup } from "$lib/components/settings/SettingsToc.svelte";
  import UpdatesCard from "$lib/components/settings/UpdatesCard.svelte";
  import CollectionSection from "$lib/components/settings/CollectionSection.svelte";
  import SyncSection from "$lib/components/settings/SyncSection.svelte";
  import BackupSection from "$lib/components/settings/BackupSection.svelte";
  import ImportExportSection from "$lib/components/settings/ImportExportSection.svelte";
  import LanguageSection from "$lib/components/settings/LanguageSection.svelte";
  import AppearanceSection from "$lib/components/settings/AppearanceSection.svelte";
  import StartupSection from "$lib/components/settings/StartupSection.svelte";
  import SpeechSection from "$lib/components/settings/SpeechSection.svelte";
  import ShortcutsSection from "$lib/components/settings/ShortcutsSection.svelte";

  const tocGroups = $derived<TocGroup[]>([
    {
      id: "data",
      title: t("settings.group.data"),
      items: [
        { id: "collection", label: t("settings.collection") },
        { id: "sync", label: t("sync.title") },
        { id: "backup", label: t("backup.title") },
        { id: "io", label: t("io.title") },
      ],
    },
    {
      id: "preferences",
      title: t("settings.group.preferences"),
      items: [
        { id: "language", label: t("settings.language") },
        { id: "appearance", label: t("settings.appearance") },
        { id: "startup", label: t("settings.startup") },
        { id: "speech", label: t("settings.speech") },
      ],
    },
    {
      id: "app",
      title: t("settings.group.app"),
      items: [
        { id: "shortcuts", label: t("settings.shortcuts") },
      ],
    },
  ]);

  onMount(() => {
    void sync.refresh();
  });
</script>

<div
  class="mx-auto grid max-w-6xl gap-10 px-8 py-10 md:grid-cols-[200px_minmax(0,1fr)]"
>
  <SettingsToc groups={tocGroups} />

  <div class="min-w-0 max-w-2xl">
    <header class="space-y-2">
      <h1 class="font-display text-3xl font-medium tracking-tight">
        {t("settings.title")}
      </h1>
      <p class="text-sm text-(--color-fg-muted)">
        {t("settings.subtitle")}
      </p>
    </header>

    <UpdatesCard />

    <h2 class="mt-12 mb-2 font-display text-xl font-medium tracking-tight text-(--color-fg-default)">
      {t("settings.group.data")}
    </h2>
    <CollectionSection />
    <SyncSection />
    <BackupSection />
    <ImportExportSection />

    <h2 class="mt-12 mb-2 font-display text-xl font-medium tracking-tight text-(--color-fg-default)">
      {t("settings.group.preferences")}
    </h2>
    <LanguageSection />
    <AppearanceSection />
    <StartupSection />
    <SpeechSection />

    <h2 class="mt-12 mb-2 font-display text-xl font-medium tracking-tight text-(--color-fg-default)">
      {t("settings.group.app")}
    </h2>
    <ShortcutsSection />
  </div>
</div>
