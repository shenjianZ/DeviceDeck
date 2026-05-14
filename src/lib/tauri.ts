import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  DeviceInfo,
  EnvironmentStatus,
  MirrorConfig,
  MirrorSession,
  AppSettings,
  AppLog,
} from "../types";

export const tauriApi = {
  // Environment
  checkEnvironment: () => invoke<EnvironmentStatus>("check_environment"),

  // Devices
  scanDevices: () => invoke<DeviceInfo[]>("scan_devices"),
  getDeviceDetail: (serial: string) =>
    invoke<DeviceInfo>("get_device_detail", { serial }),
  enableWirelessDevice: (serial: string, port = 5555) =>
    invoke<DeviceInfo>("enable_wireless_device", { serial, port }),
  connectWirelessDevice: (host: string, port = 5555) =>
    invoke<DeviceInfo>("connect_wireless_device", { host, port }),
  pairWirelessDevice: (host: string, port: number, pairingCode: string) =>
    invoke<string>("pair_wireless_device", { host, port, pairingCode }),
  disconnectWirelessDevice: (serial: string) =>
    invoke<void>("disconnect_wireless_device", { serial }),

  // Mirror
  startMirror: (serial: string, config: MirrorConfig) =>
    invoke<MirrorSession>("start_mirror", { serial, config }),
  startWirelessMirror: (serial: string, config: MirrorConfig, port = 5555) =>
    invoke<MirrorSession>("start_wireless_mirror", { serial, config, port }),
  connectWirelessAndStartMirror: (host: string, port: number, config: MirrorConfig) =>
    invoke<MirrorSession>("connect_wireless_and_start_mirror", { host, port, config }),
  stopMirror: (sessionId: string) =>
    invoke<void>("stop_mirror", { sessionId }),
  listMirrorSessions: () =>
    invoke<MirrorSession[]>("list_mirror_sessions"),

  // Logs
  getRecentLogs: (limit = 500) =>
    invoke<AppLog[]>("get_recent_logs", { limit }),
  clearLogs: () =>
    invoke<void>("clear_logs"),

  // Settings
  getSettings: () => invoke<AppSettings>("get_settings"),
  updateSettings: (settings: AppSettings) =>
    invoke<void>("update_settings", { settings }),
  resetSettings: () => invoke<AppSettings>("reset_settings"),

  // Events
  onLog: (handler: (log: AppLog) => void) =>
    listen<AppLog>("log://new", (e) => handler(e.payload)),
  onSessionUpdated: (handler: (session: MirrorSession) => void) =>
    listen<MirrorSession>("mirror://session-updated", (e) =>
      handler(e.payload)
    ),
};
