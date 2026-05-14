import { create } from "zustand";
import { tauriApi } from "../lib/tauri";
import { useNotificationStore } from "./notificationStore";
import type { MirrorConfig, MirrorSession, AppError } from "../types";
import type { UnlistenFn } from "@tauri-apps/api/event";

interface MirrorStore {
  config: MirrorConfig;
  sessions: MirrorSession[];
  isStarting: boolean;
  isStopping: string | null;
  error: AppError | null;

  updateConfig: (patch: Partial<MirrorConfig>) => void;
  applyPreset: (config: MirrorConfig) => void;
  startMirror: (serial: string) => Promise<void>;
  startWirelessMirror: (serial: string, port?: number) => Promise<void>;
  connectWirelessAndStartMirror: (host: string, port?: number) => Promise<void>;
  stopMirror: (sessionId: string) => Promise<void>;
  refreshSessions: () => Promise<void>;
  startListening: () => Promise<UnlistenFn>;
}

const DEFAULT_CONFIG: MirrorConfig = {
  maxSize: "1080",
  videoBitRate: "8M",
  maxFps: "60",
  videoCodec: "h264",
  noControl: false,
  stayAwake: true,
  turnScreenOff: false,
};

export const useMirrorStore = create<MirrorStore>((set, get) => ({
  config: { ...DEFAULT_CONFIG },
  sessions: [],
  isStarting: false,
  isStopping: null,
  error: null,

  updateConfig: (patch) =>
    set((state) => ({ config: { ...state.config, ...patch } })),

  applyPreset: (config) =>
    set((state) => ({
      config: {
        ...state.config,
        maxSize: config.maxSize,
        videoBitRate: config.videoBitRate,
        maxFps: config.maxFps,
        videoCodec: config.videoCodec,
      },
    })),

  startMirror: async (serial) => {
    set({ isStarting: true, error: null });
    try {
      await tauriApi.startMirror(serial, get().config);
      await get().refreshSessions();
      set({ isStarting: false });
      useNotificationStore.getState().showSuccess("投屏已启动", `设备 ${serial}`);
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isStarting: false });
      useNotificationStore.getState().showError("投屏启动失败", err.message, err.suggestion);
    }
  },

  startWirelessMirror: async (serial, port = 5555) => {
    set({ isStarting: true, error: null });
    try {
      await tauriApi.startWirelessMirror(serial, get().config, port);
      await get().refreshSessions();
      set({ isStarting: false });
      useNotificationStore.getState().showSuccess("无线投屏已启动", `设备 ${serial}`);
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isStarting: false });
      useNotificationStore.getState().showError("无线投屏启动失败", err.message, err.suggestion);
    }
  },

  connectWirelessAndStartMirror: async (host, port = 5555) => {
    set({ isStarting: true, error: null });
    try {
      await tauriApi.connectWirelessAndStartMirror(host, port, get().config);
      await get().refreshSessions();
      set({ isStarting: false });
      useNotificationStore.getState().showSuccess("无线连接并投屏成功", `${host}:${port}`);
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isStarting: false });
      useNotificationStore.getState().showError("无线连接失败", err.message, err.suggestion);
    }
  },

  stopMirror: async (sessionId) => {
    set({ isStopping: sessionId, error: null });
    try {
      await tauriApi.stopMirror(sessionId);
      await get().refreshSessions();
      set({ isStopping: null });
      useNotificationStore.getState().showSuccess("投屏已停止");
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isStopping: null });
      useNotificationStore.getState().showError("停止投屏失败", err.message, err.suggestion);
    }
  },

  refreshSessions: async () => {
    try {
      const sessions = await tauriApi.listMirrorSessions();
      set({ sessions });
    } catch (_e) {
      // silent
    }
  },

  startListening: async () => {
    const unlisten = await tauriApi.onSessionUpdated(() => {
      get().refreshSessions();
    });
    return unlisten;
  },
}));
