import { RefreshCw, Smartphone, Wifi, Usb, ChevronRight, X as XIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useDeviceStore } from "../stores/deviceStore";
import { useMirrorStore } from "../stores/mirrorStore";
import { usePageStore } from "../stores/pageStore";
import { Badge } from "../components/ui/Badge";
import { getStatusNames, getConnNames, getCapNames } from "../lib/presets";
import type { DeviceInfo } from "../types";

export function DevicesPage() {
  const { t } = useTranslation(["devices", "common"]);
  const devices = useDeviceStore((s) => s.devices);
  const selectedDeviceId = useDeviceStore((s) => s.selectedDeviceId);
  const isScanning = useDeviceStore((s) => s.isScanning);
  const scanDevices = useDeviceStore((s) => s.scanDevices);
  const selectDevice = useDeviceStore((s) => s.selectDevice);
  const isWirelessBusy = useDeviceStore((s) => s.isWirelessBusy);
  const startWirelessMirror = useMirrorStore((s) => s.startWirelessMirror);
  const isStartingMirror = useMirrorStore((s) => s.isStarting);
  const setPage = usePageStore((s) => s.setPage);

  const statusNames = getStatusNames(t);
  const connNames = getConnNames(t);
  const capNames = getCapNames(t);

  const selectedDevice = devices.find((d) => d.id === selectedDeviceId) ?? null;

  const handleSelect = (device: DeviceInfo) => {
    selectDevice(selectedDeviceId === device.id ? null : device.id);
  };

  const startMirrorFor = () => {
    setPage("mirror");
  };

  const switchUsbToWifiAndMirror = async (serial: string) => {
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
        <button className="btn btn-p" onClick={() => scanDevices()} disabled={isScanning} type="button">
          <RefreshCw size={14} className={isScanning ? "spin" : ""} />
          {isScanning ? t("common:buttons.scanning") : t("common:buttons.scan")}
        </button>
        <span style={{ color: "var(--t2)", fontSize: 12 }}>
          {t("devices:deviceCount", { count: devices.length })}
        </span>
      </div>

      {devices.length === 0 ? (
        <div className="empty">
          <Smartphone size={32} />
          <span>{t("devices:noDevices")}</span>
        </div>
      ) : (
        <div style={{ display: "flex", gap: 16 }}>
          <div style={{ flex: 1, minWidth: 0 }}>
            <div className="grid2">
              {devices.map((device) => (
                <div
                  key={device.id}
                  className={`card${selectedDeviceId === device.id ? " selected" : ""}`}
                  style={{ cursor: "pointer" }}
                  onClick={() => handleSelect(device)}
                >
                  <div className="row" style={{ marginBottom: 8 }}>
                    {device.connectionType === "wifi" ? <Wifi size={14} /> : <Usb size={14} />}
                    <span style={{ fontWeight: 600, flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                      {device.name || device.model || device.serial}
                    </span>
                    <Badge variant={statusBadgeVariant(device.status)}>
                      {statusNames[device.status] ?? device.status}
                    </Badge>
                  </div>
                  <div style={{ color: "var(--t2)", fontSize: 11, marginBottom: 6 }} className="mono">
                    {device.serial}
                  </div>
                  <div className="row" style={{ flexWrap: "wrap", gap: 3 }}>
                    {device.capabilities.slice(0, 4).map((cap) => (
                      <span key={cap} className="cap-tag">
                        {capNames[cap] ?? cap}
                      </span>
                    ))}
                    {device.capabilities.length > 4 && (
                      <span className="cap-tag off">+{device.capabilities.length - 4}</span>
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
                  <span style={{ fontWeight: 600, fontSize: 14 }}>{selectedDevice.name || selectedDevice.model || selectedDevice.serial}</span>
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
                  {statusNames[selectedDevice.status] ?? selectedDevice.status}
                </Badge>

                <div className="detail-grid" style={{ marginBottom: 12 }}>
                  <span className="detail-label">{t("devices:model")}</span>
                  <span className="detail-val">{selectedDevice.model}</span>
                  <span className="detail-label">{t("devices:brand")}</span>
                  <span className="detail-val">{selectedDevice.brand}</span>
                  <span className="detail-label">{t("devices:serial")}</span>
                  <span className="detail-val mono" style={{ fontSize: 11 }}>{selectedDevice.serial}</span>
                  <span className="detail-label">{t("devices:connection")}</span>
                  <span className="detail-val">{connNames[selectedDevice.connectionType] ?? selectedDevice.connectionType}</span>
                  {selectedDevice.androidVersion && (
                    <>
                      <span className="detail-label">{t("devices:android")}</span>
                      <span className="detail-val">{selectedDevice.androidVersion}</span>
                    </>
                  )}
                  {selectedDevice.screenSize && (
                    <>
                      <span className="detail-label">{t("devices:screen")}</span>
                      <span className="detail-val">{selectedDevice.screenSize}</span>
                    </>
                  )}
                  {selectedDevice.batteryLevel != null && (
                    <>
                      <span className="detail-label">{t("devices:battery")}</span>
                      <span className="detail-val">{selectedDevice.batteryLevel}%</span>
                    </>
                  )}
                </div>

                <div style={{ marginBottom: 12 }}>
                  <div style={{ color: "var(--t2)", fontSize: 11, fontWeight: 600, marginBottom: 6, textTransform: "uppercase", letterSpacing: "0.04em" }}>
                    {t("devices:capabilities")}
                  </div>
                  <div className="row" style={{ flexWrap: "wrap", gap: 3 }}>
                    {selectedDevice.capabilities.map((cap) => (
                      <span key={cap} className="cap-tag">
                        {capNames[cap] ?? cap}
                      </span>
                    ))}
                  </div>
                </div>

                <button
                  className="btn btn-p"
                  style={{ width: "100%", justifyContent: "center" }}
                  onClick={startMirrorFor}
                  disabled={selectedDevice.status !== "online"}
                  type="button"
                >
                  {t("devices:enterMirror")}
                </button>

                {selectedDevice.connectionType === "usb" && (
                  <button
                    className="btn btn-s"
                    style={{ width: "100%", justifyContent: "center", marginTop: 8 }}
                    onClick={() => switchUsbToWifiAndMirror(selectedDevice.serial)}
                    disabled={selectedDevice.status !== "online" || isWirelessBusy || isStartingMirror}
                    type="button"
                  >
                    {t("devices:usbToWifiMirror")}
                  </button>
                )}

                {selectedDevice.status !== "online" && (
                  <div className="detail-notice" style={{ background: "var(--wrn-s)", color: "var(--wrn)" }}>
                    {t("devices:deviceOffline")}
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
