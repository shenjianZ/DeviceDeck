import { create } from "zustand";
import { getVersion } from "@tauri-apps/api/app";
import { checkForAppUpdate, downloadAppUpdate, installAppUpdate } from "../lib/updater";
import type { UpdateState } from "../lib/update-types";
import type { Update, DownloadEvent } from "@tauri-apps/plugin-updater";
import { useSettingsStore } from "./settingsStore";
import { useNotificationStore } from "./notificationStore";
import i18n from "../i18n";

function classifyUpdateError(err: unknown): string {
  const msg = err instanceof Error ? err.message : String(err);
  const lower = msg.toLowerCase();
  if (lower.includes("dns") || lower.includes("getaddr") || lower.includes("resolve")) {
    return i18n.t("common:updater.dnsError");
  }
  if (lower.includes("timed out") || lower.includes("timeout") || lower.includes("connection refused")) {
    return i18n.t("common:updater.serverUnreachable");
  }
  if (lower.includes("404") || lower.includes("not found")) {
    return i18n.t("common:updater.releaseNotFound");
  }
  if (lower.includes("cert") || lower.includes("tls") || lower.includes("ssl")) {
    return i18n.t("common:updater.dnsError");
  }
  if (lower.includes("signature") || lower.includes("verify") || lower.includes("pubkey")) {
    return i18n.t("common:updater.signatureFailed");
  }
  return i18n.t("common:updater.unknownError");
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
      useNotificationStore.getState().showError(i18n.t("common:notifications.noUpdateAvailable"));
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
      useNotificationStore.getState().showSuccess(i18n.t("common:notifications.updateDownloaded"));
    } catch (err) {
      set({
        updateState: {
          ...get().updateState,
          status: "error",
          error: classifyUpdateError(err),
        },
      });
      useNotificationStore.getState().showError(i18n.t("common:notifications.updateDownloadFailed"));
    }
  },

  installUpdate: async () => {
    const { pendingUpdate } = get();
    if (!pendingUpdate) return;

    try {
      await installAppUpdate(pendingUpdate);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      useNotificationStore.getState().showError(`${i18n.t("common:notifications.updateInstallFailed")}: ${message}`);
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
