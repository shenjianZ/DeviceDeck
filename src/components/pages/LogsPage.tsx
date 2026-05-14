import { useState } from "react";
import { FileText, X } from "lucide-react";
import { Badge } from "../ui/Badge";
import { Dropdown } from "../ui/Dropdown";
import { SOURCE_NAMES } from "../../data/constants";
import { useApp } from "../../context/AppContext";

export function LogsPage() {
  const { state, dispatch } = useApp();
  const { logs } = state;
  const [sourceFilter, setSourceFilter] = useState("all");
  const [levelFilter, setLevelFilter] = useState("all");

  const filteredLogs = logs.filter(
    (log) =>
      (sourceFilter === "all" || log.source === sourceFilter) &&
      (levelFilter === "all" || log.level === levelFilter)
  );

  return (
    <div className="col" style={{ gap: 10 }}>
      <div className="action-bar">
        <Dropdown
          value={sourceFilter}
          onChange={setSourceFilter}
          options={[
            { value: "all", label: "全部来源" },
            { value: "system", label: "系统" },
            { value: "adb", label: "ADB" },
            { value: "scrcpy", label: "Scrcpy" },
          ]}
          className="w-[120px]"
        />
        <Dropdown
          value={levelFilter}
          onChange={setLevelFilter}
          options={[
            { value: "all", label: "全部等级" },
            { value: "info", label: "Info" },
            { value: "warn", label: "Warn" },
            { value: "error", label: "Error" },
          ]}
          className="w-[120px]"
        />
        <span style={{ fontSize: 11, color: "var(--t2)" }}>{filteredLogs.length} 条日志</span>
        <button
          className="btn btn-g"
          style={{ marginLeft: "auto" }}
          onClick={() => dispatch({ type: "CLEAR_LOGS" })}
          type="button"
        >
          <X size={13} />
          清空
        </button>
      </div>

      <div className="log-table">
        <div className="log-row log-head">
          <span>时间</span>
          <span>来源</span>
          <span>等级</span>
          <span>设备</span>
          <span>消息</span>
        </div>
        {filteredLogs.length === 0 ? (
          <div className="empty" style={{ padding: 24 }}>
            <FileText size={28} />
            暂无日志
          </div>
        ) : (
          filteredLogs.map((log) => (
            <div key={log.id} className="log-row">
              <span className="mono" style={{ fontSize: 11, color: "var(--t2)" }}>
                {log.time.toLocaleTimeString("zh-CN", { hour12: false })}
              </span>
              <Badge
                variant={log.source === "system" ? "system" : log.source}
                className="!text-[10px] !px-1.5 !py-px"
              >
                {SOURCE_NAMES[log.source]}
              </Badge>
              <Badge variant={log.level} className="!text-[10px] !px-1.5 !py-px">
                {log.level.toUpperCase()}
              </Badge>
              <span
                className="mono"
                style={{
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap",
                  fontSize: 11,
                  color: "var(--t2)",
                }}
              >
                {log.deviceSerial || "—"}
              </span>
              <span style={{ fontSize: 12 }}>{log.message}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
