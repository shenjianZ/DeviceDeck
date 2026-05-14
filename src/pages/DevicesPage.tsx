import { RefreshCw, Smartphone, Wifi, Usb, ChevronRight, X as XIcon } from "lucide-react";
import { useDeviceStore } from "../stores/deviceStore";
import { useMirrorStore } from "../stores/mirrorStore";
import { usePageStore } from "../stores/pageStore";
import { Badge } from "../components/ui/Badge";
import { STATUS_NAMES, CONN_NAMES, CAP_NAMES } from "../lib/presets";
import type { DeviceInfo } from "../types";

export function DevicesPage() {
  const devices = useDeviceStore((s) => s.devices);
  const selectedDeviceId = useDeviceStore((s) => s.selectedDeviceId);
  const isScanning = useDeviceStore((s) => s.isScanning);
  const scanDevices = useDeviceStore((s) => s.scanDevices);
  const selectDevice = useDeviceStore((s) => s.selectDevice);
  const isWirelessBusy = useDeviceStore((s) => s.isWirelessBusy);
  const wirelessMessage = useDeviceStore((s) => s.wirelessMessage);
  const startWirelessMirror = useMirrorStore((s) => s.startWirelessMirror);
  const isStartingMirror = useMirrorStore((s) => s.isStarting);
  const setPage = usePageStore((s) => s.setPage);

  const selectedDevice = devices.find((d) => d.id === selectedDeviceId) ?? null;

  const handleSelect = (device: DeviceInfo) => {
    if (selectedDeviceId === device.id) {
      selectDevice(null);
    } else {
      selectDevice(device.id);
    }
  };

  const startMirrorFor = (_serial: string) => {
    setPage("mirror");
  };

  const startWirelessMirrorFor = async (serial: string) => {
    await startWirelessMirror(serial, 5555);
    setPage("mirror");
    await scanDevices();
  };

  const statusBadgeVariant = (status: string): "online" | "offline" | "unauthorized" | "unknown" => {
    if (status === "online") return "online";
    if (status === "offline") return "offline";
    if (status === "unauthorized") return "unauthorized";
    return "unknown";
  };

  return (
    <div>
      <div className="action-bar">
        <button className="btn btn-p" onClick={scanDevices} disabled={isScanning} type="button">
          <RefreshCw size={14} className={isScanning ? "spin" : ""} />
          {isScanning ? "扫描中..." : "扫描设备"}
        </button>
        <span style={{ color: "var(--t2)", fontSize: 12 }}>
          共 {devices.length} 台设备
        </span>
      </div>

      {wirelessMessage && (
        <div className="detail-notice" style={{ background: "var(--ok-s)", color: "var(--ok)", marginBottom: 12 }}>
          {wirelessMessage}
        </div>
      )}

      {devices.length === 0 ? (
        <div className="empty">
          <Smartphone size={32} />
          <span>未检测到设备，请连接设备后点击「扫描设备」</span>
        </div>
      ) : (
        <div style={{ display: "flex", gap: 16 }}>
          <div style={{ flex: 1, minWidth: 0 }}>
            <div className="grid2">
              {devices.map((d) => (
                <div
                  key={d.id}
                  className={`card${selectedDeviceId === d.id ? " selected" : ""}`}
                  style={{ cursor: "pointer" }}
                  onClick={() => handleSelect(d)}
                >
                  <div className="row" style={{ marginBottom: 8 }}>
                    {d.connectionType === "wifi" ? <Wifi size={14} /> : <Usb size={14} />}
                    <span style={{ fontWeight: 600, flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                      {d.name || d.model}
                    </span>
                    <Badge variant={statusBadgeVariant(d.status)}>
                      {STATUS_NAMES[d.status] ?? d.status}
                    </Badge>
                  </div>
                  <div style={{ color: "var(--t2)", fontSize: 11, marginBottom: 6 }} className="mono">
                    {d.serial}
                  </div>
                  <div className="row" style={{ flexWrap: "wrap", gap: 3 }}>
                    {d.capabilities.slice(0, 4).map((cap) => (
                      <span key={cap} className="cap-tag">
                        {CAP_NAMES[cap] ?? cap}
                      </span>
                    ))}
                    {d.capabilities.length > 4 && (
                      <span className="cap-tag off">+{d.capabilities.length - 4}</span>
                    )}
                  </div>
                  <div className="row" style={{ marginTop: 8, justifyContent: "flex-end" }}>
                    <ChevronRight size={14} style={{ color: "var(--t2)" }} />
                  </div>
                </div>
              ))}
            </div>
          </div>

          {selectedDevice && (
            <div style={{ width: 300, flexShrink: 0 }}>
              <div className="card" style={{ position: "sticky", top: 0 }}>
                <div className="row" style={{ marginBottom: 12, justifyContent: "space-between" }}>
                  <span style={{ fontWeight: 600, fontSize: 14 }}>{selectedDevice.name || selectedDevice.model}</span>
                  <button
                    className="btn btn-g"
                    style={{ padding: 2, minHeight: "auto" }}
                    onClick={() => selectDevice(null)}
                    type="button"
                  >
                    <XIcon size={14} />
                  </button>
                </div>

                <Badge variant={statusBadgeVariant(selectedDevice.status)}>
                  {STATUS_NAMES[selectedDevice.status] ?? selectedDevice.status}
                </Badge>

                <div className="detail-grid" style={{ marginBottom: 12 }}>
                  <span className="detail-label">型号</span>
                  <span className="detail-val">{selectedDevice.model}</span>
                  <span className="detail-label">品牌</span>
                  <span className="detail-val">{selectedDevice.brand}</span>
                  <span className="detail-label">序列号</span>
                  <span className="detail-val mono" style={{ fontSize: 11 }}>{selectedDevice.serial}</span>
                  <span className="detail-label">连接</span>
                  <span className="detail-val">{CONN_NAMES[selectedDevice.connectionType] ?? selectedDevice.connectionType}</span>
                  {selectedDevice.androidVersion && (
                    <>
                      <span className="detail-label">Android</span>
                      <span className="detail-val">{selectedDevice.androidVersion}</span>
                    </>
                  )}
                  {selectedDevice.screenSize && (
                    <>
                      <span className="detail-label">屏幕</span>
                      <span className="detail-val">{selectedDevice.screenSize}</span>
                    </>
                  )}
                  {selectedDevice.batteryLevel != null && (
                    <>
                      <span className="detail-label">电量</span>
                      <span className="detail-val">{selectedDevice.batteryLevel}%</span>
                    </>
                  )}
                </div>

                <div style={{ marginBottom: 12 }}>
                  <div style={{ color: "var(--t2)", fontSize: 11, fontWeight: 600, marginBottom: 6, textTransform: "uppercase", letterSpacing: "0.04em" }}>
                    能力
                  </div>
                  <div className="row" style={{ flexWrap: "wrap", gap: 3 }}>
                    {selectedDevice.capabilities.map((cap) => (
                      <span key={cap} className="cap-tag">
                        {CAP_NAMES[cap] ?? cap}
                      </span>
                    ))}
                  </div>
                </div>

                <button
                  className="btn btn-p"
                  style={{ width: "100%", justifyContent: "center" }}
                  onClick={() => startMirrorFor(selectedDevice.serial)}
                  disabled={selectedDevice.status !== "online"}
                  type="button"
                >
                  投屏
                </button>

                <button
                  className="btn btn-s"
                  style={{ width: "100%", justifyContent: "center", marginTop: 8 }}
                  onClick={() => startWirelessMirrorFor(selectedDevice.serial)}
                  disabled={selectedDevice.status !== "online" || isWirelessBusy || isStartingMirror}
                  type="button"
                >
                  一键无线投屏
                </button>

                {selectedDevice.status !== "online" && (
                  <div className="detail-notice" style={{ background: "var(--wrn-s)", color: "var(--wrn)" }}>
                    设备不在线，无法投屏
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      )}

      <style>{`
        @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
        .spin { animation: spin 1s linear infinite; }
      `}</style>
    </div>
  );
}
