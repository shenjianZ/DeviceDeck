import { create } from "zustand";
import { tauriApi } from "../lib/tauri";
import { useNotificationStore } from "./notificationStore";
import { useMirrorStore } from "./mirrorStore";
import i18n from "../i18n";
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
        useNotificationStore.getState().showSuccess(i18n.t("common:notifications.scanComplete"), i18n.t("common:notifications.devicesFound", { count: devices.length }));
      }
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isScanning: false });
      useNotificationStore.getState().showError(i18n.t("common:notifications.scanFailed"), err.message, err.suggestion);
    }
  },

  discoverWirelessDevices: async (silent = false) => {
    set({ isDiscoveringWireless: true, error: null });
    try {
      const wirelessServices = await tauriApi.discoverWirelessDevices();
      set({ wirelessServices, isDiscoveringWireless: false });
      if (!silent) {
        useNotificationStore.getState().showSuccess(i18n.t("common:notifications.wirelessScanComplete"), i18n.t("common:notifications.servicesFound", { count: wirelessServices.length }));
      }
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isDiscoveringWireless: false });
      useNotificationStore.getState().showError(i18n.t("common:notifications.wirelessScanFailed"), err.message, err.suggestion);
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
        wirelessMessage: i18n.t("common:notifications.connectedTo", { serial: device.serial }),
      }));
      useNotificationStore.getState().showSuccess(i18n.t("common:notifications.wirelessEnabled"), `${device.serial}:${port}`);
      return device;
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isWirelessBusy: false });
      useNotificationStore.getState().showError(i18n.t("common:notifications.wirelessEnableFailed"), err.message, err.suggestion);
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
        wirelessMessage: i18n.t("common:notifications.connectedTo", { serial: device.serial }),
      }));
      useNotificationStore.getState().showSuccess(i18n.t("common:notifications.wirelessConnected"), `${host}:${port}`);
      return device;
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isWirelessBusy: false });
      useNotificationStore.getState().showError(i18n.t("common:notifications.wirelessConnectFailed"), err.message, err.suggestion);
      return null;
    }
  },

  pairWirelessDevice: async (host, port, pairingCode) => {
    set({ isWirelessBusy: true, error: null, wirelessMessage: null });
    try {
      const message = await tauriApi.pairWirelessDevice(host, port, pairingCode);
      set({ isWirelessBusy: false, wirelessMessage: message || i18n.t("common:notifications.pairSuccessWireless") });
      useNotificationStore.getState().showSuccess(i18n.t("common:notifications.pairSuccess"), `${host}:${port}`);
      return true;
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isWirelessBusy: false });
      useNotificationStore.getState().showError(i18n.t("common:notifications.pairFailed"), err.message, err.suggestion);
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
        wirelessMessage: i18n.t("common:notifications.disconnectedFrom", { serial }),
      }));
      useNotificationStore.getState().showSuccess(i18n.t("common:notifications.deviceDisconnected"), serial);
      return true;
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isWirelessBusy: false });
      useNotificationStore.getState().showError(i18n.t("common:notifications.disconnectFailed"), err.message, err.suggestion);
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
      useNotificationStore.getState().showSuccess(i18n.t("common:notifications.capabilityComplete"), i18n.t("common:notifications.capabilityDetail", { serial }));
    } catch (e: unknown) {
      const err = e as AppError;
      set({ error: err, isDetectingCapabilities: false });
      useNotificationStore.getState().showError(i18n.t("common:notifications.capabilityFailed"), err.message, err.suggestion);
    }
  },

  applyRecommendedConfig: (_serial, config) => {
    useMirrorStore.getState().applyPreset(config);
    useNotificationStore.getState().showSuccess(i18n.t("common:notifications.configApplied"), i18n.t("common:notifications.configAppliedDetail"));
  },

  takeScreenshot: async (serial, outputDirectory) => {
    await runDeviceAction(set, () => tauriApi.takeDeviceScreenshot(serial, outputDirectory), "screenshotComplete", "screenshotFailed");
  },

  installApk: async (serial, apkPath) => {
    await runDeviceAction(set, () => tauriApi.installDeviceApk(serial, apkPath), "apkInstalled", "apkInstallFailed");
  },

  pushFile: async (serial, localPath, remoteDirectory) => {
    await runDeviceAction(set, () => tauriApi.pushDeviceFile(serial, localPath, remoteDirectory), "fileSent", "fileSendFailed");
  },

  runKeyAction: async (serial, action) => {
    await runDeviceAction(set, () => tauriApi.runDeviceKeyAction(serial, action), "keyActionComplete", "keyActionFailed");
  },

  runShellCommand: async (serial, command) => {
    await runDeviceAction(set, () => tauriApi.runAdbShellCommand(serial, command), "adbCommandComplete", "adbCommandFailed");
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
  successKey: string,
  errorKey: string
) {
  set({ isDeviceActionBusy: true, error: null });
  try {
    const result = await action();
    set({ isDeviceActionBusy: false });
    const detail = result.outputPath || result.stdout || result.message;
    useNotificationStore.getState().showSuccess(i18n.t(`common:notifications.${successKey}`), detail);
  } catch (e: unknown) {
    const err = e as AppError;
    set({ error: err, isDeviceActionBusy: false });
    useNotificationStore.getState().showError(i18n.t(`common:notifications.${errorKey}`), err.message, err.suggestion || err.detail || undefined);
  }
}
