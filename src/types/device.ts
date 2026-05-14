export type DevicePlatform = "android" | "ios" | "androidTv" | "unknown";
export type DeviceStatus = "online" | "offline" | "unauthorized" | "busy" | "unknown";
export type ConnectionType = "usb" | "wifi" | "unknown";
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
