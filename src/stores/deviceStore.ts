import { create } from "zustand";
import { tauriApi } from "../lib/tauri";
import { useNotificationStore } from "./notificationStore";
import type { DeviceInfo, EnvironmentStatus, AppError, WirelessAdbService } from "../types";

interface DeviceStore {
  devices: DeviceInfo[];
  wirelessServices: WirelessAdbService[];
  selectedDeviceId: string | null;
  isScanning: boolean;
  isDiscoveringWireless: boolean;
  isWirelessBusy: boolean;
  environment: EnvironmentStatus | null;
  error: AppError | null;
  wirelessMessage: string | null;

  checkEnvironment: () => Promise<void>;
  scanDevices: (silent?: boolean) => Promise<void>;
  discoverWirelessDevices: (silent?: boolean) => Promise<void>;
  selectDevice: (id: string | null) => void;
  refreshDeviceDetail: (serial: string) => Promise<void>;
  enableWirelessDevice: (serial: string, port?: number) => Promise<DeviceInfo | null>;
  connectWirelessDevice: (host: string, port?: number) => Promise<DeviceInfo | null>;
  pairWirelessDevice: (host: string, port: number, pairingCode: string) => Promise<boolean>;
  disconnectWirelessDevice: (serial: string) => Promise<boolean>;
  clearWirelessMessage: () => void;
}

export const useDeviceStore = create<DeviceStore>((set) => ({
  devices: [],
  wirelessServices: [],
  selectedDeviceId: null,
  isScanning: false,
  isDiscoveringWireless: false,
  isWirelessBusy: false,
  environment: null,
  error: null,
  wirelessMessage: null,

  checkEnvironment: async () => {
    try {
      const env = await tauriApi.checkEnvironment();
      set({ environment: env, error: null });
    } catch (e: unknown) {
      set({ error: e as AppError });
    }
  },

  scanDevices: async (silent = false) => {
    set({ isScanning: true, error: null });
    try {
      const devices = await tauriApi.scanDevices();
      set({ devices, isScanning: false });
      if (!silent) {
        useNotificationStore.getState().showSuccess("设备扫描完成", `发现 ${devices.length} 个设备`);
      }
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isScanning: false });
      useNotificationStore.getState().showError("设备扫描失败", err.message, err.suggestion);
    }
  },

  discoverWirelessDevices: async (silent = false) => {
    set({ isDiscoveringWireless: true, error: null });
    try {
      const wirelessServices = await tauriApi.discoverWirelessDevices();
      set({ wirelessServices, isDiscoveringWireless: false });
      if (!silent) {
        useNotificationStore.getState().showSuccess("无线设备扫描完成", `发现 ${wirelessServices.length} 个服务`);
      }
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isDiscoveringWireless: false });
      useNotificationStore.getState().showError("无线设备扫描失败", err.message, err.suggestion);
    }
  },

  selectDevice: (id) => set({ selectedDeviceId: id }),

  refreshDeviceDetail: async (serial) => {
    try {
      const detail = await tauriApi.getDeviceDetail(serial);
      set((state) => ({
        devices: state.devices.map((d) =>
          d.serial === serial ? detail : d
        ),
      }));
    } catch (e: unknown) {
      set({ error: e as AppError });
    }
  },

  enableWirelessDevice: async (serial, port = 5555) => {
    set({ isWirelessBusy: true, error: null, wirelessMessage: null });
    try {
      const device = await tauriApi.enableWirelessDevice(serial, port);
      set((state) => ({
        devices: upsertDevice(state.devices, device),
        selectedDeviceId: device.id,
        isWirelessBusy: false,
        wirelessMessage: `已连接无线设备 ${device.serial}`,
      }));
      useNotificationStore.getState().showSuccess("无线设备已启用", `${device.serial}:${port}`);
      return device;
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isWirelessBusy: false });
      useNotificationStore.getState().showError("启用无线设备失败", err.message, err.suggestion);
      return null;
    }
  },

  connectWirelessDevice: async (host, port = 5555) => {
    set({ isWirelessBusy: true, error: null, wirelessMessage: null });
    try {
      const device = await tauriApi.connectWirelessDevice(host, port);
      set((state) => ({
        devices: upsertDevice(state.devices, device),
        selectedDeviceId: device.id,
        isWirelessBusy: false,
        wirelessMessage: `已连接无线设备 ${device.serial}`,
      }));
      useNotificationStore.getState().showSuccess("无线设备已连接", `${host}:${port}`);
      return device;
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isWirelessBusy: false });
      useNotificationStore.getState().showError("连接无线设备失败", err.message, err.suggestion);
      return null;
    }
  },

  pairWirelessDevice: async (host, port, pairingCode) => {
    set({ isWirelessBusy: true, error: null, wirelessMessage: null });
    try {
      const message = await tauriApi.pairWirelessDevice(host, port, pairingCode);
      set({ isWirelessBusy: false, wirelessMessage: message || "无线调试配对成功" });
      useNotificationStore.getState().showSuccess("配对成功", `${host}:${port}`);
      return true;
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isWirelessBusy: false });
      useNotificationStore.getState().showError("配对失败", err.message, err.suggestion);
      return false;
    }
  },

  disconnectWirelessDevice: async (serial) => {
    set({ isWirelessBusy: true, error: null, wirelessMessage: null });
    try {
      await tauriApi.disconnectWirelessDevice(serial);
      set((state) => ({
        devices: state.devices.filter((device) => device.serial !== serial),
        selectedDeviceId: state.selectedDeviceId === serial ? null : state.selectedDeviceId,
        isWirelessBusy: false,
        wirelessMessage: `已断开 ${serial}`,
      }));
      useNotificationStore.getState().showSuccess("设备已断开", serial);
      return true;
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isWirelessBusy: false });
      useNotificationStore.getState().showError("断开设备失败", err.message, err.suggestion);
      return false;
    }
  },

  clearWirelessMessage: () => set({ wirelessMessage: null }),
}));

function upsertDevice(devices: DeviceInfo[], device: DeviceInfo): DeviceInfo[] {
  const exists = devices.some((item) => item.serial === device.serial);
  if (!exists) return [...devices, device];
  return devices.map((item) => (item.serial === device.serial ? device : item));
}
