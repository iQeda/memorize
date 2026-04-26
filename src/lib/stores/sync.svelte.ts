import { invoke } from "$lib/ipc";
import { browser } from "$app/environment";

type SyncStatus = { logged_in: boolean; username: string | null };

type SyncReport = {
  kind: "no_changes" | "normal_done" | "full_required";
  upload_ok: boolean;
  download_ok: boolean;
  server_message: string;
};

type AutoBackupResult = { path: string };

const AUTO_BACKUP_KEY = "memorize:auto-backup-before-sync";

class SyncStore {
  loggedIn = $state(false);
  username = $state<string | null>(null);
  busy = $state(false);
  busyReason = $state<string | null>(null);
  lastError = $state<string | null>(null);
  lastMessage = $state<string | null>(null);
  fullSyncRequired = $state<{
    upload_ok: boolean;
    download_ok: boolean;
  } | null>(null);
  lastBackupPath = $state<string | null>(null);
  autoBackupBeforeSync = $state(true);

  constructor() {
    if (browser) {
      const stored = localStorage.getItem(AUTO_BACKUP_KEY);
      // Default: ON. Stored "0" = OFF.
      if (stored === "0") this.autoBackupBeforeSync = false;
    }
  }

  setAutoBackup(enabled: boolean) {
    this.autoBackupBeforeSync = enabled;
    if (browser) localStorage.setItem(AUTO_BACKUP_KEY, enabled ? "1" : "0");
  }

  private async runWithAutoBackup<T>(
    label: string,
    fn: () => Promise<T>,
  ): Promise<T> {
    if (this.autoBackupBeforeSync) {
      this.busy = true;
      this.busyReason = "バックアップ作成中…";
      try {
        const r = await invoke<AutoBackupResult>("auto_backup", {
          includeMedia: false,
        });
        this.lastBackupPath = r.path;
      } catch (e) {
        this.busy = false;
        this.busyReason = null;
        this.lastError = `バックアップ失敗のため ${label} を中止: ${e}`;
        throw e;
      }
    }
    this.busy = true;
    this.busyReason = label;
    try {
      return await fn();
    } finally {
      this.busy = false;
      this.busyReason = null;
    }
  }

  async manualBackup(outPath: string, includeMedia: boolean) {
    this.busy = true;
    this.busyReason = "バックアップ作成中…";
    this.lastError = null;
    try {
      await invoke("export_colpkg", { outPath, includeMedia });
      this.lastBackupPath = outPath;
      this.lastMessage = `バックアップ作成: ${outPath}`;
    } catch (e) {
      this.lastError = String(e);
    } finally {
      this.busy = false;
      this.busyReason = null;
    }
  }

  async restore(inPath: string) {
    this.busy = true;
    this.busyReason = "復元中…";
    this.lastError = null;
    this.lastMessage = null;
    try {
      await invoke("import_colpkg", { inPath });
      this.lastMessage = `復元完了: ${inPath}`;
    } catch (e) {
      this.lastError = String(e);
    } finally {
      this.busy = false;
      this.busyReason = null;
    }
  }

  async refresh() {
    try {
      const s = await invoke<SyncStatus>("sync_status");
      this.loggedIn = s.logged_in;
      this.username = s.username;
    } catch (e) {
      this.lastError = String(e);
    }
  }

  async login(username: string, password: string, endpoint?: string) {
    this.busy = true;
    this.lastError = null;
    try {
      const s = await invoke<SyncStatus>("sync_login_cmd", {
        input: { username, password, endpoint: endpoint || null },
      });
      this.loggedIn = s.logged_in;
      this.username = s.username;
    } catch (e) {
      this.lastError = String(e);
      throw e;
    } finally {
      this.busy = false;
    }
  }

  async logout() {
    this.busy = true;
    try {
      await invoke("sync_logout");
      this.loggedIn = false;
      this.username = null;
    } finally {
      this.busy = false;
    }
  }

  async syncNow() {
    this.lastError = null;
    this.lastMessage = null;
    this.fullSyncRequired = null;
    try {
      const r = await this.runWithAutoBackup("同期中…", () =>
        invoke<SyncReport>("sync_now"),
      );
      switch (r.kind) {
        case "no_changes":
          this.lastMessage = "変更なし";
          break;
        case "normal_done":
          this.lastMessage = "同期完了";
          break;
        case "full_required":
          this.fullSyncRequired = {
            upload_ok: r.upload_ok,
            download_ok: r.download_ok,
          };
          this.lastMessage = "フル同期が必要です";
          break;
      }
      if (r.server_message) this.lastMessage += ` — ${r.server_message}`;
    } catch (e) {
      this.lastError = String(e);
    }
  }

  async fullUpload() {
    this.lastError = null;
    try {
      await this.runWithAutoBackup("ローカル → サーバーへ上書き中…", () =>
        invoke("sync_full_upload"),
      );
      this.fullSyncRequired = null;
      this.lastMessage = "フルアップロード完了";
    } catch (e) {
      this.lastError = String(e);
    }
  }

  async fullDownload() {
    this.lastError = null;
    try {
      await this.runWithAutoBackup("サーバー → ローカルへ上書き中…", () =>
        invoke("sync_full_download"),
      );
      this.fullSyncRequired = null;
      this.lastMessage = "フルダウンロード完了";
    } catch (e) {
      this.lastError = String(e);
    }
  }
}

export const sync = new SyncStore();
