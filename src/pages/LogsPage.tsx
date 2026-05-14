import { Trash2, FileText } from "lucide-react";
import { useLogStore } from "../stores/logStore";
import { Dropdown } from "../components/ui/Dropdown";
import { Badge } from "../components/ui/Badge";
import { SOURCE_NAMES } from "../lib/presets";

const SOURCE_FILTER_OPTIONS = [
  { value: "all", label: "全部来源" },
  { value: "system", label: "系统" },
  { value: "adb", label: "ADB" },
  { value: "scrcpy", label: "Scrcpy" },
];

const LEVEL_FILTER_OPTIONS = [
  { value: "all", label: "全部级别" },
  { value: "info", label: "Info" },
  { value: "warn", label: "Warn" },
  { value: "error", label: "Error" },
];

export function LogsPage() {
  const logs = useLogStore((s) => s.logs);
  const sourceFilter = useLogStore((s) => s.sourceFilter);
  const levelFilter = useLogStore((s) => s.levelFilter);
  const setFilter = useLogStore((s) => s.setFilter);
  const clearLogs = useLogStore((s) => s.clearLogs);

  const uniqueLogs = logs.filter(
    (log, index, list) => list.findIndex((item) => item.id === log.id) === index
  );

  const filteredLogs = uniqueLogs.filter((log) => {
    if (sourceFilter !== "all" && log.source !== sourceFilter) return false;
    if (levelFilter !== "all" && log.level !== levelFilter) return false;
    return true;
  });

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

  return (
    <div>
      <div className="action-bar">
        <Dropdown
          value={sourceFilter}
          onChange={(v) => setFilter(v, undefined)}
          options={SOURCE_FILTER_OPTIONS}
          className=""
        />
        <Dropdown
          value={levelFilter}
          onChange={(v) => setFilter(undefined, v)}
          options={LEVEL_FILTER_OPTIONS}
          className=""
        />
        <div style={{ flex: 1 }} />
        <span style={{ color: "var(--t2)", fontSize: 12 }}>
          {filteredLogs.length} 条记录
        </span>
        <button className="btn btn-d" onClick={() => void clearLogs()} disabled={uniqueLogs.length === 0} type="button">
          <Trash2 size={14} />
          清空
        </button>
      </div>

      {filteredLogs.length === 0 ? (
        <div className="empty">
          <FileText size={32} />
          <span>{uniqueLogs.length === 0 ? "暂无日志记录" : "没有匹配的日志"}</span>
        </div>
      ) : (
        <div className="log-table">
          <div className="log-row log-head">
            <span>时间</span>
            <span>来源</span>
            <span>级别</span>
            <span>设备</span>
            <span>消息</span>
          </div>
          <div style={{ maxHeight: "calc(100vh - 200px)", overflowY: "auto" }}>
            {filteredLogs.map((log, index) => (
              <div key={`${log.id}-${index}`} className="log-row">
                <span className="mono">{formatTime(log.time)}</span>
                <Badge variant={sourceBadgeVariant(log.source)}>
                  {SOURCE_NAMES[log.source] ?? log.source}
                </Badge>
                <Badge variant={levelBadgeVariant(log.level)}>
                  {log.level.toUpperCase()}
                </Badge>
                <span className="mono" style={{ fontSize: 11, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {log.deviceSerial || "—"}
                </span>
                <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {log.message}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
