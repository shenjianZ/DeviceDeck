import { useState } from "react";
import { AlertTriangle, Battery, RefreshCw, Usb, Wifi, X } from "lucide-react";
import { Badge } from "../ui/Badge";
import { useApp } from "../../context/AppContext";
import { CAP_NAMES, CONN_NAMES, STATUS_NAMES } from "../../data/constants";

export function DevicesPage() {
  const { state, dispatch } = useApp();
  const { devices, selectedDeviceId } = state;
  const [scanning, setScanning] = useState(false);

  const scan = () => {
    setScanning(true);
    setTimeout(() => setScanning(false), 800);
  };

  const selected = devices.find((d) => d.id === selectedDeviceId);

  return (
    <div className="col" style={{ gap: 12 }}>
      <div className="action-bar">
        <button className="btn btn-p" onClick={scan} disabled={scanning} type="button">
          <RefreshCw size={14} className={scanning ? "animate-spin" : ""} />
          {scanning ? "扫描中..." : "扫描设备"}
        </button>
        <span style={{ fontSize: 12, color: "var(--t2)" }}>
          共 {devices.length} 台设备 · {devices.filter((d) => d.status === "online").length} 在线
        </span>
      </div>

      <div className="grid4">
        {devices.map((device) => (
          <div
            key={device.id}
            className={`card${selectedDeviceId === device.id ? " selected" : ""}`}
            style={{ cursor: "pointer" }}
            onClick={() =>
              dispatch({
                type: "SELECT_DEVICE",
                id: selectedDeviceId === device.id ? null : device.id,
              })
            }
          >
            <div className="row" style={{ marginBottom: 6 }}>
              <span
                style={{
                  flex: 1,
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap",
                  fontWeight: 600,
                  fontSize: 13,
                }}
              >
                {device.name}
              </span>
            </div>
            <div className="row" style={{ marginBottom: 6 }}>
              <Badge variant={device.status}>{STATUS_NAMES[device.status]}</Badge>
            </div>
            <div className="row" style={{ fontSize: 11, color: "var(--t2)" }}>
              {device.connectionType === "wifi" ? <Wifi size={12} /> : <Usb size={12} />}
              {CONN_NAMES[device.connectionType]}
              {device.batteryLevel != null && (
                <>
                  <Battery size={12} />
                  {device.batteryLevel}%
                </>
              )}
            </div>
            <div style={{ fontSize: 11, color: "var(--t2)", marginTop: 4 }}>
              {device.brand} · Android {device.androidVersion}
            </div>
          </div>
        ))}
      </div>

      {selected && (
        <div className="card" style={{ marginTop: 4 }}>
          <div style={{ fontWeight: 600, fontSize: 14, marginBottom: 10 }}>
            {selected.name} — 设备详情
          </div>
          <div className="detail-grid">
            <span className="detail-label">Serial</span>
            <span className="detail-val mono">{selected.serial}</span>
            <span className="detail-label">型号</span>
            <span className="detail-val">{selected.model}</span>
            <span className="detail-label">品牌</span>
            <span className="detail-val">{selected.brand}</span>
            <span className="detail-label">平台</span>
            <span className="detail-val">{selected.platform}</span>
            <span className="detail-label">Android</span>
            <span className="detail-val">{selected.androidVersion}</span>
            <span className="detail-label">屏幕</span>
            <span className="detail-val">{selected.screenSize}</span>
            <span className="detail-label">电量</span>
            <span className="detail-val">{selected.batteryLevel != null ? `${selected.batteryLevel}%` : "—"}</span>
            <span className="detail-label">连接</span>
            <span className="detail-val">{CONN_NAMES[selected.connectionType]}</span>
          </div>

          {selected.status === "unauthorized" && (
            <div className="detail-notice" style={{ background: "var(--wrn-s)", color: "var(--wrn)" }}>
              <AlertTriangle size={16} />
              请在 Android 手机上确认 USB 调试授权弹窗，然后重新扫描设备。
            </div>
          )}
          {selected.status === "offline" && (
            <div className="detail-notice" style={{ background: "var(--err-s)", color: "var(--err)" }}>
              <X size={16} />
              设备离线。请检查 USB 连接或重新插拔数据线。
            </div>
          )}

          <div style={{ marginTop: 10 }}>
            <span className="detail-label">能力标签</span>
            <div style={{ marginTop: 4 }}>
              {Object.keys(CAP_NAMES).map((key) => (
                <span
                  key={key}
                  className={selected.capabilities.includes(key) ? "cap-tag" : "cap-tag off"}
                >
                  {CAP_NAMES[key]}
                </span>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
