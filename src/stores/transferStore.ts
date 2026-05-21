import { create } from "zustand";
import { tauriApi } from "../lib/tauri";
import { useNotificationStore } from "./notificationStore";
import i18n from "../i18n";
import type { FileEntry, WifiTransferStatus, AppError, TransferProgress } from "../types";

const DEFAULT_PATH = "/sdcard";
const cancelledTransferIds = new Set<string>();

export type SortField = "name" | "size" | "modified";
export type SortDirection = "asc" | "desc";

interface TransferStore {
  // USB mode
  currentPath: string;
  files: FileEntry[];
  selectedFiles: Set<string>;
  isLoading: boolean;
  isTransferring: boolean;
  transferOperationIds: Set<string>;

  // Sort
  sortField: SortField;
  sortDirection: SortDirection;
  sortedFiles: FileEntry[];

  // Transfer progress
  activeTransfers: Map<string, TransferProgress>;

  // Wi-Fi mode
  wifiStatus: WifiTransferStatus | null;
  receivedFiles: string[];
  isWifiBusy: boolean;

  // Actions
  listDirectory: (serial: string, path?: string) => Promise<void>;
  navigateToDirectory: (serial: string, path: string) => Promise<void>;
  navigateUp: (serial: string) => Promise<void>;
  selectSingle: (path: string) => void;
  toggleFileSelection: (path: string) => void;
  clearSelection: () => void;
  selectAll: () => void;
  pullFile: (serial: string, remotePath: string, localDirectory: string) => Promise<void>;
  pushToDirectory: (serial: string, localPath: string) => Promise<void>;
  deleteFile: (serial: string, path: string) => Promise<void>;
  refreshDirectory: (serial: string) => Promise<void>;
  createDirectory: (serial: string, folderName: string) => Promise<void>;
  createFile: (serial: string, fileName: string) => Promise<void>;
  cancelTransfer: (id: string) => Promise<void>;
  setSort: (field: SortField) => void;

  startWifiTransfer: (port?: number) => Promise<void>;
  stopWifiTransfer: () => Promise<void>;
  loadWifiStatus: () => Promise<void>;
}

export const useTransferStore = create<TransferStore>((set, get) => ({
  currentPath: DEFAULT_PATH,
  files: [],
  selectedFiles: new Set(),
  isLoading: false,
  isTransferring: false,
  transferOperationIds: new Set(),
  sortField: "name" as SortField,
  sortDirection: "asc" as SortDirection,
  sortedFiles: [],
  activeTransfers: new Map<string, TransferProgress>(),
  wifiStatus: null,
  receivedFiles: [],
  isWifiBusy: false,

  setSort: (field) => {
    set((state) => {
      const direction = state.sortField === field && state.sortDirection === "asc" ? "desc" : "asc";
      return {
        sortField: field,
        sortDirection: direction,
        sortedFiles: sortFiles(state.files, field, direction),
      };
    });
  },

  listDirectory: async (serial, path) => {
    const targetPath = path ?? DEFAULT_PATH;
    set({ isLoading: true });
    try {
      const files = await tauriApi.listDeviceDirectory(serial, targetPath);
      set((state) => ({
        files,
        currentPath: targetPath,
        selectedFiles: new Set(),
        isLoading: false,
        sortedFiles: sortFiles(files, state.sortField, state.sortDirection),
      }));
    } catch (e: unknown) {
      const err = e as AppError;
      set({ isLoading: false });
      useNotificationStore.getState().showError(i18n.t("transfer:listFailed"), err.detail || err.message, err.suggestion);
    }
  },

  navigateToDirectory: async (serial, path) => {
    await get().listDirectory(serial, path);
  },

  navigateUp: async (serial) => {
    const { currentPath } = get();
    if (currentPath === "/") return;
    const parts = currentPath.split("/").filter(Boolean);
    parts.pop();
    const parentPath = parts.length === 0 ? "/" : "/" + parts.join("/");
    await get().listDirectory(serial, parentPath);
  },

  selectSingle: (path) => set({ selectedFiles: new Set([path]) }),

  toggleFileSelection: (path) => {
    set((state) => {
      const next = new Set(state.selectedFiles);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
      }
      return { selectedFiles: next };
    });
  },

  clearSelection: () => set({ selectedFiles: new Set() }),

  selectAll: () => {
    const files = get().files;
    set({ selectedFiles: new Set(files.map((f) => f.path)) });
  },

  pullFile: async (serial, remotePath, localDirectory) => {
    const operationId = `pull:${serial}:${remotePath}:${Date.now()}`;
    beginTransferOperation(set, operationId);
    try {
      const result = await tauriApi.pullDeviceFileStreaming(serial, remotePath, localDirectory);
      useNotificationStore.getState().showSuccess(
        i18n.t("transfer:pullSuccess"),
        result.outputPath || result.message
      );
    } catch (e: unknown) {
      const err = e as AppError;
      if (err.code === "TRANSFER_CANCELLED") {
        useNotificationStore.getState().showInfo(i18n.t("transfer:transferCancelled"));
      } else {
        useNotificationStore.getState().showError(i18n.t("transfer:pullFailed"), err.detail || err.message, err.suggestion);
      }
    } finally {
      endTransferOperation(set, operationId);
    }
  },

  pushToDirectory: async (serial, localPath) => {
    const { currentPath } = get();
    const operationId = `push:${serial}:${localPath}:${Date.now()}`;
    beginTransferOperation(set, operationId);
    try {
      await tauriApi.pushDeviceFileStreaming(serial, localPath, currentPath);
      useNotificationStore.getState().showSuccess(i18n.t("transfer:pushSuccess"), localPath);
      await get().listDirectory(serial, currentPath);
    } catch (e: unknown) {
      const err = e as AppError;
      if (err.code === "TRANSFER_CANCELLED") {
        useNotificationStore.getState().showInfo(i18n.t("transfer:transferCancelled"));
      } else {
        useNotificationStore.getState().showError(i18n.t("transfer:pushFailed"), err.detail || err.message, err.suggestion);
      }
    } finally {
      endTransferOperation(set, operationId);
    }
  },

  deleteFile: async (serial, path) => {
    const operationId = `delete:${serial}:${path}:${Date.now()}`;
    beginTransferOperation(set, operationId);
    try {
      await tauriApi.deleteDeviceFile(serial, path);
      useNotificationStore.getState().showSuccess(i18n.t("transfer:deleteSuccess"), path);
      await get().refreshDirectory(serial);
    } catch (e: unknown) {
      const err = e as AppError;
      useNotificationStore.getState().showError(i18n.t("transfer:deleteFailed"), err.detail || err.message, err.suggestion);
    } finally {
      endTransferOperation(set, operationId);
    }
  },

  refreshDirectory: async (serial) => {
    await get().listDirectory(serial, get().currentPath);
  },

  createDirectory: async (serial, folderName) => {
    const { currentPath } = get();
    const fullPath = currentPath === "/" ? `/${folderName}` : `${currentPath}/${folderName}`;
    try {
      await tauriApi.createDeviceDirectory(serial, fullPath);
      useNotificationStore.getState().showSuccess(i18n.t("transfer:folderCreated"), fullPath);
      await get().refreshDirectory(serial);
    } catch (e: unknown) {
      const err = e as AppError;
      useNotificationStore.getState().showError(i18n.t("transfer:folderCreateFailed"), err.detail || err.message, err.suggestion);
    }
  },

  createFile: async (serial, fileName) => {
    const { currentPath } = get();
    const fullPath = currentPath === "/" ? `/${fileName}` : `${currentPath}/${fileName}`;
    try {
      await tauriApi.createDeviceFile(serial, fullPath);
      useNotificationStore.getState().showSuccess(i18n.t("transfer:fileCreated"), fullPath);
      await get().refreshDirectory(serial);
    } catch (e: unknown) {
      const err = e as AppError;
      useNotificationStore.getState().showError(i18n.t("transfer:fileCreateFailed"), err.detail || err.message, err.suggestion);
    }
  },

  cancelTransfer: async (id) => {
    try {
      cancelledTransferIds.add(id);
      await tauriApi.cancelTransfer(id);
      set((state) => {
        const next = new Map(state.activeTransfers);
        next.delete(id);
        return { activeTransfers: next };
      });
      window.setTimeout(() => cancelledTransferIds.delete(id), 30_000);
      useNotificationStore.getState().showInfo(i18n.t("transfer:transferCancelling"));
    } catch (e: unknown) {
      cancelledTransferIds.delete(id);
      const err = e as AppError;
      useNotificationStore.getState().showError(i18n.t("transfer:cancelFailed"), err.detail || err.message, err.suggestion);
    }
  },

  startWifiTransfer: async (port) => {
    set({ isWifiBusy: true });
    try {
      const status = await tauriApi.startWifiTransfer(port);
      set({ wifiStatus: status, isWifiBusy: false });
      useNotificationStore.getState().showSuccess(i18n.t("transfer:wifiStarted"), status.url || "");
    } catch (e: unknown) {
      const err = e as AppError;
      set({ isWifiBusy: false });
      useNotificationStore.getState().showError(i18n.t("transfer:wifiStartFailed"), err.detail || err.message, err.suggestion);
    }
  },

  stopWifiTransfer: async () => {
    set({ isWifiBusy: true });
    try {
      await tauriApi.stopWifiTransfer();
      set({
        wifiStatus: { running: false, port: 0 },
        isWifiBusy: false,
      });
      useNotificationStore.getState().showSuccess(i18n.t("transfer:wifiStopped"));
    } catch (e: unknown) {
      const err = e as AppError;
      set({ isWifiBusy: false });
      useNotificationStore.getState().showError(i18n.t("transfer:wifiStopFailed"), err.detail || err.message, err.suggestion);
    }
  },

  loadWifiStatus: async () => {
    try {
      const status = await tauriApi.getWifiTransferStatus();
      set({ wifiStatus: status });
    } catch {
      // ignore
    }
  },
}));

type TransferSet = (
  partial:
    | Partial<TransferStore>
    | ((state: TransferStore) => Partial<TransferStore>),
) => void;

function beginTransferOperation(set: TransferSet, operationId: string) {
  set((state) => {
    const next = new Set(state.transferOperationIds);
    next.add(operationId);
    return { transferOperationIds: next, isTransferring: next.size > 0 };
  });
}

function endTransferOperation(set: TransferSet, operationId: string) {
  set((state) => {
    const next = new Set(state.transferOperationIds);
    next.delete(operationId);
    return { transferOperationIds: next, isTransferring: next.size > 0 };
  });
}

// Listen for transfer progress events
tauriApi.onTransferProgress((progress) => {
  if (cancelledTransferIds.has(progress.id)) return;

  useTransferStore.setState((state) => {
    const next = new Map(state.activeTransfers);
    if (progress.percent >= 100) {
      next.set(progress.id, progress);
      window.setTimeout(() => {
        useTransferStore.setState((latest) => {
          const latestTransfers = new Map(latest.activeTransfers);
          const current = latestTransfers.get(progress.id);
          if (current && current.percent >= 100) {
            latestTransfers.delete(progress.id);
          }
          return { activeTransfers: latestTransfers };
        });
      }, 1600);
    } else {
      next.set(progress.id, progress);
    }
    return { activeTransfers: next };
  });
});

function sortFiles(files: FileEntry[], field: SortField, direction: SortDirection): FileEntry[] {
  const sorted = [...files];
  sorted.sort((a, b) => {
    // Directories always first
    if (a.isDirectory !== b.isDirectory) return a.isDirectory ? -1 : 1;
    const mul = direction === "asc" ? 1 : -1;
    switch (field) {
      case "size": {
        const sa = a.size ?? 0;
        const sb = b.size ?? 0;
        return mul * (sa - sb);
      }
      case "modified":
        return mul * (a.modified ?? "").localeCompare(b.modified ?? "");
      case "name":
      default:
        return mul * a.name.toLowerCase().localeCompare(b.name.toLowerCase());
    }
  });
  return sorted;
}
