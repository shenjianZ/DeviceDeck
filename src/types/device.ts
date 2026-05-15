export type DevicePlatform = "android" | "ios" | "androidTv" | "unknown";
export type DeviceStatus = "online" | "offline" | "unauthorized" | "busy" | "unknown";
export type ConnectionType = "usb" | "wifi" | "unknown";
export type WirelessAdbServiceType = "pairing" | "connect";
export type DeviceCapability =
  | "mirror"
  | "control"
  | "screenshot"
  | "recording"
  | "wireless"
  | "installApp"
  | "uninstallApp"
  | "logs"
  | "fileTransfer"
  | "automation";

export interface DeviceInfo {
  id: string;
  serial: string;
  name: string;
  model: string;
  brand: string;
  platform: DevicePlatform;
  status: DeviceStatus;
  connectionType: ConnectionType;
  androidVersion?: string;
  screenSize?: string;
  batteryLevel?: number | null;
  capabilities: DeviceCapability[];
}

export interface WirelessAdbService {
  id: string;
  name: string;
  host: string;
  port: number;
  serviceType: WirelessAdbServiceType;
}

export type VideoCodec = "h264" | "h265" | "av1";

export interface DeviceCapabilityReport {
  serial: string;
  supportedEncoders: string[];
  supportedCodecs: VideoCodec[];
  screenWidth: number | null;
  screenHeight: number | null;
  androidVersion: string | null;
}

export interface RecommendedConfig {
  label: string;
  description: string;
  config: import("./mirror").MirrorConfig;
}

export interface DeviceActionResult {
  message: string;
  outputPath?: string | null;
  stdout?: string | null;
  stderr?: string | null;
}

export type DeviceKeyAction =
  | "home"
  | "back"
  | "appSwitch"
  | "menu"
  | "power"
  | "volumeUp"
  | "volumeDown"
  | "expandNotifications"
  | "collapseNotifications"
  | "turnScreenOff"
  | "screenBlack"
  | "screenRestore";
