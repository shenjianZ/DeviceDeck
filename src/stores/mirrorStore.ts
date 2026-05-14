import { create } from "zustand";
import { tauriApi } from "../lib/tauri";
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

  applyPreset: (config) => set({ config: { ...config } }),

  startMirror: async (serial) => {
    set({ isStarting: true, error: null });
    try {
      await tauriApi.startMirror(serial, get().config);
      await get().refreshSessions();
      set({ isStarting: false });
    } catch (e: unknown) {
      set({ error: e as AppError, isStarting: false });
    }
  },

  startWirelessMirror: async (serial, port = 5555) => {
    set({ isStarting: true, error: null });
    try {
      await tauriApi.startWirelessMirror(serial, get().config, port);
      await get().refreshSessions();
      set({ isStarting: false });
    } catch (e: unknown) {
      set({ error: e as AppError, isStarting: false });
    }
  },

  connectWirelessAndStartMirror: async (host, port = 5555) => {
    set({ isStarting: true, error: null });
    try {
      await tauriApi.connectWirelessAndStartMirror(host, port, get().config);
      await get().refreshSessions();
      set({ isStarting: false });
    } catch (e: unknown) {
      set({ error: e as AppError, isStarting: false });
    }
  },

  stopMirror: async (sessionId) => {
    set({ isStopping: sessionId, error: null });
    try {
      await tauriApi.stopMirror(sessionId);
      await get().refreshSessions();
      set({ isStopping: null });
    } catch (e: unknown) {
      set({ error: e as AppError, isStopping: null });
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
