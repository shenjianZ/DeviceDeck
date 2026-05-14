import { useEffect, useMemo, useState } from "react";
import { Monitor, Play, RefreshCw, Square, Usb, Wifi } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Badge } from "../components/ui/Badge";
import { Dropdown } from "../components/ui/Dropdown";
import { useDeviceStore } from "../stores/deviceStore";
import { useMirrorStore } from "../stores/mirrorStore";
import { getStatusNames } from "../lib/presets";
import { formatTimeAgo } from "../lib/format";
import type { WirelessAdbService } from "../types";

export function MirrorPage() {
  const { t } = useTranslation(["mirror", "common"]);

  const devices = useDeviceStore((s) => s.devices);
  const wirelessServices = useDeviceStore((s) => s.wirelessServices);
  const scanDevices = useDeviceStore((s) => s.scanDevices);
  const discoverWirelessDevices = useDeviceStore((s) => s.discoverWirelessDevices);
  const isScanning = useDeviceStore((s) => s.isScanning);
  const isDiscoveringWireless = useDeviceStore((s) => s.isDiscoveringWireless);
  const isWirelessBusy = useDeviceStore((s) => s.isWirelessBusy);
  const pairWirelessDevice = useDeviceStore((s) => s.pairWirelessDevice);

  const sessions = useMirrorStore((s) => s.sessions);
  const isStarting = useMirrorStore((s) => s.isStarting);
  const isStopping = useMirrorStore((s) => s.isStopping);
  const startMirror = useMirrorStore((s) => s.startMirror);
  const startWirelessMirror = useMirrorStore((s) => s.startWirelessMirror);
  const connectWirelessAndStartMirror = useMirrorStore((s) => s.connectWirelessAndStartMirror);
  const stopMirror = useMirrorStore((s) => s.stopMirror);

  const statusNames = getStatusNames(t);

  const usbDevices = devices.filter((device) => device.status === "online" && device.connectionType === "usb");
  const wifiDevices = devices.filter((device) => device.status === "online" && device.connectionType === "wifi");
  const pairingServices = wirelessServices.filter((service) => service.serviceType === "pairing");
  const connectServices = wirelessServices.filter((service) => service.serviceType === "connect");

  const usbOptions = usbDevices.map((device) => ({
    value: device.serial,
    label: `${device.name || device.model || device.serial} (${device.serial})`,
  }));
  const wifiDeviceOptions = wifiDevices.map((device) => ({
    value: device.serial,
    label: `${device.name || device.model || device.serial} (${device.serial})`,
  }));
  const pairingOptions = pairingServices.map(serviceOption);
  const connectOptions = connectServices.map(serviceOption);

  const [selectedUsbSerial, setSelectedUsbSerial] = useState("");
  const [selectedWifiSerial, setSelectedWifiSerial] = useState("");
  const [wirelessPort, setWirelessPort] = useState(5555);
  const [selectedPairingId, setSelectedPairingId] = useState("");
  const [selectedConnectId, setSelectedConnectId] = useState("");
  const [pairCode, setPairCode] = useState("");
  const [manualHost, setManualHost] = useState("");
  const [manualPort, setManualPort] = useState(5555);

  useEffect(() => {
    if (!selectedUsbSerial && usbDevices.length > 0) {
      setSelectedUsbSerial(usbDevices[0].serial);
    }
  }, [selectedUsbSerial, usbDevices]);

  useEffect(() => {
    if (!selectedWifiSerial && wifiDevices.length > 0) {
      setSelectedWifiSerial(wifiDevices[0].serial);
    }
  }, [selectedWifiSerial, wifiDevices]);

  useEffect(() => {
    if (!selectedPairingId && pairingServices.length > 0) {
      setSelectedPairingId(pairingServices[0].id);
    }
  }, [pairingServices, selectedPairingId]);

  useEffect(() => {
    if (!selectedConnectId && connectServices.length > 0) {
      setSelectedConnectId(connectServices[0].id);
    }
  }, [connectServices, selectedConnectId]);

  const selectedPairingService = useMemo(
    () => pairingServices.find((service) => service.id === selectedPairingId),
    [pairingServices, selectedPairingId]
  );
  const selectedConnectService = useMemo(
    () => connectServices.find((service) => service.id === selectedConnectId),
    [connectServices, selectedConnectId]
  );

  const runningSessions = sessions.filter((session) => session.status === "running");
  const isBusy = isStarting || isWirelessBusy || isDiscoveringWireless;

  const handleRefreshAll = async () => {
    await Promise.all([scanDevices(), discoverWirelessDevices()]);
  };

  const handlePairSelectedService = async () => {
    if (!selectedPairingService) return;
    const ok = await pairWirelessDevice(
      selectedPairingService.host,
      selectedPairingService.port,
      pairCode
    );
    if (ok) {
      setPairCode("");
      await discoverWirelessDevices();
    }
  };

  const handleConnectSelectedService = async () => {
    if (!selectedConnectService) return;
    await connectWirelessAndStartMirror(selectedConnectService.host, selectedConnectService.port);
    await scanDevices();
  };

  const handleManualConnect = async () => {
    await connectWirelessAndStartMirror(manualHost, manualPort);
    await scanDevices();
  };

  return (
    <div>
      <div className="row" style={{ justifyContent: "space-between", marginBottom: 8 }}>
        <h2 className="sec-title flush">{t("mirror:usbConnection")}</h2>
        <button className="btn btn-s" onClick={handleRefreshAll} disabled={isScanning || isDiscoveringWireless} type="button">
          <RefreshCw size={14} className={isScanning || isDiscoveringWireless ? "spin" : ""} />
          {t("mirror:scanDevices")}
        </button>
      </div>

      <div className="grid2">
        <div className="card">
          <div className="row" style={{ marginBottom: 10 }}>
            <Usb size={16} style={{ color: "var(--acc)" }} />
            <div>
              <div style={{ fontWeight: 600 }}>{t("mirror:usbMirror")}</div>
              <div style={{ color: "var(--t2)", fontSize: 11 }}>{t("mirror:usbMirrorDesc")}</div>
            </div>
          </div>
          <Dropdown
            value={selectedUsbSerial}
            onChange={setSelectedUsbSerial}
            options={usbOptions}
            placeholder={usbOptions.length === 0 ? t("mirror:noUsbDevice") : t("mirror:selectUsbDevice")}
          />
          <button
            className="btn btn-p"
            style={{ marginTop: 10 }}
            onClick={() => startMirror(selectedUsbSerial)}
            disabled={!selectedUsbSerial || isStarting}
            type="button"
          >
            {isStarting ? <RefreshCw size={14} className="spin" /> : <Play size={14} />}
            {t("mirror:usbMirror")}
          </button>
        </div>

        <div className="card">
          <div className="row" style={{ marginBottom: 10 }}>
            <Wifi size={16} style={{ color: "var(--acc)" }} />
            <div>
              <div style={{ fontWeight: 600 }}>{t("mirror:usbToWifi")}</div>
              <div style={{ color: "var(--t2)", fontSize: 11 }}>{t("mirror:usbToWifiDesc")}</div>
            </div>
          </div>
          <div className="grid2" style={{ marginBottom: 10 }}>
            <Dropdown
              value={selectedUsbSerial}
              onChange={setSelectedUsbSerial}
              options={usbOptions}
              placeholder={usbOptions.length === 0 ? t("mirror:noUsbDevice") : t("mirror:selectUsbDevice")}
            />
            <input
              className="inp"
              type="number"
              min={1}
              max={65535}
              value={wirelessPort}
              onChange={(event) => setWirelessPort(parseInt(event.target.value, 10) || 5555)}
            />
          </div>
          <button
            className="btn btn-p"
            onClick={() => startWirelessMirror(selectedUsbSerial, wirelessPort)}
            disabled={!selectedUsbSerial || isBusy}
            type="button"
          >
            {isStarting ? <RefreshCw size={14} className="spin" /> : <Wifi size={14} />}
            {t("mirror:switchWifiMirror")}
          </button>
        </div>
      </div>

      <h2 className="sec-title">{t("mirror:wifiConnection")}</h2>
      <div className="grid2">
        <div className="card">
          <div className="row" style={{ marginBottom: 10, justifyContent: "space-between" }}>
            <div className="row">
              <Wifi size={16} style={{ color: "var(--acc)" }} />
              <div>
                <div style={{ fontWeight: 600 }}>{t("mirror:autoDiscover")}</div>
                <div style={{ color: "var(--t2)", fontSize: 11 }}>{t("mirror:autoDiscoverDesc")}</div>
              </div>
            </div>
            <button className="btn btn-s" onClick={() => discoverWirelessDevices()} disabled={isDiscoveringWireless} type="button">
              <RefreshCw size={14} className={isDiscoveringWireless ? "spin" : ""} />
              {t("mirror:scanWifi")}
            </button>
          </div>

          <div className="grid2" style={{ marginBottom: 10 }}>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:connectableDevices")}</label>
              <Dropdown
                value={selectedConnectId}
                onChange={setSelectedConnectId}
                options={connectOptions}
                placeholder={connectOptions.length === 0 ? t("mirror:noConnectableService") : t("mirror:selectConnectable")}
              />
            </div>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:connectedWifiAdb")}</label>
              <Dropdown
                value={selectedWifiSerial}
                onChange={setSelectedWifiSerial}
                options={wifiDeviceOptions}
                placeholder={wifiDeviceOptions.length === 0 ? t("mirror:noConnectedWifi") : t("mirror:selectConnected")}
              />
            </div>
          </div>

          <div className="row" style={{ gap: 8, flexWrap: "wrap" }}>
            <button
              className="btn btn-p"
              onClick={handleConnectSelectedService}
              disabled={!selectedConnectService || isBusy}
              type="button"
            >
              {isStarting ? <RefreshCw size={14} className="spin" /> : <Play size={14} />}
              {t("mirror:connectAndMirror")}
            </button>
            <button
              className="btn btn-s"
              onClick={() => startMirror(selectedWifiSerial)}
              disabled={!selectedWifiSerial || isStarting}
              type="button"
            >
              <Monitor size={14} />
              {t("mirror:connectedDeviceMirror")}
            </button>
          </div>
        </div>

        <div className="card">
          <div style={{ fontWeight: 600, marginBottom: 10 }}>{t("mirror:pairing")}</div>
          <div className="grid2" style={{ marginBottom: 10 }}>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:pairingService")}</label>
              <Dropdown
                value={selectedPairingId}
                onChange={setSelectedPairingId}
                options={pairingOptions}
                placeholder={pairingOptions.length === 0 ? t("mirror:noPairingService") : t("mirror:selectPairingService")}
              />
            </div>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:pairCode")}</label>
              <input
                className="inp mono"
                value={pairCode}
                onChange={(event) => setPairCode(event.target.value)}
                placeholder={t("mirror:pairCodePlaceholder")}
              />
            </div>
          </div>
          <button
            className="btn btn-s"
            onClick={handlePairSelectedService}
            disabled={!selectedPairingService || !pairCode.trim() || isWirelessBusy}
            type="button"
          >
            {isWirelessBusy ? <RefreshCw size={14} className="spin" /> : <Wifi size={14} />}
            {t("mirror:pairDevice")}
          </button>
          <div style={{ color: "var(--t2)", fontSize: 11, marginTop: 8 }}>
            {t("mirror:pairingHint")}
          </div>
        </div>
      </div>

      <div className="card" style={{ marginTop: 10 }}>
        <div style={{ fontWeight: 600, marginBottom: 10 }}>{t("mirror:manualConnect")}</div>
        <div className="grid2" style={{ marginBottom: 10 }}>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:deviceIp")}</label>
            <input className="inp mono" value={manualHost} onChange={(event) => setManualHost(event.target.value)} placeholder="192.168.1.23" />
          </div>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:connectPort")}</label>
            <input
              className="inp"
              type="number"
              min={1}
              max={65535}
              value={manualPort}
              onChange={(event) => setManualPort(parseInt(event.target.value, 10) || 5555)}
            />
          </div>
        </div>
        <button
          className="btn btn-p"
          onClick={handleManualConnect}
          disabled={!manualHost.trim() || isStarting}
          type="button"
        >
          <Play size={14} />
          {t("mirror:manualConnectBtn")}
        </button>
      </div>

      <h2 className="sec-title">
        {t("mirror:activeSessions")}
        <span style={{ fontWeight: 400, marginLeft: 8, fontSize: 12, color: "var(--t2)" }}>
          ({runningSessions.length} {t("mirror:running")})
        </span>
      </h2>

      {sessions.length === 0 ? (
        <div className="empty">
          <Monitor size={32} />
          <span>{t("mirror:noSessions")}</span>
        </div>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          {sessions.map((session) => (
            <div key={session.id} className="session-row">
              <Monitor size={16} style={{ color: session.status === "running" ? "var(--ok)" : "var(--t2)", flexShrink: 0 }} />
              <div style={{ flex: 1, minWidth: 0 }}>
                <div className="row" style={{ gap: 6 }}>
                  <span style={{ fontWeight: 600, fontSize: 12 }} className="mono">{session.deviceSerial}</span>
                  <Badge variant={session.status === "running" ? "online" : session.status === "failed" ? "offline" : "unknown"}>
                    {statusNames[session.status] ?? session.status}
                  </Badge>
                </div>
                <div style={{ color: "var(--t2)", fontSize: 11, marginTop: 2 }}>
                  {session.config.maxSize} / {session.config.videoBitRate} / {session.config.maxFps}fps / {session.config.videoCodec.toUpperCase()}
                  {" · "}
                  {t("mirror:startedAt")} {formatTimeAgo(session.startedAt)}
                </div>
              </div>
              {session.status === "running" && (
                <button
                  className="btn btn-d"
                  onClick={() => stopMirror(session.id)}
                  disabled={isStopping === session.id}
                  type="button"
                >
                  <Square size={12} />
                  {isStopping === session.id ? t("mirror:stopping") : t("mirror:stop")}
                </button>
              )}
            </div>
          ))}
        </div>
      )}

      <style>{`
        @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
        .spin { animation: spin 1s linear infinite; }
      `}</style>
    </div>
  );
}

function serviceOption(service: WirelessAdbService) {
  return {
    value: service.id,
    label: `${service.name || "Android"} (${service.host}:${service.port})`,
  };
}
