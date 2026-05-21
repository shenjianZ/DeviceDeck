import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import {
  ArrowDown,
  ArrowUp,
  ChevronRight,
  Copy,
  Download,
  File,
  FilePlus2,
  Folder,
  FolderOpen,
  FolderPlus,
  FolderSync,
  RefreshCw,
  Trash2,
  Upload,
  Wifi,
  WifiOff,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { useDeviceStore } from "../stores/deviceStore";
import { useTransferStore } from "../stores/transferStore";
import type { FileEntry, TransferProgress } from "../types";

type TransferMode = "usb" | "wifi";

interface ContextMenuState {
  x: number;
  y: number;
  target: "background" | "entry";
  entry?: FileEntry;
}

interface MenuItem {
  label: string;
  icon: LucideIcon;
  danger?: boolean;
  action: () => void;
}

interface DeleteRequest {
  paths: string[];
  names: string[];
}

type CreateMode = "file" | "folder";

export function FileTransferPage() {
  const { t } = useTranslation(["transfer", "common"]);
  const [mode, setMode] = useState<TransferMode>("usb");

  return (
    <div
      data-dd-app-shortcuts="file-transfer"
      style={{ display: "flex", flexDirection: "column", height: "100%" }}
    >
      <div className="action-bar" style={{ gap: 8, marginBottom: 16, flexShrink: 0 }}>
        <button
          className={`btn ${mode === "usb" ? "btn-p" : "btn-s"}`}
          onClick={() => setMode("usb")}
          type="button"
        >
          <FolderSync size={14} />
          {t("usbMode")}
        </button>
        <button
          className={`btn ${mode === "wifi" ? "btn-p" : "btn-s"}`}
          onClick={() => setMode("wifi")}
          type="button"
        >
          <Wifi size={14} />
          {t("wifiMode")}
        </button>
      </div>

      <div style={{ flex: 1, minHeight: 0, position: "relative" }}>
        <div style={{ display: mode === "usb" ? "contents" : "none" }}>
          <UsbTransferPanel />
        </div>
        <div style={{ display: mode === "wifi" ? "contents" : "none" }}>
          <WifiTransferPanel />
        </div>
      </div>
    </div>
  );
}

/* ======================== Context Menu ======================== */

function ContextMenu({
  items,
  x,
  y,
  onClose,
}: {
  items: MenuItem[];
  x: number;
  y: number;
  onClose: () => void;
}) {
  const [style, setStyle] = useState<React.CSSProperties>({ left: x, top: y });

  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (menuRef.current && menuRef.current.contains(e.target as Node)) return;
      onClose();
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [onClose]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [onClose]);

  useEffect(() => {
    const menu = document.querySelector(".ctx-menu") as HTMLElement | null;
    if (!menu) return;
    const rect = menu.getBoundingClientRect();
    const adjusted: React.CSSProperties = { left: x, top: y };
    if (rect.right > window.innerWidth - 8) adjusted.left = x - rect.width;
    if (rect.bottom > window.innerHeight - 8) adjusted.top = y - rect.height;
    if (adjusted.left !== x || adjusted.top !== y) setStyle(adjusted);
  }, [x, y]);

  return createPortal(
    <div ref={menuRef} className="ctx-menu" style={style} onClick={(e) => e.stopPropagation()}>
      {items.map((item, i) => {
        const Icon = item.icon;
        return (
          <button
            key={i}
            className={`ctx-item${item.danger ? " danger" : ""}`}
            type="button"
            onClick={() => { item.action(); onClose(); }}
          >
            <Icon size={14} />
            <span>{item.label}</span>
          </button>
        );
      })}
    </div>,
    document.body,
  );
}

function DeleteConfirmDialog({
  request,
  onCancel,
  onConfirm,
}: {
  request: DeleteRequest;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const { t } = useTranslation(["transfer", "common"]);
  const visibleNames = request.names.slice(0, 4);
  const extraCount = request.names.length - visibleNames.length;

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onCancel();
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [onCancel]);

  return createPortal(
    <div className="dialog-backdrop" role="presentation" onMouseDown={onCancel}>
      <div
        className="confirm-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="delete-dialog-title"
        onMouseDown={(e) => e.stopPropagation()}
      >
        <div className="confirm-dialog-title" id="delete-dialog-title">
          {t("deleteDialogTitle")}
        </div>
        <div className="confirm-dialog-message">
          {request.paths.length === 1
            ? t("deleteDialogSingle")
            : t("deleteDialogMulti", { count: request.paths.length })}
        </div>
        <ul className="confirm-dialog-list">
          {visibleNames.map((name, index) => (
            <li key={`${name}-${index}`}>{name}</li>
          ))}
          {extraCount > 0 && <li>{t("deleteDialogMore", { count: extraCount })}</li>}
        </ul>
        <div className="confirm-dialog-warning">{t("deleteDialogWarning")}</div>
        <div className="confirm-dialog-actions">
          <button className="btn btn-s" type="button" onClick={onCancel}>
            {t("common:buttons.cancel")}
          </button>
          <button className="btn btn-p danger-btn" type="button" onClick={onConfirm}>
            <Trash2 size={14} />
            {t("delete")}
          </button>
        </div>
      </div>
    </div>,
    document.body,
  );
}

/* ======================== USB Panel ======================== */

function UsbTransferPanel() {
  const { t } = useTranslation(["transfer", "common"]);
  const devices = useDeviceStore((s) => s.devices);
  const scanDevices = useDeviceStore((s) => s.scanDevices);
  const isScanning = useDeviceStore((s) => s.isScanning);
  const currentPath = useTransferStore((s) => s.currentPath);
  const files = useTransferStore((s) => s.files);
  const selectedFiles = useTransferStore((s) => s.selectedFiles);
  const isLoading = useTransferStore((s) => s.isLoading);
  const listDirectory = useTransferStore((s) => s.listDirectory);
  const navigateUp = useTransferStore((s) => s.navigateUp);
  const selectSingle = useTransferStore((s) => s.selectSingle);
  const toggleFileSelection = useTransferStore((s) => s.toggleFileSelection);
  const clearSelection = useTransferStore((s) => s.clearSelection);
  const pullFile = useTransferStore((s) => s.pullFile);
  const pushToDirectory = useTransferStore((s) => s.pushToDirectory);
  const deleteFile = useTransferStore((s) => s.deleteFile);
  const refreshDirectory = useTransferStore((s) => s.refreshDirectory);
  const sortedFiles = useTransferStore((s) => s.sortedFiles);
  const sortField = useTransferStore((s) => s.sortField);
  const sortDirection = useTransferStore((s) => s.sortDirection);
  const setSort = useTransferStore((s) => s.setSort);
  const selectAll = useTransferStore((s) => s.selectAll);
  const createDirectory = useTransferStore((s) => s.createDirectory);
  const createFile = useTransferStore((s) => s.createFile);
  const activeTransfers = useTransferStore((s) => s.activeTransfers);

  const [selectedDeviceSerial, setSelectedDeviceSerial] = useState<string | null>(null);
  const [pathInput, setPathInput] = useState(currentPath);
  const [ctx, setCtx] = useState<ContextMenuState | null>(null);
  const [isDragOver, setIsDragOver] = useState(false);
  const [createMode, setCreateMode] = useState<CreateMode | null>(null);
  const [newItemName, setNewItemName] = useState("");
  const [deleteRequest, setDeleteRequest] = useState<DeleteRequest | null>(null);

  const onlineDevices = useMemo(() => devices.filter((d) => d.status === "online"), [devices]);
  const selectedDevice = onlineDevices.find((d) => d.serial === selectedDeviceSerial) ?? null;

  // Refs to avoid stale closures in keyboard/drag handlers
  const selectedFilesRef = useRef(selectedFiles);
  selectedFilesRef.current = selectedFiles;
  const selectedDeviceRef = useRef(selectedDevice);
  selectedDeviceRef.current = selectedDevice;

  useEffect(() => { scanDevices(true); }, [scanDevices]);
  useEffect(() => { setPathInput(currentPath); }, [currentPath]);

  const handleSelectDevice = (serial: string) => {
    setSelectedDeviceSerial(serial);
    listDirectory(serial);
  };

  const handleNavigate = (serial: string, entry: FileEntry) => {
    if (entry.isDirectory) listDirectory(serial, entry.path);
  };

  const handlePathSubmit = (serial: string) => {
    if (pathInput.trim()) listDirectory(serial, pathInput.trim());
  };

  const handlePush = useCallback(async () => {
    if (!selectedDevice) return;
    const localPath = await open({ multiple: true });
    if (Array.isArray(localPath)) {
      for (const p of localPath) await pushToDirectory(selectedDevice.serial, p);
    } else if (typeof localPath === "string") {
      await pushToDirectory(selectedDevice.serial, localPath);
    }
  }, [selectedDevice, pushToDirectory]);

  const uploadPathsSequentially = useCallback(async (serial: string, paths: string[]) => {
    for (const path of paths) {
      await pushToDirectory(serial, path);
    }
  }, [pushToDirectory]);

  const handlePullEntry = useCallback(async (entry: FileEntry) => {
    if (!selectedDevice) return;
    const localDir = await open({ directory: true, multiple: false });
    if (typeof localDir === "string") {
      await pullFile(selectedDevice.serial, entry.path, localDir);
    }
  }, [selectedDevice, pullFile]);

  const handlePullSelected = useCallback(async () => {
    if (!selectedDevice || selectedFiles.size === 0) return;
    const localDir = await open({ directory: true, multiple: false });
    if (typeof localDir === "string") {
      for (const remotePath of selectedFiles) {
        await pullFile(selectedDevice.serial, remotePath, localDir);
      }
      clearSelection();
    }
  }, [selectedDevice, selectedFiles, pullFile, clearSelection]);

  const requestDelete = useCallback((entries: FileEntry[]) => {
    if (entries.length === 0) return;
    setDeleteRequest({
      paths: entries.map((entry) => entry.path),
      names: entries.map((entry) => entry.name || entry.path),
    });
  }, []);

  const handleDeleteEntry = useCallback((entry: FileEntry) => {
    requestDelete([entry]);
  }, [requestDelete]);

  const handleDeleteSelected = useCallback(() => {
    if (selectedFiles.size === 0) return;
    const selectedEntries = files.filter((entry) => selectedFiles.has(entry.path));
    requestDelete(selectedEntries.length > 0
      ? selectedEntries
      : [...selectedFiles].map((path) => ({ path, name: path, isDirectory: false } as FileEntry)));
  }, [files, selectedFiles, requestDelete]);

  const confirmDelete = useCallback(async () => {
    if (!selectedDevice || !deleteRequest) return;
    const paths = [...deleteRequest.paths];
    setDeleteRequest(null);
    for (const path of paths) {
      await deleteFile(selectedDevice.serial, path);
    }
    clearSelection();
  }, [selectedDevice, deleteRequest, deleteFile, clearSelection]);

  // Keyboard shortcuts (must be after handleDeleteSelected declaration)
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.tagName === "SELECT") return;
      const dev = selectedDeviceRef.current;
      if (!dev) return;

      if (e.key === "Delete" && selectedFilesRef.current.size > 0) {
        e.preventDefault();
        handleDeleteSelected();
      } else if (e.key === "a" && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        selectAll();
      } else if (e.key === "F5") {
        e.preventDefault();
        refreshDirectory(dev.serial);
      } else if (e.key === "Escape") {
        clearSelection();
      }
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [selectAll, clearSelection, refreshDirectory, handleDeleteSelected]);

  const handleContextMenu = useCallback((e: React.MouseEvent, entry?: FileEntry) => {
    e.preventDefault();
    if (entry && !selectedFilesRef.current.has(entry.path)) selectSingle(entry.path);
    setCtx({ x: e.clientX, y: e.clientY, target: entry ? "entry" : "background", entry });
  }, [selectSingle]);

  const closeCtx = useCallback(() => setCtx(null), []);

  const breadcrumbs = currentPath.split("/").filter(Boolean);

  // Tauri drag-drop: HTML5 events are intercepted by Tauri, use native API instead
  useEffect(() => {
    if (!selectedDevice) return;
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "over") {
        setIsDragOver(true);
      } else if (event.payload.type === "drop") {
        setIsDragOver(false);
        const paths = event.payload.paths;
        if (paths.length > 0) {
          void uploadPathsSequentially(selectedDevice.serial, paths);
        }
      } else {
        setIsDragOver(false);
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [selectedDevice, uploadPathsSequentially]);

  const openCreateInput = useCallback((mode: CreateMode) => {
    setCreateMode(mode);
    setNewItemName("");
  }, []);

  const handleCreateItem = useCallback(async () => {
    const name = newItemName.trim();
    if (!selectedDevice || !createMode || !name) return;
    if (createMode === "folder") {
      await createDirectory(selectedDevice.serial, name);
    } else {
      await createFile(selectedDevice.serial, name);
    }
    setCreateMode(null);
    setNewItemName("");
  }, [selectedDevice, createMode, newItemName, createDirectory, createFile]);

  const cancelCreateItem = useCallback(() => {
    setCreateMode(null);
    setNewItemName("");
  }, []);

  // Build context menu items
  const ctxItems = useMemo(() => {
    if (!ctx) return [];
    const items: MenuItem[] = [];

    if (ctx.target === "entry" && ctx.entry) {
      const entry = ctx.entry;
      if (entry.isDirectory) {
        items.push({ label: t("open"), icon: FolderOpen, action: () => selectedDevice && handleNavigate(selectedDevice.serial, entry) });
        items.push({ label: t("pullFolder"), icon: Download, action: () => handlePullEntry(entry) });
        items.push({ label: t("delete"), icon: Trash2, danger: true, action: () => handleDeleteEntry(entry) });
      } else {
        items.push({ label: t("pull"), icon: Download, action: () => handlePullEntry(entry) });
        items.push({ label: t("delete"), icon: Trash2, danger: true, action: () => handleDeleteEntry(entry) });
      }
    } else {
      items.push({ label: t("newFile"), icon: FilePlus2, action: () => openCreateInput("file") });
      items.push({ label: t("newFolder"), icon: FolderPlus, action: () => openCreateInput("folder") });
      items.push({ label: t("push"), icon: Upload, action: handlePush });
      items.push({ label: t("refresh"), icon: RefreshCw, action: () => selectedDevice && refreshDirectory(selectedDevice.serial) });
      if (files.length > 0) {
        items.push({ label: t("selectAll"), icon: File, action: selectAll });
      }
      if (selectedFiles.size > 0) {
        items.push({ label: t("pullSelected", { count: selectedFiles.size }), icon: Download, action: handlePullSelected });
        items.push({ label: t("deleteSelected", { count: selectedFiles.size }), icon: Trash2, danger: true, action: handleDeleteSelected });
      }
    }
    return items;
  }, [
    ctx,
    t,
    selectedDevice,
    files.length,
    selectedFiles,
    handlePush,
    handlePullEntry,
    handleDeleteEntry,
    handlePullSelected,
    handleDeleteSelected,
    openCreateInput,
    refreshDirectory,
    selectAll,
  ]);

  if (onlineDevices.length === 0) {
    return (
      <div className="empty">
        <FolderSync size={32} />
        <span>{t("noDevice")}</span>
        <button className="btn btn-s" onClick={() => scanDevices()} disabled={isScanning} type="button">
          <RefreshCw size={14} className={isScanning ? "spin" : ""} />
          {isScanning ? t("common:buttons.scanning") : t("common:buttons.scan")}
        </button>
      </div>
    );
  }

  return (
    <div
      data-dd-context-menu="file-transfer"
      style={{ display: "flex", flexDirection: "column", height: "100%" }}
    >
      {/* Device selector */}
      <div className="action-bar" style={{ marginBottom: 8, gap: 8, flexShrink: 0 }}>
        <select
          className="inp"
          value={selectedDeviceSerial ?? ""}
          onChange={(e) => handleSelectDevice(e.target.value)}
          style={{ minWidth: 200 }}
        >
          <option value="" disabled>{t("selectDevice")}</option>
          {onlineDevices.map((d) => (
            <option key={d.serial} value={d.serial}>
              {d.name || d.model || d.serial}
            </option>
          ))}
        </select>
        <button className="btn btn-g" onClick={() => scanDevices()} disabled={isScanning} type="button">
          <RefreshCw size={14} className={isScanning ? "spin" : ""} />
        </button>
        {selectedDevice && (
          <div style={{ marginLeft: "auto" }}>
            <button className="btn btn-s transfer-upload-btn" onClick={handlePush} type="button">
              <Upload size={14} />
              {t("push")}
            </button>
          </div>
        )}
      </div>

      {selectedDevice && (
        <div style={{ display: "flex", flexDirection: "column", flex: 1, minHeight: 0 }}>
          {/* Breadcrumb */}
          <div className="action-bar" style={{ marginBottom: 6, gap: 6, flexShrink: 0 }}>
            <button className="btn btn-g" onClick={() => navigateUp(selectedDevice.serial)} type="button" title={t("goUp")}>
              <ArrowUp size={14} />
            </button>
            <div style={{ display: "flex", gap: 2, flexWrap: "wrap", alignItems: "center", flex: 1, minWidth: 0 }}>
              <button
                className="btn btn-g"
                style={{ padding: "2px 6px", fontSize: 11 }}
                onClick={() => listDirectory(selectedDevice.serial, "/")}
                type="button"
              >
                /
              </button>
              {breadcrumbs.map((part, idx) => {
                const fullPath = "/" + breadcrumbs.slice(0, idx + 1).join("/");
                return (
                  <span key={fullPath} style={{ display: "flex", alignItems: "center", gap: 2 }}>
                    <ChevronRight size={12} style={{ color: "var(--t3)" }} />
                    <button
                      className="btn btn-g"
                      style={{ padding: "2px 6px", fontSize: 11 }}
                      onClick={() => listDirectory(selectedDevice.serial, fullPath)}
                      type="button"
                    >
                      {part}
                    </button>
                  </span>
                );
              })}
            </div>
          </div>

          {/* Path input */}
          <div className="row" style={{ marginBottom: 8, gap: 6, flexShrink: 0 }}>
            <input
              className="inp mono"
              value={pathInput}
              onChange={(e) => setPathInput(e.target.value)}
              onKeyDown={(e) => { if (e.key === "Enter") handlePathSubmit(selectedDevice.serial); }}
              style={{ flex: 1, fontSize: 12 }}
            />
            <button className="btn btn-g" onClick={() => handlePathSubmit(selectedDevice.serial)} type="button">
              <ChevronRight size={14} />
            </button>
            <button
              className="btn btn-g"
              onClick={() => refreshDirectory(selectedDevice.serial)}
              disabled={isLoading}
              type="button"
              title={t("refresh")}
            >
              <RefreshCw size={14} className={isLoading ? "spin" : ""} />
            </button>
          </div>

          {/* New item input */}
          {createMode && (
            <div className="row" style={{ marginBottom: 8, gap: 6, flexShrink: 0 }}>
              {createMode === "folder" ? (
                <FolderPlus size={14} style={{ color: "var(--acc)", flexShrink: 0 }} />
              ) : (
                <FilePlus2 size={14} style={{ color: "var(--acc)", flexShrink: 0 }} />
              )}
              <input
                className="inp"
                value={newItemName}
                onChange={(e) => setNewItemName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleCreateItem();
                  if (e.key === "Escape") cancelCreateItem();
                }}
                placeholder={createMode === "folder" ? t("newFolderPlaceholder") : t("newFilePlaceholder")}
                style={{ flex: 1, fontSize: 13 }}
                autoFocus
              />
              <button className="btn btn-p" style={{ padding: "2px 12px", fontSize: 12 }} onClick={handleCreateItem} type="button">
                {t("common:buttons.confirm")}
              </button>
              <button className="btn btn-s" style={{ padding: "2px 12px", fontSize: 12 }} onClick={cancelCreateItem} type="button">
                {t("common:buttons.cancel")}
              </button>
            </div>
          )}

          {/* Scrollable file list */}
          <div
            className={isDragOver ? "drop-zone active" : "drop-zone"}
            style={{ flex: 1, minHeight: 0, overflow: "auto" }}
            onContextMenu={(e) => handleContextMenu(e)}
          >
            {isLoading ? (
              <div className="empty" style={{ padding: 40 }}>
                <RefreshCw size={24} className="spin" />
                <span>{t("loading")}</span>
              </div>
            ) : files.length === 0 ? (
              <div className="empty" style={{ padding: 40 }}>
                <Folder size={24} />
                <span>{t("emptyDir")}</span>
              </div>
            ) : (
              <div className="file-list">
                <div className="file-header">
                  <span style={{ width: 24 }} />
                  <SortHeader field="name" label={t("name")} current={sortField} direction={sortDirection} onSort={setSort} />
                  <SortHeader field="size" label={t("size")} current={sortField} direction={sortDirection} onSort={setSort} style={{ width: 80, textAlign: "right" }} />
                  <SortHeader field="modified" label={t("modified")} current={sortField} direction={sortDirection} onSort={setSort} style={{ width: 140, textAlign: "right" }} />
                </div>
                {sortedFiles.map((entry) => (
                  <FileRow
                    key={entry.path}
                    entry={entry}
                    selected={selectedFiles.has(entry.path)}
                    onSelect={() => selectSingle(entry.path)}
                    onMultiSelect={() => toggleFileSelection(entry.path)}
                    onNavigate={() => handleNavigate(selectedDevice.serial, entry)}
                    onContextMenu={(e) => handleContextMenu(e, entry)}
                  />
                ))}
              </div>
            )}
          </div>

          {/* Transfer progress bars */}
          {activeTransfers.size > 0 && (
            <div style={{ marginTop: 8, flexShrink: 0 }}>
              {[...activeTransfers.values()].map((t) => (
                <div key={t.id} className="transfer-progress">
                  <div className="transfer-info">
                    <span className="transfer-name">{t.fileName}</span>
                    <span className="transfer-percent">{formatTransferProgress(t)}</span>
                  </div>
                  <div className={`progress-bar${t.total > 0 ? "" : " indeterminate"}`}>
                    <div className="progress-fill" style={{ width: t.total > 0 ? `${t.percent}%` : "38%" }} />
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Context menu portal */}
      {ctx && ctxItems.length > 0 && (
        <ContextMenu items={ctxItems} x={ctx.x} y={ctx.y} onClose={closeCtx} />
      )}
      {deleteRequest && (
        <DeleteConfirmDialog
          request={deleteRequest}
          onCancel={() => setDeleteRequest(null)}
          onConfirm={confirmDelete}
        />
      )}

      <style>{`
        .file-list { border: 1px solid var(--bd); border-radius: 6px; overflow: hidden; }
        .file-header {
          display: flex; align-items: center; gap: 8px; padding: 6px 10px;
          background: var(--bg2); font-size: 11px; color: var(--t2); font-weight: 600;
          text-transform: uppercase; letter-spacing: 0.04em;
        }
        .sort-header {
          display: inline-flex; align-items: center; gap: 4px;
          background: none; border: none; color: var(--t2); font-size: 11px;
          font-weight: 600; text-transform: uppercase; letter-spacing: 0.04em;
          cursor: pointer; padding: 0;
          transition: color 0.15s;
        }
        .sort-header:hover { color: var(--acc); }
        .transfer-upload-btn {
          min-width: 76px;
          justify-content: center;
          white-space: nowrap;
        }
        .transfer-upload-btn svg { flex-shrink: 0; }
        .file-row {
          display: flex; align-items: center; gap: 8px; padding: 6px 10px;
          border-top: 1px solid var(--bd); cursor: pointer; font-size: 13px;
          transition: background 0.1s;
        }
        .file-row:hover { background: var(--bg-3); }
        .file-row.selected {
          background: color-mix(in srgb, var(--acc) 20%, transparent);
          outline: 1px solid color-mix(in srgb, var(--acc) 40%, transparent);
          outline-offset: -1px;
        }
        .file-row.selected .file-name { color: var(--acc); font-weight: 600; }
        .file-row .file-icon { width: 24px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; }
        .file-row .file-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
        .file-row .file-size { width: 80px; text-align: right; color: var(--t2); font-size: 12px; }
        .file-row .file-modified { width: 140px; text-align: right; color: var(--t2); font-size: 12px; }
        .ctx-menu {
          position: fixed; z-index: 9999; min-width: 200px;
          background: var(--bg-0); border: 1px solid var(--bdh); border-radius: 8px;
          padding: 4px; box-shadow: 0 8px 32px rgba(0,0,0,0.45), 0 0 0 1px var(--bdh);
        }
        .ctx-item {
          display: flex; align-items: center; gap: 10px; width: 100%; padding: 8px 14px;
          background: none; border: none; color: var(--t1); font-size: 13px;
          cursor: pointer; border-radius: 6px; text-align: left;
          transition: background 0.1s;
        }
        .ctx-item:hover { background: color-mix(in srgb, var(--acc) 15%, transparent); color: var(--acc); }
        .ctx-item.danger { color: var(--wrn); }
        .ctx-item.danger:hover { background: color-mix(in srgb, var(--wrn) 15%, transparent); }
        .dialog-backdrop {
          position: fixed; inset: 0; z-index: 10000;
          display: flex; align-items: center; justify-content: center;
          padding: 24px; background: rgba(0, 0, 0, 0.52);
        }
        .confirm-dialog {
          width: min(420px, 100%);
          background: var(--bg-0); border: 1px solid var(--bdh); border-radius: 8px;
          padding: 18px; box-shadow: 0 18px 48px rgba(0,0,0,0.5);
        }
        .confirm-dialog-title { font-size: 16px; font-weight: 700; color: var(--t1); margin-bottom: 8px; }
        .confirm-dialog-message { font-size: 13px; color: var(--t2); margin-bottom: 10px; }
        .confirm-dialog-list {
          margin: 0 0 10px; padding: 8px 10px 8px 24px;
          max-height: 124px; overflow: auto; border: 1px solid var(--bd);
          border-radius: 6px; background: var(--bg2); color: var(--t1); font-size: 12px;
        }
        .confirm-dialog-list li { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin: 2px 0; }
        .confirm-dialog-warning { color: var(--wrn); font-size: 12px; margin-bottom: 16px; }
        .confirm-dialog-actions { display: flex; justify-content: flex-end; gap: 8px; }
        .danger-btn { background: var(--wrn); border-color: var(--wrn); color: var(--bg-0); }
        .drop-zone { position: relative; }
        .drop-zone.active { outline: 2px dashed var(--acc); outline-offset: -2px; background: color-mix(in srgb, var(--acc) 8%, transparent); }
        .transfer-progress { padding: 8px 12px; background: var(--bg2); border-radius: 6px; margin-bottom: 4px; }
        .transfer-info { display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px; }
        .transfer-name { font-size: 12px; color: var(--t1); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 50%; }
        .transfer-percent { font-size: 12px; color: var(--acc); font-weight: 600; white-space: nowrap; }
        .progress-bar { height: 4px; background: var(--bd); border-radius: 2px; overflow: hidden; }
        .progress-fill { height: 100%; background: var(--acc); border-radius: 2px; transition: width 0.3s ease; }
        .progress-bar.indeterminate .progress-fill { animation: progress-slide 1.2s ease-in-out infinite; }
        @keyframes progress-slide { 0% { transform: translateX(-110%); } 100% { transform: translateX(280%); } }
        @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
        .spin { animation: spin 1s linear infinite; }
      `}</style>
    </div>
  );
}

/* ======================== Sort Header ======================== */

function SortHeader({
  field,
  label,
  current,
  direction,
  onSort,
  style,
}: {
  field: "name" | "size" | "modified";
  label: string;
  current: string;
  direction: "asc" | "desc";
  onSort: (field: "name" | "size" | "modified") => void;
  style?: React.CSSProperties;
}) {
  const active = current === field;
  return (
    <button
      className="sort-header"
      style={{ flex: field === "name" ? 1 : undefined, ...style }}
      onClick={() => onSort(field)}
      type="button"
    >
      {label}
      {active && (
        direction === "asc" ? <ArrowUp size={10} /> : <ArrowDown size={10} />
      )}
    </button>
  );
}

/* ======================== File Row ======================== */

function FileRow({
  entry,
  selected,
  onSelect,
  onMultiSelect,
  onNavigate,
  onContextMenu,
}: {
  entry: FileEntry;
  selected: boolean;
  onSelect: () => void;
  onMultiSelect: () => void;
  onNavigate: () => void;
  onContextMenu: (e: React.MouseEvent) => void;
}) {
  const { t } = useTranslation("transfer");
  const Icon = entry.isDirectory ? Folder : File;
  const iconColor = entry.isDirectory ? "var(--acc)" : "var(--t2)";

  return (
    <div
      className={`file-row${selected ? " selected" : ""}`}
      onClick={(e) => {
        if (e.ctrlKey || e.metaKey) {
          onMultiSelect();
        } else {
          onSelect();
        }
      }}
      onDoubleClick={() => {
        if (entry.isDirectory) onNavigate();
      }}
      onContextMenu={onContextMenu}
    >
      <div className="file-icon">
        <Icon size={16} style={{ color: iconColor }} />
      </div>
      <span className="file-name">{entry.name}</span>
      <span className="file-size">
        {entry.size != null ? formatSize(entry.size, t) : ""}
      </span>
      <span className="file-modified">{entry.modified ?? ""}</span>
    </div>
  );
}

/* ======================== Wi-Fi Panel ======================== */

function WifiTransferPanel() {
  const { t } = useTranslation("transfer");
  const wifiStatus = useTransferStore((s) => s.wifiStatus);
  const isWifiBusy = useTransferStore((s) => s.isWifiBusy);
  const startWifiTransfer = useTransferStore((s) => s.startWifiTransfer);
  const stopWifiTransfer = useTransferStore((s) => s.stopWifiTransfer);
  const loadWifiStatus = useTransferStore((s) => s.loadWifiStatus);

  useEffect(() => { loadWifiStatus(); }, [loadWifiStatus]);

  const isRunning = wifiStatus?.running ?? false;

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  return (
    <div>
      <div className="card" style={{ padding: 24, textAlign: "center" }}>
        <div style={{ marginBottom: 16 }}>
          {isRunning ? (
            <WifiOff size={40} style={{ color: "var(--acc)" }} />
          ) : (
            <Wifi size={40} style={{ color: "var(--t3)" }} />
          )}
        </div>

        <div style={{ marginBottom: 16, fontWeight: 600, fontSize: 16 }}>
          {isRunning ? t("wifiRunning") : t("wifiStopped_status")}
        </div>

        {isRunning && wifiStatus?.url && (
          <>
            <div style={{ marginBottom: 16 }}>
              {wifiStatus.qrCodeDataUrl && (
                <img
                  src={wifiStatus.qrCodeDataUrl}
                  alt="QR Code"
                  style={{ width: 200, height: 200, margin: "0 auto", display: "block" }}
                />
              )}
              <div style={{ color: "var(--t2)", fontSize: 12, marginTop: 8 }}>{t("wifiQrCode")}</div>
            </div>

            <div style={{ marginBottom: 12 }}>
              <div style={{ color: "var(--t2)", fontSize: 11, fontWeight: 600, marginBottom: 4, textTransform: "uppercase", letterSpacing: "0.04em" }}>
                {t("wifiUrl")}
              </div>
              <div className="row" style={{ justifyContent: "center", gap: 8 }}>
                <code style={{ fontSize: 14, color: "var(--acc)", userSelect: "all" }}>{wifiStatus.url}</code>
                <button className="btn btn-g" style={{ padding: 2 }} onClick={() => copyToClipboard(wifiStatus.url!)} type="button">
                  <Copy size={14} />
                </button>
              </div>
            </div>

            {wifiStatus.token && (
              <div style={{ marginBottom: 16 }}>
                <div style={{ color: "var(--t2)", fontSize: 11, fontWeight: 600, marginBottom: 4, textTransform: "uppercase", letterSpacing: "0.04em" }}>
                  {t("wifiToken")}
                </div>
                <div className="row" style={{ justifyContent: "center", gap: 8 }}>
                  <code style={{ fontSize: 20, fontWeight: 700, letterSpacing: "0.15em", color: "var(--acc)" }}>
                    {wifiStatus.token}
                  </code>
                  <button className="btn btn-g" style={{ padding: 2 }} onClick={() => copyToClipboard(wifiStatus.token!)} type="button">
                    <Copy size={14} />
                  </button>
                </div>
              </div>
            )}
          </>
        )}

        {!isRunning && (
          <div style={{ color: "var(--t2)", fontSize: 13, marginBottom: 16 }}>{t("wifiHint")}</div>
        )}

        <button
          className={`btn ${isRunning ? "btn-s" : "btn-p"}`}
          onClick={() => isRunning ? stopWifiTransfer() : startWifiTransfer()}
          disabled={isWifiBusy}
          type="button"
          style={{ padding: "8px 24px" }}
        >
          {isWifiBusy ? (
            <RefreshCw size={14} className="spin" />
          ) : isRunning ? (
            <WifiOff size={14} />
          ) : (
            <Wifi size={14} />
          )}
          {isRunning ? t("wifiStop") : t("wifiStart")}
        </button>
      </div>
    </div>
  );
}

/* ======================== Helpers ======================== */

function formatSize(bytes: number, t: (key: string) => string): string {
  if (bytes < 1024) return `${bytes} ${t("bytes")}`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} ${t("kilobytes")}`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} ${t("megabytes")}`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} ${t("gigabytes")}`;
}

function formatTransferProgress(progress: TransferProgress): string {
  const speed = progress.speed ? ` - ${progress.speed}` : "";
  if (progress.total > 0) {
    return `${formatSize(progress.transferred, (key) => sizeUnit(key))} / ${formatSize(progress.total, (key) => sizeUnit(key))} - ${progress.percent}%${speed}`;
  }
  return `${formatSize(progress.transferred, (key) => sizeUnit(key))}${speed}`;
}

function sizeUnit(key: string): string {
  switch (key) {
    case "bytes":
      return "B";
    case "kilobytes":
      return "KB";
    case "megabytes":
      return "MB";
    case "gigabytes":
      return "GB";
    default:
      return key;
  }
}
