export interface MirrorConfig {
  maxSize: string;
  videoBitRate: string;
  maxFps: string;
  noControl: boolean;
  stayAwake: boolean;
  turnScreenOff: boolean;
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
