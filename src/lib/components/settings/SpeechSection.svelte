<script lang="ts">
  import { EyeOff } from "lucide-svelte";
  import { speech } from "$lib/stores/speech.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import SettingsSection from "./SettingsSection.svelte";
  import SettingRow from "./SettingRow.svelte";
  import ToggleSwitch from "$lib/components/ToggleSwitch.svelte";
  import SpeechControls from "$lib/components/SpeechControls.svelte";
</script>

<SettingsSection id="speech" title={t("settings.speech")}>
  <div
    class="rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-5 shadow-(--shadow-subtle)"
  >
    <SpeechControls layout="rows" />
    <SettingRow
      icon={EyeOff}
      label={t("settings.speech.hideDefaultLabel")}
      subtitle={t("settings.speech.hideDefaultBody")}
      divider
    >
      {#snippet action()}
        <ToggleSwitch
          checked={speech.hideDefault}
          onToggle={(v) => speech.setHideDefault(v)}
          label={t("settings.speech.hideDefaultLabel")}
        />
      {/snippet}
    </SettingRow>
    <p class="mt-3 text-xs text-(--color-fg-subtle)">{t("settings.speech.macOnly")}</p>
  </div>
</SettingsSection>
