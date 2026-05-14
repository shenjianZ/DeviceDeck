export type { DeviceInfo, DevicePlatform, DeviceStatus, ConnectionType, DeviceCapability } from "./device";
export type { MirrorConfig, MirrorSession, MirrorPreset, SessionStatus } from "./mirror";
export type { AppLog, LogSource, LogLevel } from "./logs";
export type { AppSettings, ToolStatus, EnvironmentStatus, AppError } from "./settings";

export type Page = "dashboard" | "devices" | "mirror" | "logs" | "settings";
