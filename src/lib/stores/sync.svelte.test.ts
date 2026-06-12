import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "$lib/ipc";
import { sync } from "./sync.svelte";

vi.mock("$lib/ipc", () => ({ invoke: vi.fn() }));

const invokeMock = vi.mocked(invoke);

beforeEach(() => {
  invokeMock.mockReset();
  // Singleton store — reset the state every test.
  sync.loggedIn = false;
  sync.username = null;
  sync.busy = false;
  sync.busyReason = null;
  sync.lastError = null;
  sync.lastMessage = null;
  sync.lastBackupPath = null;
});

describe("sync.login", () => {
  it("sets loggedIn/username on success and resets busy", async () => {
    invokeMock.mockResolvedValue({ logged_in: true, username: "alice" });
    await sync.login("alice", "pw");
    expect(sync.loggedIn).toBe(true);
    expect(sync.username).toBe("alice");
    expect(sync.busy).toBe(false);
    expect(sync.lastError).toBeNull();
  });

  it("rethrows on failure, records lastError, and resets busy", async () => {
    invokeMock.mockRejectedValue(new Error("bad credentials"));
    await expect(sync.login("alice", "wrong")).rejects.toThrow("bad credentials");
    expect(sync.lastError).toContain("bad credentials");
    expect(sync.busy).toBe(false);
    expect(sync.loggedIn).toBe(false);
  });
});

describe("sync.logout", () => {
  it("clears loggedIn/username and resets busy", async () => {
    sync.loggedIn = true;
    sync.username = "alice";
    invokeMock.mockResolvedValue(undefined);
    await sync.logout();
    expect(sync.loggedIn).toBe(false);
    expect(sync.username).toBeNull();
    expect(sync.busy).toBe(false);
  });

  it("resets busy even when the backend call fails", async () => {
    invokeMock.mockRejectedValue(new Error("keychain unavailable"));
    await expect(sync.logout()).rejects.toThrow("keychain unavailable");
    expect(sync.busy).toBe(false);
  });
});

describe("sync.manualBackup", () => {
  it("records the backup path and a message on success", async () => {
    invokeMock.mockResolvedValue(undefined);
    await sync.manualBackup("/tmp/backup.colpkg", false);
    expect(invokeMock).toHaveBeenCalledWith("export_colpkg", {
      outPath: "/tmp/backup.colpkg",
      includeMedia: false,
    });
    expect(sync.lastBackupPath).toBe("/tmp/backup.colpkg");
    expect(sync.lastMessage).toContain("/tmp/backup.colpkg");
    expect(sync.busy).toBe(false);
  });

  it("swallows errors into lastError and resets busy (error path)", async () => {
    invokeMock.mockRejectedValue(new Error("disk full"));
    await sync.manualBackup("/tmp/backup.colpkg", false);
    expect(sync.lastError).toContain("disk full");
    expect(sync.busy).toBe(false);
    expect(sync.busyReason).toBeNull();
    expect(sync.lastBackupPath).toBeNull();
  });
});

describe("sync.restore", () => {
  it("sets a message on success", async () => {
    invokeMock.mockResolvedValue(undefined);
    await sync.restore("/tmp/backup.colpkg");
    expect(sync.lastMessage).toContain("/tmp/backup.colpkg");
    expect(sync.busy).toBe(false);
  });

  it("clears a stale message and records lastError on failure", async () => {
    sync.lastMessage = "stale";
    invokeMock.mockRejectedValue(new Error("corrupt colpkg"));
    await sync.restore("/tmp/backup.colpkg");
    expect(sync.lastMessage).toBeNull();
    expect(sync.lastError).toContain("corrupt colpkg");
    expect(sync.busy).toBe(false);
  });
});
