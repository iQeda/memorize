import { t } from "$lib/i18n/index.svelte";

let alreadyChecked = false;

export async function checkForAppUpdates(): Promise<void> {
  if (alreadyChecked) return;
  alreadyChecked = true;

  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await check();
    if (!update) return;

    const { ask } = await import("@tauri-apps/plugin-dialog");
    const ok = await ask(
      t("updater.askBody", {
        version: update.version,
        current: update.currentVersion,
      }),
      {
        title: t("updater.askTitle"),
        kind: "info",
        okLabel: t("updater.installNow"),
        cancelLabel: t("updater.later"),
      },
    );
    if (!ok) return;

    await update.downloadAndInstall();

    const { relaunch } = await import("@tauri-apps/plugin-process");
    await relaunch();
  } catch (err) {
    console.warn("[updater] check failed:", err);
  }
}
