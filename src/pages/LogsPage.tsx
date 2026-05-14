import { useEffect, useState } from "react";
import { Trash2, FileText, RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useLogStore } from "../stores/logStore";
import { Dropdown } from "../components/ui/Dropdown";
import { Badge } from "../components/ui/Badge";
import { Pagination } from "../components/ui/Pagination";
import { getSourceNames } from "../lib/presets";

export function LogsPage() {
  const { t } = useTranslation(["logs", "common"]);
  const logs = useLogStore((s) => s.logs);
  const total = useLogStore((s) => s.total);
  const page = useLogStore((s) => s.page);
  const pageSize = useLogStore((s) => s.pageSize);
  const totalPages = useLogStore((s) => s.totalPages);
  const isLoading = useLogStore((s) => s.isLoading);
  const sourceFilter = useLogStore((s) => s.sourceFilter);
  const levelFilter = useLogStore((s) => s.levelFilter);
  const setFilter = useLogStore((s) => s.setFilter);
  const clearLogs = useLogStore((s) => s.clearLogs);
  const loadPaginatedLogs = useLogStore((s) => s.loadPaginatedLogs);
  const startListening = useLogStore((s) => s.startListening);
  const [expandedLogId, setExpandedLogId] = useState<string | null>(null);

  const sourceNames = getSourceNames(t);

  const sourceFilterOptions = [
    { value: "all", label: t("logs:allSource") },
    { value: "system", label: t("logs:system") },
    { value: "adb", label: "ADB" },
    { value: "scrcpy", label: "Scrcpy" },
  ];

  const levelFilterOptions = [
    { value: "all", label: t("logs:allLevel") },
    { value: "info", label: "Info" },
    { value: "warn", label: "Warn" },
    { value: "error", label: "Error" },
  ];

  useEffect(() => {
    loadPaginatedLogs(1);
    const cleanup = startListening();
    return () => {
      cleanup.then((fn) => fn());
    };
  }, [loadPaginatedLogs, startListening]);

  const levelBadgeVariant = (level: string): "info" | "warn" | "error" => {
    if (level === "warn") return "warn";
    if (level === "error") return "error";
    return "info";
  };

  const sourceBadgeVariant = (source: string): "system" | "adb" | "scrcpy" => {
    if (source === "adb") return "adb";
    if (source === "scrcpy") return "scrcpy";
    return "system";
  };

  const formatTime = (ts: number): string => {
    const d = new Date(ts);
    const pad = (n: number) => n.toString().padStart(2, "0");
    return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
  };

  const toggleLogDetail = (id: string) => {
    setExpandedLogId((current) => (current === id ? null : id));
  };

  return (
    <div className="logs-page">
      <div className="action-bar">
        <Dropdown
          className="toolbar-select"
          value={sourceFilter}
          onChange={(v) => setFilter(v, undefined)}
          options={sourceFilterOptions}
        />
        <Dropdown
          className="toolbar-select"
          value={levelFilter}
          onChange={(v) => setFilter(undefined, v)}
          options={levelFilterOptions}
        />
        <div style={{ flex: 1 }} />
        <button
          className="btn btn-s"
          onClick={() => loadPaginatedLogs(page)}
          disabled={isLoading}
          type="button"
        >
          <RefreshCw size={14} className={isLoading ? "spin" : ""} />
          {t("logs:refresh")}
        </button>
        <button
          className="btn btn-d"
          onClick={() => void clearLogs()}
          disabled={logs.length === 0}
          type="button"
        >
          <Trash2 size={14} />
          {t("logs:clear")}
        </button>
      </div>

      {logs.length === 0 ? (
        <div className="empty">
          <FileText size={32} />
          <span>{total === 0 ? t("logs:noLogs") : t("common:empty.loading")}</span>
        </div>
      ) : (
        <div className="logs-content">
          <div className="log-table">
            <div className="log-row log-head">
              <span>{t("logs:time")}</span>
              <span>{t("logs:source")}</span>
              <span>{t("logs:level")}</span>
              <span>{t("logs:device")}</span>
              <span>{t("logs:message")}</span>
            </div>
            <div
              className="log-table-body"
            >
              {logs.map((log) => {
                const sourceName = sourceNames[log.source] ?? log.source;
                const time = formatTime(log.time);
                const expanded = expandedLogId === log.id;

                return (
                <div key={log.id} className="log-item">
                  <div
                    className="log-row log-row-action"
                    role="button"
                    tabIndex={0}
                    aria-expanded={expanded}
                    aria-label={`${time} ${sourceName} ${log.level.toUpperCase()} ${log.deviceSerial || ""} ${log.message}`}
                    onClick={() => toggleLogDetail(log.id)}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        toggleLogDetail(log.id);
                      }
                    }}
                  >
                  <span className="mono">{time}</span>
                  <Badge variant={sourceBadgeVariant(log.source)}>
                    {sourceName}
                  </Badge>
                  <Badge variant={levelBadgeVariant(log.level)}>
                    {log.level.toUpperCase()}
                  </Badge>
                  <span
                    className="mono"
                    style={{
                      fontSize: 11,
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {log.deviceSerial || "—"}
                  </span>
                  <span
                    style={{
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {log.message}
                  </span>
                  </div>
                  {expanded && (
                    <div className="log-detail" data-testid={`log-detail-${log.id}`}>
                      <div className="log-detail-title">{t("logs:detailTitle")}</div>
                      <div className="log-detail-meta">
                        <span>{time}</span>
                        <span>{sourceName}</span>
                        <span>{log.level.toUpperCase()}</span>
                        <span className="mono">{log.deviceSerial || "-"}</span>
                      </div>
                      <div className="log-detail-message">{log.message}</div>
                    </div>
                  )}
                </div>
                );
              })}
            </div>
          </div>

          <Pagination
            page={page}
            totalPages={totalPages}
            total={total}
            pageSize={pageSize}
            isLoading={isLoading}
            onPageChange={(targetPage) => loadPaginatedLogs(targetPage)}
          />
        </div>
      )}

      <style>{`
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
        .spin { animation: spin 1s linear infinite; }
      `}</style>
    </div>
  );
}
