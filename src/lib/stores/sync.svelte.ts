import { invoke } from "$lib/ipc";
import { browser } from "$app/environment";

type SyncStatus = { logged_in: boolean; username: string | null };

type SyncReport = {
  kind: "no_changes" | "normal_done" | "full_required";
  upload_ok: boolean;
  download_ok: boolean;
  server_message: string;
  host_number: number;
  new_endpoint: string | null;
  local_pending_notes: number;
  local_pending_cards: number;
};

type AutoBackupResult = { path: string };

type ProgressEvent =
  | {
      kind: "media_sync";
      checked: number;
      downloaded_files: number;
      downloaded_deletions: number;
      uploaded_files: number;
      uploaded_deletions: number;
    }
  | {
      kind: "normal_sync";
      stage: string;
      local_update: number;
      local_remove: number;
      remote_update: number;
      remote_remove: number;
    }
  | { kind: "full_sync"; transferred_bytes: number; total_bytes: number }
  | { kind: "import"; message: string }
  | { kind: "export"; message: string }
  | { kind: "other" };

const AUTO_BACKUP_KEY = "memorize:auto-backup-before-sync";

function fmtBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / (1024 * 1024)).toFixed(1)} MB`;
}

function describeProgress(p: ProgressEvent): string {
  switch (p.kind) {
    case "media_sync":
      return `メディア: ${p.checked} 件確認 / ↓${p.downloaded_files} ↑${p.uploaded_files}`;
    case "normal_sync":
      return `${p.stage}: 追加 ${p.local_update}/${p.remote_update} 削除 ${p.local_remove}/${p.remote_remove}`;
    case "full_sync":
      return p.total_bytes > 0
        ? `${fmtBytes(p.transferred_bytes)} / ${fmtBytes(p.total_bytes)}`
        : `${fmtBytes(p.transferred_bytes)} 転送中…`;
    case "import":
      return "Import 中…";
    case "export":
      return "Export 中…";
    default:
      return "処理中…";
  }
}

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
  lastReport = $state<SyncReport | null>(null);
  autoBackupBeforeSync = $state(true);

  constructor() {
    if (browser) {
      const stored = localStorage.getItem(AUTO_BACKUP_KEY);
      // Default: ON. Stored "0" = OFF.
      if (stored === "0") this.autoBackupBeforeSync = false;
      this.subscribeProgress();
    }
  }

  private async subscribeProgress() {
    try {
      const { listen } = await import("@tauri-apps/api/event");
      await listen<ProgressEvent>("progress", (e) => {
        if (this.busy) {
          this.busyReason = describeProgress(e.payload);
        }
      });
    } catch {
      // Not running inside Tauri (browser dev mode); ignore.
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
      this.lastReport = r;
      const pendingHint =
        r.local_pending_notes > 0 || r.local_pending_cards > 0
          ? ` / 未同期 ${r.local_pending_notes} 単語 + ${r.local_pending_cards} カード`
          : "";
      switch (r.kind) {
        case "no_changes":
          this.lastMessage = `変更なし${pendingHint}`;
          break;
        case "normal_done":
          this.lastMessage = `同期完了${pendingHint}`;
          break;
        case "full_required":
          this.fullSyncRequired = {
            upload_ok: r.upload_ok,
            download_ok: r.download_ok,
          };
          this.lastMessage = `フル同期が必要です${pendingHint}`;
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
