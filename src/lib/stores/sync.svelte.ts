import { invoke } from "$lib/ipc";

type SyncStatus = { logged_in: boolean; username: string | null };

type SyncReport = {
  kind: "no_changes" | "normal_done" | "full_required";
  upload_ok: boolean;
  download_ok: boolean;
  server_message: string;
};

class SyncStore {
  loggedIn = $state(false);
  username = $state<string | null>(null);
  busy = $state(false);
  lastError = $state<string | null>(null);
  lastMessage = $state<string | null>(null);
  fullSyncRequired = $state<{
    upload_ok: boolean;
    download_ok: boolean;
  } | null>(null);

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
    this.busy = true;
    this.lastError = null;
    this.lastMessage = null;
    this.fullSyncRequired = null;
    try {
      const r = await invoke<SyncReport>("sync_now");
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
    } finally {
      this.busy = false;
    }
  }

  async fullUpload() {
    this.busy = true;
    this.lastError = null;
    try {
      await invoke("sync_full_upload");
      this.fullSyncRequired = null;
      this.lastMessage = "フルアップロード完了";
    } catch (e) {
      this.lastError = String(e);
    } finally {
      this.busy = false;
    }
  }

  async fullDownload() {
    this.busy = true;
    this.lastError = null;
    try {
      await invoke("sync_full_download");
      this.fullSyncRequired = null;
      this.lastMessage = "フルダウンロード完了";
    } catch (e) {
      this.lastError = String(e);
    } finally {
      this.busy = false;
    }
  }
}

export const sync = new SyncStore();
