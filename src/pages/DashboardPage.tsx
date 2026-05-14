import {
  Activity,
  Cpu,
  Monitor,
  RefreshCw,
  Smartphone,
  Terminal,
  ArrowRight,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { useDeviceStore } from "../stores/deviceStore";
import { useMirrorStore } from "../stores/mirrorStore";
import { useLogStore } from "../stores/logStore";
import { usePageStore } from "../stores/pageStore";
import { Badge } from "../components/ui/Badge";
import { getStatusNames, getSourceNames } from "../lib/presets";
import { formatTimeAgo } from "../lib/format";
import type { Page } from "../types";

export function DashboardPage() {
  const { t } = useTranslation(["dashboard", "common", "logs"]);
  const devices = useDeviceStore((s) => s.devices);
  const environment = useDeviceStore((s) => s.environment);
  const isScanning = useDeviceStore((s) => s.isScanning);
  const scanDevices = useDeviceStore((s) => s.scanDevices);
  const sessions = useMirrorStore((s) => s.sessions);
  const logs = useLogStore((s) => s.logs);
  const setPage = usePageStore((s) => s.setPage);

  const statusNames = getStatusNames(t);
  const sourceNames = getSourceNames(t);

  const onlineCount = devices.filter((d) => d.status === "online").length;
  const runningSessions = sessions.filter((s) => s.status === "running").length;
  const recentLogs = logs.slice(-8).reverse();

  const goTo = (page: Page) => () => setPage(page);

  return (
    <div>
      <h2 className="sec-title flush">{t("dashboard:envStatus")}</h2>
      <div className="grid2">
        <div className="env-row">
          <div className="env-icon" style={{ background: environment?.adb?.available ? "var(--ok-s)" : "var(--err-s)", color: environment?.adb?.available ? "var(--ok)" : "var(--err)" }}>
            <Terminal size={18} />
          </div>
          <div className="col" style={{ flex: 1 }}>
            <div style={{ fontWeight: 600, fontSize: 13 }}>ADB</div>
            <div style={{ color: "var(--t2)", fontSize: 12 }}>
              {environment?.adb?.available
                ? environment.adb.version ?? t("dashboard:available")
                : environment?.adb?.message ?? t("dashboard:detecting")}
            </div>
          </div>
          {environment?.adb?.available ? (
            <Badge variant="online">{t("common:status.normal")}</Badge>
          ) : (
            <Badge variant="offline">{t("common:status.unavailable")}</Badge>
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
                ? environment.scrcpy.version ?? t("dashboard:available")
                : environment?.scrcpy?.message ?? t("dashboard:detecting")}
            </div>
          </div>
          {environment?.scrcpy?.available ? (
            <Badge variant="online">{t("common:status.normal")}</Badge>
          ) : (
            <Badge variant="offline">{t("common:status.unavailable")}</Badge>
          )}
        </div>
      </div>

      <h2 className="sec-title">{t("dashboard:overview")}</h2>
      <div className="grid3">
        <div className="card">
          <div className="card-title">{t("dashboard:totalDevices")}</div>
          <div className="card-val">{devices.length}</div>
          <div style={{ color: "var(--t2)", fontSize: 12, marginTop: 4 }}>
            {onlineCount} {t("dashboard:onlineCount")}
          </div>
        </div>
        <div className="card">
          <div className="card-title">{t("dashboard:activeSessions")}</div>
          <div className="card-val">{runningSessions}</div>
          <div style={{ color: "var(--t2)", fontSize: 12, marginTop: 4 }}>
            {t("dashboard:running")}
          </div>
        </div>
        <div className="card">
          <div className="card-title">{t("dashboard:logEntries")}</div>
          <div className="card-val">{logs.length}</div>
          <div style={{ color: "var(--t2)", fontSize: 12, marginTop: 4 }}>
            {t("dashboard:totalRecords")}
          </div>
        </div>
      </div>

      <h2 className="sec-title">{t("dashboard:quickActions")}</h2>
      <div className="row" style={{ gap: 8 }}>
        <button className="btn btn-p" onClick={() => scanDevices()} disabled={isScanning} type="button">
          <RefreshCw size={14} className={isScanning ? "spin" : ""} />
          {isScanning ? t("dashboard:scanning") : t("dashboard:scanDevices")}
        </button>
        <button className="btn btn-s" onClick={goTo("devices")} type="button">
          <Smartphone size={14} />
          {t("dashboard:deviceManagement")}
          <ArrowRight size={12} />
        </button>
        <button className="btn btn-s" onClick={goTo("mirror")} type="button">
          <Monitor size={14} />
          {t("dashboard:mirrorControl")}
          <ArrowRight size={12} />
        </button>
        <button className="btn btn-s" onClick={goTo("logs")} type="button">
          <Activity size={14} />
          {t("dashboard:runLogs")}
          <ArrowRight size={12} />
        </button>
      </div>

      {devices.length > 0 && (
        <>
          <h2 className="sec-title">{t("dashboard:deviceList")}</h2>
          <div className="grid3">
            {devices.map((d) => (
              <div key={d.id} className="card" style={{ cursor: "pointer" }} onClick={goTo("devices")}>
                <div className="row" style={{ marginBottom: 6 }}>
                  <Smartphone size={14} />
                  <span style={{ fontWeight: 600 }}>{d.name || d.model}</span>
                </div>
                <div className="row">
                  <Badge variant={d.status === "online" ? "online" : d.status === "offline" ? "offline" : "unauthorized"}>
                    {statusNames[d.status] ?? d.status}
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
          <h2 className="sec-title">{t("dashboard:recentLogs")}</h2>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {recentLogs.map((log) => (
              <div key={log.id} className="row" style={{ fontSize: 12, padding: "4px 0", borderBottom: "1px solid var(--bd)" }}>
                <span className="mono" style={{ color: "var(--t2)", width: 64, flexShrink: 0 }}>
                  {formatTimeAgo(log.time)}
                </span>
                <Badge variant={log.source === "system" ? "system" : log.source === "adb" ? "adb" : "scrcpy"}>
                  {sourceNames[log.source] ?? log.source}
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
          <span>{t("dashboard:noData")}</span>
        </div>
      )}

      <style>{`
        @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
        .spin { animation: spin 1s linear infinite; }
      `}</style>
    </div>
  );
}
