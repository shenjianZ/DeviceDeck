export type UpdateStatus =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "downloaded"
  | "up-to-date"
  | "error";

export type UpdateSummary = {
  version: string;
  currentVersion: string;
  body?: string | null;
  date?: string | null;
};

export type UpdateState = {
  status: UpdateStatus;
  currentVersion: string;
  latestVersion: string | null;
  downloadedVersion: string | null;
  contentLength: number | null;
  downloadedBytes: number;
  error: string | null;
  availableUpdate: UpdateSummary | null;
};
