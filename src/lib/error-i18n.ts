import i18n from "../i18n";
import type { AppError } from "../types";

export function localizeErrorMessage(error: AppError): string {
  const code = error.code;
  const entry = i18n.t(`errors:${code}.message`, { defaultValue: null });
  if (entry) return entry;
  return error.message;
}

export function localizeErrorSuggestion(error: AppError): string | undefined {
  const code = error.code;

  if (code === "ADB_NOT_FOUND") {
    const platformKey = navigator.platform.toLowerCase().includes("linux")
      ? "errors:ADB_NOT_FOUND_LINUX_ARM64.suggestion"
      : "errors:ADB_NOT_FOUND.suggestion";
    return i18n.t(platformKey) || undefined;
  }
  if (code === "SCRCPY_NOT_FOUND") {
    const platformKey = navigator.platform.toLowerCase().includes("linux")
      ? "errors:SCRCPY_NOT_FOUND_LINUX_ARM64.suggestion"
      : "errors:SCRCPY_NOT_FOUND.suggestion";
    return i18n.t(platformKey) || undefined;
  }

  const entry = i18n.t(`errors:${code}.suggestion`, { defaultValue: null });
  if (entry) return entry;
  return error.suggestion || undefined;
}
