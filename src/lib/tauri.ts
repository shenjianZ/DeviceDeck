import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  DeviceInfo,
  EnvironmentStatus,
  MirrorConfig,
  MirrorSession,
  AppSettings,
  AppLog,
  WirelessAdbService,
  RecommendedConfig,
  DeviceActionResult,
  DeviceKeyAction,
} from "../types";

/** 分页查询结果 */
export interface PaginatedLogs {
  logs: AppLog[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

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
  discoverWirelessDevices: () =>
    invoke<WirelessAdbService[]>("discover_wireless_devices"),
  pairWirelessDevice: (host: string, port: number, pairingCode: string) =>
    invoke<string>("pair_wireless_device", { host, port, pairingCode }),
  disconnectWirelessDevice: (serial: string) =>
    invoke<void>("disconnect_wireless_device", { serial }),
  detectDeviceCapabilities: (serial: string) =>
    invoke<RecommendedConfig[]>("detect_device_capabilities", { serial }),
  takeDeviceScreenshot: (serial: string, outputDirectory?: string) =>
    invoke<DeviceActionResult>("take_device_screenshot", { serial, outputDirectory }),
  installDeviceApk: (serial: string, apkPath: string) =>
    invoke<DeviceActionResult>("install_device_apk", { serial, apkPath }),
  pushDeviceFile: (serial: string, localPath: string, remoteDirectory: string) =>
    invoke<DeviceActionResult>("push_device_file", { serial, localPath, remoteDirectory }),
  runDeviceKeyAction: (serial: string, action: DeviceKeyAction) =>
    invoke<DeviceActionResult>("run_device_key_action", { serial, action }),
  runAdbShellCommand: (serial: string, command: string, timeoutMs = 30000) =>
    invoke<DeviceActionResult>("run_adb_shell_command", { serial, command, timeoutMs }),

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
  getLogsPaginated: (page = 1, pageSize = 50, sourceFilter?: string, levelFilter?: string) =>
    invoke<PaginatedLogs>("get_logs_paginated", {
      page,
      pageSize,
      sourceFilter: sourceFilter === "all" ? undefined : sourceFilter,
      levelFilter: levelFilter === "all" ? undefined : levelFilter,
    }),
  clearLogs: () =>
    invoke<void>("clear_logs"),

  // Settings
  getSettings: () => invoke<AppSettings>("get_settings"),
  updateSettings: (settings: AppSettings) =>
    invoke<void>("update_settings", { settings }),
  resetSettings: () => invoke<AppSettings>("reset_settings"),

  // Autostart
  setAutostart: (enabled: boolean) =>
    enabled
      ? invoke("plugin:autostart|enable")
      : invoke("plugin:autostart|disable"),
  getAutostart: () => invoke<boolean>("plugin:autostart|is_enabled"),

  // Events
  onLog: (handler: (log: AppLog) => void) =>
    listen<AppLog>("log://new", (e) => handler(e.payload)),
  onSessionUpdated: (handler: (session: MirrorSession) => void) =>
    listen<MirrorSession>("mirror://session-updated", (e) =>
      handler(e.payload)
    ),
};
