import { create } from "zustand";
import { tauriApi } from "../lib/tauri";
import { useNotificationStore } from "./notificationStore";
import { applyTheme } from "../lib/theme";
import { defaultAdvancedMirrorConfig } from "../lib/presets";
import i18n from "../i18n";
import type { AppSettings, AppError } from "../types";

interface SettingsStore {
  settings: AppSettings;
  isLoading: boolean;
  error: AppError | null;

  loadSettings: () => Promise<void>;
  updateSettings: (settings: AppSettings) => Promise<void>;
  resetSettings: () => Promise<void>;
  updateSetting: <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => Promise<void>;
}

const DEFAULT_SETTINGS: AppSettings = {
  useBundledAdb: true,
  useBundledScrcpy: true,
  customAdbPath: "",
  customScrcpyPath: "",
  defaultMirrorConfig: {
    ...defaultAdvancedMirrorConfig(),
    maxSize: "1080",
    videoBitRate: "8M",
    maxFps: "60",
    videoCodec: "h264",
    noControl: false,
    stayAwake: true,
    turnScreenOff: false,
    screenBlackMode: false,
  },
  lastMirrorConfig: null,
  theme: "dark",
  logRetentionDays: 7,
  autoScanDevices: true,
  deviceScanIntervalSeconds: 30,
  fontSize: 14,
  locale: "zh-CN",
  autoStart: false,
  autoUpdateEnabled: true,
  firstRun: true,
  wifiUploadDir: "",
  wifiMaxUploadGB: 10,
  wifiChunkSizeMB: 16,
  wifiUploadConcurrency: 2,
};

function applyFontSize(fontSize: number) {
  const root = document.documentElement;
  root.style.setProperty("--font-size-base", `${fontSize}px`);
  root.style.setProperty("--font-size-xs", `${Math.max(11, fontSize - 2)}px`);
  root.style.setProperty("--font-size-sm", `${Math.max(12, fontSize - 1)}px`);
  root.style.setProperty("--font-size-md", `${fontSize}px`);
  root.style.setProperty("--font-size-lg", `${fontSize + 2}px`);
  root.style.setProperty("--font-size-xl", `${fontSize + 5}px`);
  root.style.setProperty("--font-size-2xl", `${fontSize + 9}px`);
  document.body.style.fontSize = `${fontSize}px`;
}

function applySettingSideEffect(key: keyof AppSettings, value: AppSettings[keyof AppSettings]) {
  switch (key) {
    case "theme":
      applyTheme(value as "dark" | "light");
      break;
    case "fontSize":
      if (typeof value === "number") applyFontSize(value);
      break;
    case "autoStart":
      tauriApi.setAutostart(value as boolean).catch(() => {});
      break;
  }
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  settings: { ...DEFAULT_SETTINGS },
  isLoading: false,
  error: null,

  loadSettings: async () => {
    set({ isLoading: true });
    try {
      const settings = await tauriApi.getSettings();
      set({ settings, isLoading: false });
      if (settings.theme) {
        applyTheme(settings.theme as "dark" | "light");
      }
      if (settings.fontSize) {
        applyFontSize(settings.fontSize);
      }
    } catch (e: unknown) {
      set({ error: e as AppError, isLoading: false });
    }
  },

  updateSettings: async (settings) => {
    try {
      await tauriApi.updateSettings(settings);
      set({ settings, error: null });
      if (settings.fontSize) {
        applyFontSize(settings.fontSize);
      }
      useNotificationStore.getState().showSuccess(i18n.t("common:toast.saveSuccess"));
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err });
      useNotificationStore.getState().showError(i18n.t("common:toast.saveFailed"), err.message, err.suggestion);
    }
  },

  resetSettings: async () => {
    try {
      const settings = await tauriApi.resetSettings();
      set({ settings, error: null });
      if (settings.fontSize) {
        applyFontSize(settings.fontSize);
      }
      useNotificationStore.getState().showSuccess(i18n.t("common:toast.resetSuccess"));
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err });
      useNotificationStore.getState().showError(i18n.t("common:toast.resetFailed"), err.message, err.suggestion);
    }
  },

  updateSetting: async (key, value) => {
    const current = get().settings;
    const updated = { ...current, [key]: value };
    try {
      await tauriApi.updateSettings(updated);
      set({ settings: updated, error: null });
      applySettingSideEffect(key, value);
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err });
      useNotificationStore.getState().showError(i18n.t("common:toast.saveFailed"), err.message, err.suggestion);
    }
  },
}));
