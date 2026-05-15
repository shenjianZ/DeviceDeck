import type { MirrorConfig, MirrorPreset } from "../types";

type TFn = (key: string) => string;

export const PRESET_CONFIGS: Omit<MirrorPreset, "name" | "description">[] = [
  {
    id: "smooth",
    config: {
      ...defaultAdvancedMirrorConfig(),
      maxSize: "720",
      videoBitRate: "4M",
      maxFps: "60",
      videoCodec: "h264",
      noControl: false,
      stayAwake: true,
      turnScreenOff: false,
      screenBlackMode: false,
    },
  },
  {
    id: "hd",
    config: {
      ...defaultAdvancedMirrorConfig(),
      maxSize: "1080",
      videoBitRate: "8M",
      maxFps: "60",
      videoCodec: "h264",
      noControl: false,
      stayAwake: true,
      turnScreenOff: false,
      screenBlackMode: false,
    },
  },
  {
    id: "ultra",
    config: {
      ...defaultAdvancedMirrorConfig(),
      maxSize: "native",
      videoBitRate: "32M",
      maxFps: "60",
      videoCodec: "h264",
      noControl: false,
      stayAwake: true,
      turnScreenOff: false,
      screenBlackMode: false,
    },
  },
  {
    id: "h265-max",
    config: {
      ...defaultAdvancedMirrorConfig(),
      maxSize: "native",
      videoBitRate: "50M",
      maxFps: "60",
      videoCodec: "h265",
      noControl: false,
      stayAwake: true,
      turnScreenOff: false,
      screenBlackMode: false,
    },
  },
];

export function defaultAdvancedMirrorConfig(): Pick<
  MirrorConfig,
  | "recordMode"
  | "recordFormat"
  | "recordDirectory"
  | "alwaysOnTop"
  | "windowBorderless"
  | "printFps"
  | "orientation"
  | "audioEnabled"
  | "audioSource"
  | "audioCodec"
  | "audioDuplicate"
  | "requireAudio"
> {
  return {
    recordMode: "off",
    recordFormat: "mp4",
    recordDirectory: "",
    alwaysOnTop: false,
    windowBorderless: false,
    printFps: false,
    orientation: "unlocked",
    audioEnabled: true,
    audioSource: "output",
    audioCodec: "opus",
    audioDuplicate: false,
    requireAudio: false,
  };
}

const PRESET_NAMES: Record<string, string> = {
  smooth: "presets.smooth.name",
  hd: "presets.hd.name",
  ultra: "presets.ultra.name",
  "h265-max": "presets.h265Max.name",
};

const PRESET_DESCS: Record<string, string> = {
  smooth: "presets.smooth.desc",
  hd: "presets.hd.desc",
  ultra: "presets.ultra.desc",
  "h265-max": "presets.h265Max.desc",
};

export function getPresets(t: TFn): MirrorPreset[] {
  return PRESET_CONFIGS.map((p) => ({
    ...p,
    name: t(PRESET_NAMES[p.id] ?? p.id),
    description: t(PRESET_DESCS[p.id] ?? ""),
  }));
}

export const OPT_RES_VALUES: ({ value: string; label: string } | { value: string; labelKey: string })[] = [
  { value: "720", label: "720p" },
  { value: "1080", label: "1080p" },
  { value: "1440", label: "1440p" },
  { value: "native", labelKey: "mirror:native" },
];

export function getOptRes(t: TFn): { value: string; label: string }[] {
  return OPT_RES_VALUES.map((o) =>
    "labelKey" in o ? { value: o.value, label: t((o as { labelKey: string }).labelKey) } : o as { value: string; label: string }
  );
}

export const OPT_BR = [
  { value: "2M", label: "2M" },
  { value: "4M", label: "4M" },
  { value: "8M", label: "8M" },
  { value: "16M", label: "16M" },
  { value: "24M", label: "24M" },
  { value: "32M", label: "32M" },
  { value: "50M", label: "50M" },
];

export const OPT_FPS = [
  { value: "30", label: "30fps" },
  { value: "60", label: "60fps" },
  { value: "90", label: "90fps" },
  { value: "120", label: "120fps" },
];

export function getOptCodec(t: TFn) {
  return [
    { value: "h264", label: `H.264 ${t("mirror:compatible")}` },
    { value: "h265", label: `H.265 ${t("mirror:highQuality")}` },
    { value: "av1", label: "AV1" },
  ];
}

export function getStatusNames(t: TFn): Record<string, string> {
  return {
    online: t("common:status.online"),
    offline: t("common:status.offline"),
    unauthorized: t("common:status.unauthorized"),
    busy: t("common:status.busy"),
    unknown: t("common:status.unknown"),
  };
}

export function getConnNames(t: TFn): Record<string, string> {
  return {
    usb: t("devices:usb"),
    wifi: t("devices:wifi"),
    unknown: t("common:status.unknown"),
  };
}

export function getCapNames(t: TFn): Record<string, string> {
  return {
    mirror: t("capabilities.mirror"),
    control: t("capabilities.control"),
    screenshot: t("capabilities.screenshot"),
    recording: t("capabilities.recording"),
    wireless: t("capabilities.wireless"),
    installApp: t("capabilities.installApp"),
    uninstallApp: t("capabilities.uninstallApp"),
    logs: t("capabilities.logs"),
    fileTransfer: t("capabilities.fileTransfer"),
    automation: t("capabilities.automation"),
  };
}

export function getSourceNames(t: TFn): Record<string, string> {
  return {
    system: t("logs:system"),
    adb: "ADB",
    scrcpy: "Scrcpy",
  };
}

export function getPageTitles(t: TFn): Record<string, string> {
  return {
    dashboard: t("sidebar:dashboard"),
    devices: t("sidebar:devices"),
    mirror: t("sidebar:mirror"),
    logs: t("sidebar:logs"),
    settings: t("sidebar:settings"),
  };
}

// Legacy static exports for backward compatibility (used by SettingsPage OPT_* imports)
export const OPT_RES = OPT_RES_VALUES.map((o): { value: string; label: string } =>
  "labelKey" in o ? { value: o.value, label: o.value === "native" ? "Native" : o.value } : o
);
export const OPT_CODEC = [
  { value: "h264", label: "H.264" },
  { value: "h265", label: "H.265" },
  { value: "av1", label: "AV1" },
];
