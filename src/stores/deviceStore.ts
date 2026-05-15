import { create } from "zustand";
import { tauriApi } from "../lib/tauri";
import { useNotificationStore } from "./notificationStore";
import { useMirrorStore } from "./mirrorStore";
import type {
  DeviceInfo,
  EnvironmentStatus,
  AppError,
  WirelessAdbService,
  RecommendedConfig,
  MirrorConfig,
  DeviceKeyAction,
} from "../types";

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
  capabilityReports: Record<string, RecommendedConfig[]>;
  isDetectingCapabilities: boolean;
  isDeviceActionBusy: boolean;

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
  detectCapabilities: (serial: string) => Promise<void>;
  applyRecommendedConfig: (serial: string, config: MirrorConfig) => void;
  takeScreenshot: (serial: string, outputDirectory?: string) => Promise<void>;
  installApk: (serial: string, apkPath: string) => Promise<void>;
  pushFile: (serial: string, localPath: string, remoteDirectory: string) => Promise<void>;
  runKeyAction: (serial: string, action: DeviceKeyAction) => Promise<void>;
  runShellCommand: (serial: string, command: string) => Promise<void>;
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
  capabilityReports: {},
  isDetectingCapabilities: false,
  isDeviceActionBusy: false,

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

  detectCapabilities: async (serial) => {
    set({ isDetectingCapabilities: true, error: null });
    try {
      const recommendations = await tauriApi.detectDeviceCapabilities(serial);
      set((state) => ({
        capabilityReports: { ...state.capabilityReports, [serial]: recommendations },
        isDetectingCapabilities: false,
      }));
      useNotificationStore.getState().showSuccess("能力检测完成", `设备 ${serial} 的推荐配置已生成`);
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isDetectingCapabilities: false });
      useNotificationStore.getState().showError("能力检测失败", err.message, err.suggestion);
    }
  },

  applyRecommendedConfig: (_serial, config) => {
    useMirrorStore.getState().applyPreset(config);
    useNotificationStore.getState().showSuccess("已应用配置", "推荐配置已应用到投屏参数");
  },

  takeScreenshot: async (serial, outputDirectory) => {
    await runDeviceAction(set, () => tauriApi.takeDeviceScreenshot(serial, outputDirectory), "截图完成", "截图失败");
  },

  installApk: async (serial, apkPath) => {
    await runDeviceAction(set, () => tauriApi.installDeviceApk(serial, apkPath), "APK 安装完成", "APK 安装失败");
  },

  pushFile: async (serial, localPath, remoteDirectory) => {
    await runDeviceAction(set, () => tauriApi.pushDeviceFile(serial, localPath, remoteDirectory), "文件发送完成", "文件发送失败");
  },

  runKeyAction: async (serial, action) => {
    await runDeviceAction(set, () => tauriApi.runDeviceKeyAction(serial, action), "快捷操作完成", "快捷操作失败");
  },

  runShellCommand: async (serial, command) => {
    await runDeviceAction(set, () => tauriApi.runAdbShellCommand(serial, command), "ADB 命令完成", "ADB 命令失败");
  },
}));

function upsertDevice(devices: DeviceInfo[], device: DeviceInfo): DeviceInfo[] {
  const exists = devices.some((item) => item.serial === device.serial);
  if (!exists) return [...devices, device];
  return devices.map((item) => (item.serial === device.serial ? device : item));
}

async function runDeviceAction(
  set: (partial: Partial<DeviceStore>) => void,
  action: () => Promise<{ message: string; outputPath?: string | null; stdout?: string | null }>,
  successTitle: string,
  errorTitle: string
) {
  set({ isDeviceActionBusy: true, error: null });
  try {
    const result = await action();
    set({ isDeviceActionBusy: false });
    const detail = result.outputPath || result.stdout || result.message;
    useNotificationStore.getState().showSuccess(successTitle, detail);
  } catch (e: unknown) {
    const err = e as AppError;
    set({ error: err, isDeviceActionBusy: false });
    useNotificationStore.getState().showError(errorTitle, err.message, err.suggestion || err.detail || undefined);
  }
}
