export interface Device {
  id: string;
  serial: string;
  name: string;
  model: string;
  brand: string;
  platform: string;
  status: "online" | "offline" | "unauthorized" | "unknown";
  connectionType: "usb" | "wifi" | "unknown";
  androidVersion: string;
  screenSize: string;
  batteryLevel: number | null;
  capabilities: string[];
}

export interface MirrorConfig {
  maxSize: string;
  videoBitRate: string;
  maxFps: string;
  noControl: boolean;
  stayAwake: boolean;
  turnScreenOff: boolean;
}

export interface Preset {
  id: string;
  name: string;
  desc: string;
  config: MirrorConfig;
}

export interface Session {
  id: string;
  deviceSerial: string;
  platform: string;
  processId: number;
  status: "running" | "stopped";
  startedAt: number;
  config: MirrorConfig;
}

export interface LogEntry {
  id: string;
  time: Date;
  source: "system" | "adb" | "scrcpy";
  level: "info" | "warn" | "error";
  deviceSerial: string;
  message: string;
}

export interface AppSettings {
  useBundledAdb: boolean;
  useBundledScrcpy: boolean;
  customAdbPath: string;
  customScrcpyPath: string;
  defaultMirrorConfig: MirrorConfig;
  theme: "dark" | "light";
  logRetentionDays: number;
}

export type Page = "dashboard" | "devices" | "mirror" | "logs" | "settings";
