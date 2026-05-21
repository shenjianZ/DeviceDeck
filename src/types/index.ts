export type {
  DeviceInfo,
  DevicePlatform,
  DeviceStatus,
  ConnectionType,
  DeviceCapability,
  WirelessAdbService,
  WirelessAdbServiceType,
  VideoCodec,
  DeviceCapabilityReport,
  RecommendedConfig,
  DeviceActionResult,
  DeviceKeyAction,
  FileEntry,
  WifiTransferStatus,
  TransferProgress,
} from "./device";
export type {
  MirrorConfig,
  MirrorSession,
  MirrorPreset,
  SessionStatus,
  RecordMode,
  RecordFormat,
  MirrorOrientation,
  AudioSource,
  AudioCodec,
} from "./mirror";
export type { AppLog, LogSource, LogLevel } from "./logs";
export type { AppSettings, ToolStatus, EnvironmentStatus, AppError } from "./settings";

export type Page = "dashboard" | "devices" | "mirror" | "logs" | "settings" | "transfer";
