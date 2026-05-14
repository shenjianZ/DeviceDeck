import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";
import type { UpdateSummary } from "./update-types";

function toSummary(update: Update): UpdateSummary {
  return {
    version: update.version,
    currentVersion: update.currentVersion,
    body: update.body,
    date: update.date,
  };
}

export async function checkForAppUpdate() {
  const update = await check();
  return update ? { update, summary: toSummary(update) } : null;
}

export async function downloadAppUpdate(
  update: Update,
  onEvent: (event: DownloadEvent) => void,
) {
  await update.download(onEvent);
  return toSummary(update);
}

export async function installAppUpdate(update: Update) {
  await update.install();
}
