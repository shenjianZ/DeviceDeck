import { create } from "zustand";
import type { AppLog } from "../types";
import { tauriApi, type PaginatedLogs } from "../lib/tauri";
import type { UnlistenFn } from "@tauri-apps/api/event";

interface LogStore {
  // 分页数据
  logs: AppLog[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
  isLoading: boolean;

  // 过滤器
  sourceFilter: string;
  levelFilter: string;

  // 实时监听
  isListening: boolean;

  // 方法
  loadLogs: (limit?: number) => Promise<void>;
  loadPaginatedLogs: (page?: number) => Promise<void>;
  nextPage: () => Promise<void>;
  prevPage: () => Promise<void>;
  goToPage: (page: number) => Promise<void>;
  addLog: (log: AppLog) => void;
  clearLogs: () => Promise<void>;
  setFilter: (source?: string, level?: string) => void;
  startListening: () => Promise<UnlistenFn>;
}

export const useLogStore = create<LogStore>((set, get) => ({
  // 分页数据
  logs: [],
  total: 0,
  page: 1,
  pageSize: 50,
  totalPages: 0,
  isLoading: false,

  // 过滤器
  sourceFilter: "all",
  levelFilter: "all",

  // 实时监听
  isListening: false,

  // 加载日志（用于兼容旧代码）
  loadLogs: async (limit = 500) => {
    const logs = await tauriApi.getRecentLogs(limit);
    set({ logs });
  },

  // 分页加载日志
  loadPaginatedLogs: async (page?: number) => {
    const { page: currentPage, pageSize, sourceFilter, levelFilter } = get();
    const targetPage = page ?? currentPage;

    set({ isLoading: true });
    try {
      const result: PaginatedLogs = await tauriApi.getLogsPaginated(
        targetPage,
        pageSize,
        sourceFilter,
        levelFilter
      );
      set({
        logs: result.logs,
        total: result.total,
        page: result.page,
        pageSize: result.page_size,
        totalPages: result.total_pages,
      });
    } catch (error) {
      console.error("Failed to load paginated logs:", error);
    } finally {
      set({ isLoading: false });
    }
  },

  // 下一页
  nextPage: async () => {
    const { page, totalPages } = get();
    if (page < totalPages) {
      await get().loadPaginatedLogs(page + 1);
    }
  },

  // 上一页
  prevPage: async () => {
    const { page } = get();
    if (page > 1) {
      await get().loadPaginatedLogs(page - 1);
    }
  },

  // 跳转到指定页
  goToPage: async (targetPage: number) => {
    const { totalPages } = get();
    if (targetPage >= 1 && targetPage <= totalPages) {
      await get().loadPaginatedLogs(targetPage);
    }
  },

  // 添加新日志（实时监听用）
  addLog: (log) =>
    set((state) => {
      // 如果当前在第一页，添加到列表顶部
      if (state.page === 1) {
        const newLogs = [log, ...state.logs];
        // 如果超过页面大小，移除最后一个
        if (newLogs.length > state.pageSize) {
          newLogs.pop();
        }
        return {
          logs: newLogs,
          total: state.total + 1,
        };
      }
      // 不在第一页时只更新总数
      return { total: state.total + 1 };
    }),

  clearLogs: async () => {
    await tauriApi.clearLogs();
    set({ logs: [], total: 0, page: 1, totalPages: 0 });
  },

  // 设置过滤器
  setFilter: async (source, level) => {
    const updates: Partial<LogStore> = {};
    if (source !== undefined) updates.sourceFilter = source;
    if (level !== undefined) updates.levelFilter = level;

    set(updates);
    // 过滤器变化后重新加载第一页
    set({ page: 1 });
    await get().loadPaginatedLogs(1);
  },

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
