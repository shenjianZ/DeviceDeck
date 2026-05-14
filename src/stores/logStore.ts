import { create } from "zustand";
import type { AppLog } from "../types";
import { tauriApi } from "../lib/tauri";
import type { UnlistenFn } from "@tauri-apps/api/event";

interface LogStore {
  logs: AppLog[];
  sourceFilter: string;
  levelFilter: string;
  isListening: boolean;

  loadLogs: (limit?: number) => Promise<void>;
  addLog: (log: AppLog) => void;
  clearLogs: () => Promise<void>;
  setFilter: (source?: string, level?: string) => void;
  startListening: () => Promise<UnlistenFn>;
}

const mergeLogs = (logs: AppLog[]) => {
  const byId = new Map<string, AppLog>();
  logs.forEach((log) => byId.set(log.id, log));
  return Array.from(byId.values()).sort((a, b) => a.time - b.time);
};

export const useLogStore = create<LogStore>((set, get) => ({
  logs: [],
  sourceFilter: "all",
  levelFilter: "all",
  isListening: false,

  loadLogs: async (limit = 500) => {
    const logs = await tauriApi.getRecentLogs(limit);
    set((state) => ({ logs: mergeLogs([...state.logs, ...logs]) }));
  },

  addLog: (log) =>
    set((state) => {
      if (state.logs.some((item) => item.id === log.id)) {
        return state;
      }
      return { logs: [...state.logs, log] };
    }),

  clearLogs: async () => {
    await tauriApi.clearLogs();
    set({ logs: [] });
  },


  setFilter: (source, level) =>
    set(() => ({
      ...(source !== undefined ? { sourceFilter: source } : {}),
      ...(level !== undefined ? { levelFilter: level } : {}),
    })),

  startListening: async () => {
    if (get().isListening) {
      return () => undefined;
    }
    set({ isListening: true });
    const unlisten = await tauriApi.onLog((log) => {
      get().addLog(log);
    });
    return () => {
      set({ isListening: false });
      unlisten();
    };
  },
}));
