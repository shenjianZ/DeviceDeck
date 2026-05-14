import { create } from "zustand";
import { getVersion } from "@tauri-apps/api/app";
import { checkForAppUpdate, downloadAppUpdate, installAppUpdate } from "../lib/updater";
import type { UpdateState } from "../lib/update-types";
import type { Update, DownloadEvent } from "@tauri-apps/plugin-updater";
import { useSettingsStore } from "./settingsStore";
import { useNotificationStore } from "./notificationStore";

function classifyUpdateError(err: unknown): string {
  const msg = err instanceof Error ? err.message : String(err);
  const lower = msg.toLowerCase();
  if (lower.includes("dns") || lower.includes("getaddr") || lower.includes("resolve")) {
    return "网络连接失败，请检查网络设置";
  }
  if (lower.includes("timed out") || lower.includes("timeout") || lower.includes("connection refused")) {
    return "无法连接到更新服务器";
  }
  if (lower.includes("404") || lower.includes("not found")) {
    return "未找到发布版本";
  }
  if (lower.includes("cert") || lower.includes("tls") || lower.includes("ssl")) {
    return "网络连接失败，请检查网络设置";
  }
  if (lower.includes("signature") || lower.includes("verify") || lower.includes("pubkey")) {
    return "更新签名校验失败";
  }
  return "未知错误";
}

type UpdaterStore = {
  updateState: UpdateState;
  pendingUpdate: Update | null;
  checkForUpdates: () => Promise<void>;
  downloadUpdate: () => Promise<void>;
  installUpdate: () => Promise<void>;
  startAutoUpdateCheck: () => void;
};

const INITIAL_STATE: UpdateState = {
  status: "idle",
  currentVersion: "...",
  latestVersion: null,
  downloadedVersion: null,
  contentLength: null,
  downloadedBytes: 0,
  error: null,
  availableUpdate: null,
};

let hasRunStartupCheck = false;

async function fetchAppVersion(): Promise<string> {
  try {
    return await getVersion();
  } catch {
    return "...";
  }
}

export const useUpdaterStore = create<UpdaterStore>((set, get) => ({
  updateState: { ...INITIAL_STATE },
  pendingUpdate: null,

  checkForUpdates: async () => {
    set({
      updateState: { ...get().updateState, status: "checking", error: null },
    });

    try {
      const result = await checkForAppUpdate();
      if (!result) {
        set({
          updateState: {
            ...get().updateState,
            status: "up-to-date",
          },
        });
        return;
      }

      const { update, summary } = result;
      set({
        updateState: {
          ...get().updateState,
          status: "available",
          currentVersion: summary.currentVersion,
          latestVersion: summary.version,
          availableUpdate: summary,
        },
        pendingUpdate: update,
      });
    } catch (err) {
      set({
        updateState: {
          ...get().updateState,
          status: "error",
          error: classifyUpdateError(err),
        },
      });
    }
  },

  downloadUpdate: async () => {
    const { pendingUpdate } = get();
    if (!pendingUpdate) {
      useNotificationStore.getState().showError("没有可用的更新");
      return;
    }

    set({
      updateState: {
        ...get().updateState,
        status: "downloading",
        downloadedBytes: 0,
        contentLength: null,
      },
    });

    try {
      const onEvent = (event: DownloadEvent) => {
        const current = get().updateState;
        switch (event.event) {
          case "Started":
            set({
              updateState: {
                ...current,
                contentLength: event.data.contentLength ?? null,
              },
            });
            break;
          case "Progress":
            set({
              updateState: {
                ...get().updateState,
                downloadedBytes: get().updateState.downloadedBytes + event.data.chunkLength,
              },
            });
            break;
          case "Finished":
            break;
        }
      };

      const summary = await downloadAppUpdate(pendingUpdate, onEvent);
      set({
        updateState: {
          ...get().updateState,
          status: "downloaded",
          downloadedVersion: summary.version,
        },
      });
      useNotificationStore.getState().showSuccess("更新下载完成");
    } catch (err) {
      set({
        updateState: {
          ...get().updateState,
          status: "error",
          error: classifyUpdateError(err),
        },
      });
      useNotificationStore.getState().showError("下载更新失败");
    }
  },

  installUpdate: async () => {
    const { pendingUpdate } = get();
    if (!pendingUpdate) return;

    try {
      await installAppUpdate(pendingUpdate);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      useNotificationStore.getState().showError(`安装更新失败: ${message}`);
    }
  },

  startAutoUpdateCheck: () => {
    if (hasRunStartupCheck) return;

    const settings = useSettingsStore.getState().settings;
    if (!settings.autoUpdateEnabled) return;

    hasRunStartupCheck = true;

    get()
      .checkForUpdates()
      .then(() => {
        const state = get().updateState;
        if (state.status === "available" && get().pendingUpdate) {
          return get().downloadUpdate();
        }
      })
      .catch((e) => {
        console.warn("Auto update check/download failed:", e);
      });
  },
}));

// Fetch current version immediately
fetchAppVersion()
  .then((ver) => {
    useUpdaterStore.setState((s) => ({
      updateState: { ...s.updateState, currentVersion: ver },
    }));
  })
  .catch(() => {});
