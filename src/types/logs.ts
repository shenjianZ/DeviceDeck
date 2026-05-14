export type LogSource = "system" | "adb" | "scrcpy";
export type LogLevel = "info" | "warn" | "error";

export interface AppLog {
  id: string;
  time: number;
  source: LogSource;
  level: LogLevel;
  deviceSerial: string;
  message: string;
}
