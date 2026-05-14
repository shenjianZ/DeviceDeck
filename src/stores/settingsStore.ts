import { create } from "zustand";
import { tauriApi } from "../lib/tauri";
import type { AppSettings, AppError } from "../types";

interface SettingsStore {
  settings: AppSettings;
  isLoading: boolean;
  error: AppError | null;

  loadSettings: () => Promise<void>;
  updateSettings: (settings: AppSettings) => Promise<void>;
  resetSettings: () => Promise<void>;
}

const DEFAULT_SETTINGS: AppSettings = {
  useBundledAdb: true,
  useBundledScrcpy: true,
  customAdbPath: "",
  customScrcpyPath: "",
  defaultMirrorConfig: {
    maxSize: "1080",
    videoBitRate: "8M",
    maxFps: "60",
    noControl: false,
    stayAwake: true,
    turnScreenOff: false,
  },
  lastMirrorConfig: null,
  theme: "dark",
  logRetentionDays: 7,
};

export const useSettingsStore = create<SettingsStore>((set) => ({
  settings: { ...DEFAULT_SETTINGS },
  isLoading: false,
  error: null,

  loadSettings: async () => {
    set({ isLoading: true });
    try {
      const settings = await tauriApi.getSettings();
      set({ settings, isLoading: false });
    } catch (e: unknown) {
      set({ error: e as AppError, isLoading: false });
    }
  },

  updateSettings: async (settings) => {
    try {
      await tauriApi.updateSettings(settings);
      set({ settings, error: null });
    } catch (e: unknown) {
      set({ error: e as AppError });
    }
  },

  resetSettings: async () => {
    try {
      const settings = await tauriApi.resetSettings();
      set({ settings, error: null });
    } catch (e: unknown) {
      set({ error: e as AppError });
    }
  },
}));
