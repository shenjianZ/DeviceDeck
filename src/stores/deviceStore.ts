import { create } from "zustand";
import { tauriApi } from "../lib/tauri";
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
  scanDevices: () => Promise<void>;
  discoverWirelessDevices: () => Promise<void>;
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

  scanDevices: async () => {
    set({ isScanning: true, error: null });
    try {
      const devices = await tauriApi.scanDevices();
      set({ devices, isScanning: false });
    } catch (e: unknown) {
      set({ error: e as AppError, isScanning: false });
    }
  },

  discoverWirelessDevices: async () => {
    set({ isDiscoveringWireless: true, error: null });
    try {
      const wirelessServices = await tauriApi.discoverWirelessDevices();
      set({ wirelessServices, isDiscoveringWireless: false });
    } catch (e: unknown) {
      set({ error: e as AppError, isDiscoveringWireless: false });
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
      return device;
    } catch (e: unknown) {
      set({ error: e as AppError, isWirelessBusy: false });
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
      return device;
    } catch (e: unknown) {
      set({ error: e as AppError, isWirelessBusy: false });
      return null;
    }
  },

  pairWirelessDevice: async (host, port, pairingCode) => {
    set({ isWirelessBusy: true, error: null, wirelessMessage: null });
    try {
      const message = await tauriApi.pairWirelessDevice(host, port, pairingCode);
      set({ isWirelessBusy: false, wirelessMessage: message || "无线调试配对成功" });
      return true;
    } catch (e: unknown) {
      set({ error: e as AppError, isWirelessBusy: false });
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
      return true;
    } catch (e: unknown) {
      set({ error: e as AppError, isWirelessBusy: false });
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
