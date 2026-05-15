import { useState } from "react";
import {
  Camera,
  ChevronRight,
  FileUp,
  Home,
  Package,
  Power,
  RefreshCw,
  ScanSearch,
  Smartphone,
  Terminal,
  Usb,
  Volume2,
  Wifi,
  X as XIcon,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { useDeviceStore } from "../stores/deviceStore";
import { useMirrorStore } from "../stores/mirrorStore";
import { usePageStore } from "../stores/pageStore";
import { Badge } from "../components/ui/Badge";
import { getStatusNames, getConnNames, getCapNames } from "../lib/presets";
import type { DeviceInfo, DeviceKeyAction } from "../types";

const KEY_ACTIONS: { action: DeviceKeyAction; labelKey: string; icon: typeof Home }[] = [
  { action: "home", labelKey: "devices:keyActions.home", icon: Home },
  { action: "back", labelKey: "devices:keyActions.back", icon: ChevronRight },
  { action: "appSwitch", labelKey: "devices:keyActions.appSwitch", icon: Smartphone },
  { action: "power", labelKey: "devices:keyActions.power", icon: Power },
  { action: "volumeUp", labelKey: "devices:keyActions.volumeUp", icon: Volume2 },
  { action: "volumeDown", labelKey: "devices:keyActions.volumeDown", icon: Volume2 },
  { action: "expandNotifications", labelKey: "devices:keyActions.expandNotifications", icon: Terminal },
  { action: "collapseNotifications", labelKey: "devices:keyActions.collapseNotifications", icon: Terminal },
  { action: "turnScreenOff", labelKey: "devices:keyActions.turnScreenOff", icon: Power },
];

const DEFAULT_PUSH_DIRECTORY = "/sdcard/Download/DeviceDeck";

export function DevicesPage() {
  const { t } = useTranslation(["devices", "common"]);
  const devices = useDeviceStore((s) => s.devices);
  const selectedDeviceId = useDeviceStore((s) => s.selectedDeviceId);
  const isScanning = useDeviceStore((s) => s.isScanning);
  const scanDevices = useDeviceStore((s) => s.scanDevices);
  const selectDevice = useDeviceStore((s) => s.selectDevice);
  const isWirelessBusy = useDeviceStore((s) => s.isWirelessBusy);
  const isDetectingCapabilities = useDeviceStore((s) => s.isDetectingCapabilities);
  const capabilityReports = useDeviceStore((s) => s.capabilityReports);
  const detectCapabilities = useDeviceStore((s) => s.detectCapabilities);
  const applyRecommendedConfig = useDeviceStore((s) => s.applyRecommendedConfig);
  const isDeviceActionBusy = useDeviceStore((s) => s.isDeviceActionBusy);
  const takeScreenshot = useDeviceStore((s) => s.takeScreenshot);
  const installApk = useDeviceStore((s) => s.installApk);
  const pushFile = useDeviceStore((s) => s.pushFile);
  const runKeyAction = useDeviceStore((s) => s.runKeyAction);
  const runShellCommand = useDeviceStore((s) => s.runShellCommand);
  const startWirelessMirror = useMirrorStore((s) => s.startWirelessMirror);
  const isStartingMirror = useMirrorStore((s) => s.isStarting);
  const setPage = usePageStore((s) => s.setPage);
  const [remoteDirectory, setRemoteDirectory] = useState(DEFAULT_PUSH_DIRECTORY);
  const [shellCommand, setShellCommand] = useState("");

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

  const handleScreenshot = async (serial: string) => {
    const outputDirectory = await open({ directory: true, multiple: false });
    await takeScreenshot(serial, typeof outputDirectory === "string" ? outputDirectory : undefined);
  };

  const handleInstallApk = async (serial: string) => {
    const apkPath = await open({
      multiple: false,
      filters: [{ name: "Android APK", extensions: ["apk"] }],
    });
    if (typeof apkPath === "string") {
      await installApk(serial, apkPath);
    }
  };

  const handlePushFile = async (serial: string) => {
    const localPath = await open({ multiple: false });
    if (typeof localPath === "string") {
      await pushFile(serial, localPath, remoteDirectory);
    }
  };

  const handleRunShellCommand = async (serial: string) => {
    const command = shellCommand.trim();
    if (!command) return;
    await runShellCommand(serial, command);
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
                  className="btn btn-s"
                  style={{ width: "100%", justifyContent: "center", marginBottom: 8 }}
                  onClick={() => detectCapabilities(selectedDevice.serial)}
                  disabled={selectedDevice.status !== "online" || isDetectingCapabilities}
                  type="button"
                >
                  {isDetectingCapabilities ? (
                    <RefreshCw size={14} className="spin" />
                  ) : (
                    <ScanSearch size={14} />
                  )}
                  {isDetectingCapabilities ? t("devices:capability.detecting") : t("devices:capability.detect")}
                </button>

                {capabilityReports[selectedDevice.serial] && (
                  <div style={{ marginBottom: 12 }}>
                    <div style={{ color: "var(--t2)", fontSize: 11, fontWeight: 600, marginBottom: 6, textTransform: "uppercase", letterSpacing: "0.04em" }}>
                      {t("devices:capability.recommendedConfigs")}
                    </div>
                    {capabilityReports[selectedDevice.serial].map((rec, idx) => (
                      <div
                        key={idx}
                        className="card"
                        style={{ padding: 8, marginBottom: 4, cursor: "pointer", borderLeft: "2px solid var(--acc)" }}
                        onClick={() => applyRecommendedConfig(selectedDevice.serial, rec.config)}
                      >
                        <div style={{ fontWeight: 600, fontSize: 12 }}>
                          {t(`devices:${rec.label}`)}
                        </div>
                        <div style={{ color: "var(--t2)", fontSize: 11 }}>
                          {t(`devices:${rec.description}`)}
                        </div>
                        <div className="mono" style={{ color: "var(--t3)", fontSize: 10, marginTop: 2 }}>
                          {rec.config.maxSize === "native" ? "Native" : `${rec.config.maxSize}p`} / {rec.config.videoBitRate} / {rec.config.maxFps}fps / {rec.config.videoCodec.toUpperCase()}
                        </div>
                      </div>
                    ))}
                  </div>
                )}

                <div style={{ marginBottom: 12 }}>
                  <div style={{ color: "var(--t2)", fontSize: 11, fontWeight: 600, marginBottom: 6, textTransform: "uppercase", letterSpacing: "0.04em" }}>
                    {t("devices:deviceTools")}
                  </div>
                  <div className="row" style={{ gap: 6, flexWrap: "wrap", marginBottom: 8 }}>
                    <button
                      className="btn btn-s"
                      type="button"
                      disabled={selectedDevice.status !== "online" || isDeviceActionBusy}
                      onClick={() => handleScreenshot(selectedDevice.serial)}
                    >
                      <Camera size={14} />
                      {t("devices:screenshot")}
                    </button>
                    <button
                      className="btn btn-s"
                      type="button"
                      disabled={selectedDevice.status !== "online" || isDeviceActionBusy}
                      onClick={() => handleInstallApk(selectedDevice.serial)}
                    >
                      <Package size={14} />
                      {t("devices:installApk")}
                    </button>
                    <button
                      className="btn btn-s"
                      type="button"
                      disabled={selectedDevice.status !== "online" || isDeviceActionBusy}
                      onClick={() => handlePushFile(selectedDevice.serial)}
                    >
                      <FileUp size={14} />
                      {t("devices:pushFile")}
                    </button>
                  </div>
                  <input
                    className="inp mono"
                    value={remoteDirectory}
                    onChange={(event) => setRemoteDirectory(event.target.value)}
                    placeholder={DEFAULT_PUSH_DIRECTORY}
                    style={{ marginBottom: 8, width: "100%" }}
                  />
                  <div className="row" style={{ gap: 4, flexWrap: "wrap", marginBottom: 8 }}>
                    {KEY_ACTIONS.map((item) => {
                      const Icon = item.icon;
                      return (
                        <button
                          key={item.action}
                          className="btn btn-g"
                          type="button"
                          disabled={selectedDevice.status !== "online" || isDeviceActionBusy}
                          onClick={() => runKeyAction(selectedDevice.serial, item.action)}
                          title={t(item.labelKey)}
                        >
                          <Icon size={13} />
                          {t(item.labelKey)}
                        </button>
                      );
                    })}
                  </div>
                  <div className="row" style={{ gap: 6 }}>
                    <input
                      className="inp mono"
                      value={shellCommand}
                      onChange={(event) => setShellCommand(event.target.value)}
                      onKeyDown={(event) => {
                        if (event.key === "Enter") {
                          handleRunShellCommand(selectedDevice.serial);
                        }
                      }}
                      placeholder="getprop ro.build.version.release"
                      style={{ flex: 1, minWidth: 0 }}
                    />
                    <button
                      className="btn btn-s"
                      type="button"
                      disabled={selectedDevice.status !== "online" || isDeviceActionBusy || !shellCommand.trim()}
                      onClick={() => handleRunShellCommand(selectedDevice.serial)}
                    >
                      <Terminal size={14} />
                      {t("devices:execute")}
                    </button>
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
