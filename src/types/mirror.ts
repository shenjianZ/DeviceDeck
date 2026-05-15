export type RecordMode = "off" | "window" | "background";
export type RecordFormat = "mp4" | "mkv";
export type MirrorOrientation = "unlocked" | "0" | "90" | "180" | "270";
export type AudioSource =
  | "output"
  | "playback"
  | "mic"
  | "mic-camcorder"
  | "voice-recognition"
  | "voice-communication"
  | "voice-performance";
export type AudioCodec = "opus" | "aac" | "flac" | "raw";

export interface MirrorConfig {
  maxSize: string;
  videoBitRate: string;
  maxFps: string;
  videoCodec: string;
  noControl: boolean;
  stayAwake: boolean;
  turnScreenOff: boolean;
  screenBlackMode: boolean;
  recordMode: RecordMode;
  recordFormat: RecordFormat;
  recordDirectory: string;
  alwaysOnTop: boolean;
  windowBorderless: boolean;
  printFps: boolean;
  orientation: MirrorOrientation;
  audioEnabled: boolean;
  audioSource: AudioSource;
  audioCodec: AudioCodec;
  audioDuplicate: boolean;
  requireAudio: boolean;
}

export type SessionStatus = "running" | "stopped" | "failed";

export interface MirrorSession {
  id: string;
  deviceSerial: string;
  platform: string;
  processId?: number | null;
  status: SessionStatus;
  startedAt: number;
  stoppedAt?: number | null;
  config: MirrorConfig;
}

export interface MirrorPreset {
  id: string;
  name: string;
  description: string;
  config: MirrorConfig;
}
