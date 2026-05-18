import i18n from "../i18n";

export function formatTimeAgo(timestamp: number): string {
  const diff = Date.now() - timestamp;
  const seconds = Math.floor(diff / 1000);
  if (seconds < 60) return i18n.t("common:timeAgo.seconds", { count: seconds });
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return i18n.t("common:timeAgo.minutes", { count: minutes });
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return i18n.t("common:timeAgo.hours", { count: hours });
  const days = Math.floor(hours / 24);
  return i18n.t("common:timeAgo.days", { count: days });
}
