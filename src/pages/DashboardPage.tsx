import {
  Activity,
  Cpu,
  Monitor,
  RefreshCw,
  Smartphone,
  Terminal,
  ArrowRight,
} from "lucide-react";
import { useDeviceStore } from "../stores/deviceStore";
import { useMirrorStore } from "../stores/mirrorStore";
import { useLogStore } from "../stores/logStore";
import { usePageStore } from "../stores/pageStore";
import { Badge } from "../components/ui/Badge";
import { STATUS_NAMES, SOURCE_NAMES } from "../lib/presets";
import { formatTimeAgo } from "../lib/format";
import type { Page } from "../types";

export function DashboardPage() {
  const devices = useDeviceStore((s) => s.devices);
  const environment = useDeviceStore((s) => s.environment);
  const isScanning = useDeviceStore((s) => s.isScanning);
  const scanDevices = useDeviceStore((s) => s.scanDevices);
  const sessions = useMirrorStore((s) => s.sessions);
  const logs = useLogStore((s) => s.logs);
  const setPage = usePageStore((s) => s.setPage);

  const onlineCount = devices.filter((d) => d.status === "online").length;
  const runningSessions = sessions.filter((s) => s.status === "running").length;
  const recentLogs = logs.slice(-8).reverse();

  const goTo = (page: Page) => () => setPage(page);

  return (
    <div>
      <h2 className="sec-title flush">环境状态</h2>
      <div className="grid2">
        <div className="env-row">
          <div className="env-icon" style={{ background: environment?.adb?.available ? "var(--ok-s)" : "var(--err-s)", color: environment?.adb?.available ? "var(--ok)" : "var(--err)" }}>
            <Terminal size={18} />
          </div>
          <div className="col" style={{ flex: 1 }}>
            <div style={{ fontWeight: 600, fontSize: 13 }}>ADB</div>
            <div style={{ color: "var(--t2)", fontSize: 12 }}>
              {environment?.adb?.available
                ? environment.adb.version ?? "可用"
                : environment?.adb?.message ?? "检测中..."}
            </div>
          </div>
          {environment?.adb?.available ? (
            <Badge variant="online">正常</Badge>
          ) : (
            <Badge variant="offline">不可用</Badge>
          )}
        </div>
        <div className="env-row">
          <div className="env-icon" style={{ background: environment?.scrcpy?.available ? "var(--ok-s)" : "var(--err-s)", color: environment?.scrcpy?.available ? "var(--ok)" : "var(--err)" }}>
            <Monitor size={18} />
          </div>
          <div className="col" style={{ flex: 1 }}>
            <div style={{ fontWeight: 600, fontSize: 13 }}>Scrcpy</div>
            <div style={{ color: "var(--t2)", fontSize: 12 }}>
              {environment?.scrcpy?.available
                ? environment.scrcpy.version ?? "可用"
                : environment?.scrcpy?.message ?? "检测中..."}
            </div>
          </div>
          {environment?.scrcpy?.available ? (
            <Badge variant="online">正常</Badge>
          ) : (
            <Badge variant="offline">不可用</Badge>
          )}
        </div>
      </div>

      <h2 className="sec-title">概览</h2>
      <div className="grid3">
        <div className="card">
          <div className="card-title">已连接设备</div>
          <div className="card-val">{devices.length}</div>
          <div style={{ color: "var(--t2)", fontSize: 12, marginTop: 4 }}>
            {onlineCount} 在线
          </div>
        </div>
        <div className="card">
          <div className="card-title">投屏会话</div>
          <div className="card-val">{runningSessions}</div>
          <div style={{ color: "var(--t2)", fontSize: 12, marginTop: 4 }}>
            运行中
          </div>
        </div>
        <div className="card">
          <div className="card-title">日志条目</div>
          <div className="card-val">{logs.length}</div>
          <div style={{ color: "var(--t2)", fontSize: 12, marginTop: 4 }}>
            总记录
          </div>
        </div>
      </div>

      <h2 className="sec-title">快捷操作</h2>
      <div className="row" style={{ gap: 8 }}>
        <button className="btn btn-p" onClick={scanDevices} disabled={isScanning} type="button">
          <RefreshCw size={14} className={isScanning ? "spin" : ""} />
          {isScanning ? "扫描中..." : "扫描设备"}
        </button>
        <button className="btn btn-s" onClick={goTo("devices")} type="button">
          <Smartphone size={14} />
          设备管理
          <ArrowRight size={12} />
        </button>
        <button className="btn btn-s" onClick={goTo("mirror")} type="button">
          <Monitor size={14} />
          投屏控制
          <ArrowRight size={12} />
        </button>
        <button className="btn btn-s" onClick={goTo("logs")} type="button">
          <Activity size={14} />
          运行日志
          <ArrowRight size={12} />
        </button>
      </div>

      {devices.length > 0 && (
        <>
          <h2 className="sec-title">设备列表</h2>
          <div className="grid3">
            {devices.map((d) => (
              <div key={d.id} className="card" style={{ cursor: "pointer" }} onClick={goTo("devices")}>
                <div className="row" style={{ marginBottom: 6 }}>
                  <Smartphone size={14} />
                  <span style={{ fontWeight: 600 }}>{d.name || d.model}</span>
                </div>
                <div className="row">
                  <Badge variant={d.status === "online" ? "online" : d.status === "offline" ? "offline" : "unauthorized"}>
                    {STATUS_NAMES[d.status] ?? d.status}
                  </Badge>
                  <span style={{ color: "var(--t2)", fontSize: 11 }}>{d.serial}</span>
                </div>
              </div>
            ))}
          </div>
        </>
      )}

      {recentLogs.length > 0 && (
        <>
          <h2 className="sec-title">最近日志</h2>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {recentLogs.map((log) => (
              <div key={log.id} className="row" style={{ fontSize: 12, padding: "4px 0", borderBottom: "1px solid var(--bd)" }}>
                <span className="mono" style={{ color: "var(--t2)", width: 64, flexShrink: 0 }}>
                  {formatTimeAgo(log.time)}
                </span>
                <Badge variant={log.source === "system" ? "system" : log.source === "adb" ? "adb" : "scrcpy"}>
                  {SOURCE_NAMES[log.source] ?? log.source}
                </Badge>
                <span style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {log.message}
                </span>
              </div>
            ))}
          </div>
        </>
      )}

      {devices.length === 0 && logs.length === 0 && (
        <div className="empty" style={{ marginTop: 24 }}>
          <Cpu size={32} />
          <span>暂无数据，点击「扫描设备」开始</span>
        </div>
      )}

      <style>{`
        @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
        .spin { animation: spin 1s linear infinite; }
      `}</style>
    </div>
  );
}
