import type { MirrorConfig } from "./mirror";

export interface AppSettings {
  useBundledAdb: boolean;
  useBundledScrcpy: boolean;
  customAdbPath: string;
  customScrcpyPath: string;
  defaultMirrorConfig: MirrorConfig;
  lastMirrorConfig?: MirrorConfig | null;
  theme: string;
  logRetentionDays: number;
  autoScanDevices: boolean;
  deviceScanIntervalSeconds: number;
  fontSize?: number;
  locale?: "zh-CN" | "en";
  autoStart?: boolean;
  autoUpdateEnabled?: boolean;
  firstRun?: boolean;
}

export interface ToolStatus {
  name: string;
  available: boolean;
  path?: string | null;
  version?: string | null;
  message?: string | null;
}

export interface EnvironmentStatus {
  adb: ToolStatus;
  scrcpy: ToolStatus;
  providerStatus: string;
}

export interface AppError {
  code: string;
  message: string;
  detail?: string | null;
  suggestion?: string | null;
}
