import type { MirrorPreset } from "../types";

export const PRESETS: MirrorPreset[] = [
  {
    id: "smooth",
    name: "流畅模式",
    description: "720p · 4M · 60fps",
    config: {
      maxSize: "720",
      videoBitRate: "4M",
      maxFps: "60",
      noControl: false,
      stayAwake: true,
      turnScreenOff: false,
    },
  },
  {
    id: "hd",
    name: "高清模式",
    description: "1080p · 8M · 60fps",
    config: {
      maxSize: "1080",
      videoBitRate: "8M",
      maxFps: "60",
      noControl: false,
      stayAwake: true,
      turnScreenOff: false,
    },
  },
  {
    id: "ultra",
    name: "极清模式",
    description: "1440p · 16M · 60fps",
    config: {
      maxSize: "1440",
      videoBitRate: "16M",
      maxFps: "60",
      noControl: false,
      stayAwake: true,
      turnScreenOff: false,
    },
  },
  {
    id: "readonly",
    name: "只读演示",
    description: "1080p · 8M · 60fps · 只读",
    config: {
      maxSize: "1080",
      videoBitRate: "8M",
      maxFps: "60",
      noControl: true,
      stayAwake: true,
      turnScreenOff: false,
    },
  },
];

export const OPT_RES = [
  { value: "720", label: "720p" },
  { value: "1080", label: "1080p" },
  { value: "1440", label: "1440p" },
  { value: "native", label: "原始" },
];

export const OPT_BR = [
  { value: "2M", label: "2M" },
  { value: "4M", label: "4M" },
  { value: "8M", label: "8M" },
  { value: "16M", label: "16M" },
];

export const OPT_FPS = [
  { value: "30", label: "30fps" },
  { value: "60", label: "60fps" },
  { value: "90", label: "90fps" },
  { value: "120", label: "120fps" },
];

export const CAP_NAMES: Record<string, string> = {
  mirror: "投屏",
  control: "控制",
  screenshot: "截图",
  recording: "录制",
  wireless: "无线",
  installApp: "安装",
  uninstallApp: "卸载",
  logs: "日志",
  fileTransfer: "文件",
  automation: "自动化",
};

export const STATUS_NAMES: Record<string, string> = {
  online: "在线",
  offline: "离线",
  unauthorized: "未授权",
  busy: "忙碌",
  unknown: "未知",
};

export const CONN_NAMES: Record<string, string> = {
  usb: "USB",
  wifi: "WiFi",
  unknown: "未知",
};

export const SOURCE_NAMES: Record<string, string> = {
  system: "系统",
  adb: "ADB",
  scrcpy: "Scrcpy",
};

export const PAGE_TITLES: Record<string, string> = {
  dashboard: "仪表盘",
  devices: "设备管理",
  mirror: "投屏控制",
  logs: "运行日志",
  settings: "设置",
};
