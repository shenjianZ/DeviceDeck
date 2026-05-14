import { useEffect, useMemo, useState } from "react";
import { Monitor, Play, RefreshCw, Square, Usb, Wifi } from "lucide-react";
import { Badge } from "../components/ui/Badge";
import { Dropdown } from "../components/ui/Dropdown";
import { Toggle } from "../components/ui/Toggle";
import { useDeviceStore } from "../stores/deviceStore";
import { useMirrorStore } from "../stores/mirrorStore";
import { OPT_BR, OPT_CODEC, OPT_FPS, OPT_RES, PRESETS, STATUS_NAMES } from "../lib/presets";
import { formatTimeAgo } from "../lib/format";
import type { WirelessAdbService } from "../types";

export function MirrorPage() {
  const devices = useDeviceStore((s) => s.devices);
  const wirelessServices = useDeviceStore((s) => s.wirelessServices);
  const scanDevices = useDeviceStore((s) => s.scanDevices);
  const discoverWirelessDevices = useDeviceStore((s) => s.discoverWirelessDevices);
  const isScanning = useDeviceStore((s) => s.isScanning);
  const isDiscoveringWireless = useDeviceStore((s) => s.isDiscoveringWireless);
  const isWirelessBusy = useDeviceStore((s) => s.isWirelessBusy);
  const pairWirelessDevice = useDeviceStore((s) => s.pairWirelessDevice);
  const wirelessMessage = useDeviceStore((s) => s.wirelessMessage);
  const deviceError = useDeviceStore((s) => s.error);

  const config = useMirrorStore((s) => s.config);
  const sessions = useMirrorStore((s) => s.sessions);
  const isStarting = useMirrorStore((s) => s.isStarting);
  const isStopping = useMirrorStore((s) => s.isStopping);
  const mirrorError = useMirrorStore((s) => s.error);
  const updateConfig = useMirrorStore((s) => s.updateConfig);
  const applyPreset = useMirrorStore((s) => s.applyPreset);
  const startMirror = useMirrorStore((s) => s.startMirror);
  const startWirelessMirror = useMirrorStore((s) => s.startWirelessMirror);
  const connectWirelessAndStartMirror = useMirrorStore((s) => s.connectWirelessAndStartMirror);
  const stopMirror = useMirrorStore((s) => s.stopMirror);

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

  const activePreset = PRESETS.find(
    (preset) =>
      preset.config.maxSize === config.maxSize &&
      preset.config.videoBitRate === config.videoBitRate &&
      preset.config.maxFps === config.maxFps &&
      preset.config.videoCodec === config.videoCodec
  );

  const selectedPairingService = useMemo(
    () => pairingServices.find((service) => service.id === selectedPairingId),
    [pairingServices, selectedPairingId]
  );
  const selectedConnectService = useMemo(
    () => connectServices.find((service) => service.id === selectedConnectId),
    [connectServices, selectedConnectId]
  );

  const runningSessions = sessions.filter((session) => session.status === "running");
  const error = mirrorError ?? deviceError;
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
      {error && (
        <div className="detail-notice" style={{ background: "var(--err-s)", color: "var(--err)", marginBottom: 12 }}>
          <div>
            <div>{error.message}</div>
            {error.suggestion && <div style={{ fontSize: 11, marginTop: 2 }}>{error.suggestion}</div>}
            {error.detail && <div className="mono" style={{ fontSize: 11, marginTop: 2 }}>{error.detail}</div>}
          </div>
        </div>
      )}

      {wirelessMessage && (
        <div className="detail-notice" style={{ background: "var(--ok-s)", color: "var(--ok)", marginBottom: 12 }}>
          {wirelessMessage}
        </div>
      )}

      <div className="row" style={{ justifyContent: "space-between", marginBottom: 8 }}>
        <h2 className="sec-title flush">投屏参数</h2>
        <button className="btn btn-s" onClick={handleRefreshAll} disabled={isScanning || isDiscoveringWireless} type="button">
          <RefreshCw size={14} className={isScanning || isDiscoveringWireless ? "spin" : ""} />
          扫描设备
        </button>
      </div>

      <div className="grid4">
        {PRESETS.map((preset) => (
          <div
            key={preset.id}
            className={`preset-card${activePreset?.id === preset.id ? " selected" : ""}`}
            onClick={() => applyPreset(preset.config)}
          >
            <div style={{ fontWeight: 600, fontSize: 13, marginBottom: 2 }}>{preset.name}</div>
            <div style={{ color: "var(--t2)", fontSize: 11 }}>{preset.description}</div>
          </div>
        ))}
      </div>

      <div className="card" style={{ marginTop: 10 }}>
        <div className="grid4" style={{ marginBottom: 12 }}>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>分辨率</label>
            <Dropdown value={config.maxSize} onChange={(value) => updateConfig({ maxSize: value })} options={OPT_RES} />
          </div>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>码率</label>
            <Dropdown value={config.videoBitRate} onChange={(value) => updateConfig({ videoBitRate: value })} options={OPT_BR} />
          </div>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>帧率</label>
            <Dropdown value={config.maxFps} onChange={(value) => updateConfig({ maxFps: value })} options={OPT_FPS} />
          </div>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>编码</label>
            <Dropdown value={config.videoCodec} onChange={(value) => updateConfig({ videoCodec: value })} options={OPT_CODEC} />
          </div>
        </div>

        <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
          <SettingToggle
            title="只读模式"
            description="禁用鼠标和键盘控制"
            value={config.noControl}
            onChange={(value) => updateConfig({ noControl: value })}
          />
          <SettingToggle
            title="保持唤醒"
            description="投屏时阻止设备自动息屏"
            value={config.stayAwake}
            onChange={(value) => updateConfig({ stayAwake: value })}
          />
          <SettingToggle
            title="关闭设备屏幕"
            description="投屏启动后关闭手机屏幕"
            value={config.turnScreenOff}
            onChange={(value) => updateConfig({ turnScreenOff: value })}
          />
        </div>
      </div>

      <h2 className="sec-title">USB 连接</h2>
      <div className="grid2">
        <div className="card">
          <div className="row" style={{ marginBottom: 10 }}>
            <Usb size={16} style={{ color: "var(--acc)" }} />
            <div>
              <div style={{ fontWeight: 600 }}>USB 投屏</div>
              <div style={{ color: "var(--t2)", fontSize: 11 }}>使用当前 USB ADB 连接直接启动 scrcpy</div>
            </div>
          </div>
          <Dropdown
            value={selectedUsbSerial}
            onChange={setSelectedUsbSerial}
            options={usbOptions}
            placeholder={usbOptions.length === 0 ? "未发现 USB 在线设备" : "选择 USB 设备"}
          />
          <button
            className="btn btn-p"
            style={{ marginTop: 10 }}
            onClick={() => startMirror(selectedUsbSerial)}
            disabled={!selectedUsbSerial || isStarting}
            type="button"
          >
            {isStarting ? <RefreshCw size={14} className="spin" /> : <Play size={14} />}
            USB 投屏
          </button>
        </div>

        <div className="card">
          <div className="row" style={{ marginBottom: 10 }}>
            <Wifi size={16} style={{ color: "var(--acc)" }} />
            <div>
              <div style={{ fontWeight: 600 }}>USB 切换为 WiFi</div>
              <div style={{ color: "var(--t2)", fontSize: 11 }}>需要先用 USB 连接，随后执行 adb tcpip 并投屏</div>
            </div>
          </div>
          <div className="grid2" style={{ marginBottom: 10 }}>
            <Dropdown
              value={selectedUsbSerial}
              onChange={setSelectedUsbSerial}
              options={usbOptions}
              placeholder={usbOptions.length === 0 ? "未发现 USB 在线设备" : "选择 USB 设备"}
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
            切换为 WiFi 并投屏
          </button>
        </div>
      </div>

      <h2 className="sec-title">WiFi 无线连接</h2>
      <div className="grid2">
        <div className="card">
          <div className="row" style={{ marginBottom: 10, justifyContent: "space-between" }}>
            <div className="row">
              <Wifi size={16} style={{ color: "var(--acc)" }} />
              <div>
                <div style={{ fontWeight: 600 }}>自动发现无线调试</div>
                <div style={{ color: "var(--t2)", fontSize: 11 }}>扫描 Android 11+ 无线调试服务，不需要手输 IP 和端口</div>
              </div>
            </div>
            <button className="btn btn-s" onClick={discoverWirelessDevices} disabled={isDiscoveringWireless} type="button">
              <RefreshCw size={14} className={isDiscoveringWireless ? "spin" : ""} />
              扫描 WiFi
            </button>
          </div>

          <div className="grid2" style={{ marginBottom: 10 }}>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>可连接设备</label>
              <Dropdown
                value={selectedConnectId}
                onChange={setSelectedConnectId}
                options={connectOptions}
                placeholder={connectOptions.length === 0 ? "未发现可连接服务" : "选择可连接设备"}
              />
            </div>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>已连接 WiFi ADB</label>
              <Dropdown
                value={selectedWifiSerial}
                onChange={setSelectedWifiSerial}
                options={wifiDeviceOptions}
                placeholder={wifiDeviceOptions.length === 0 ? "暂无已连接 WiFi 设备" : "选择已连接设备"}
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
              连接并投屏
            </button>
            <button
              className="btn btn-s"
              onClick={() => startMirror(selectedWifiSerial)}
              disabled={!selectedWifiSerial || isStarting}
              type="button"
            >
              <Monitor size={14} />
              已连接设备投屏
            </button>
          </div>
        </div>

        <div className="card">
          <div style={{ fontWeight: 600, marginBottom: 10 }}>无线调试配对</div>
          <div className="grid2" style={{ marginBottom: 10 }}>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>配对服务</label>
              <Dropdown
                value={selectedPairingId}
                onChange={setSelectedPairingId}
                options={pairingOptions}
                placeholder={pairingOptions.length === 0 ? "未发现配对服务" : "选择配对服务"}
              />
            </div>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>配对码</label>
              <input
                className="inp mono"
                value={pairCode}
                onChange={(event) => setPairCode(event.target.value)}
                placeholder="手机显示的配对码"
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
            配对设备
          </button>
          <div style={{ color: "var(--t2)", fontSize: 11, marginTop: 8 }}>
            首次连接必须在手机无线调试页面输入配对码；配对后使用左侧“连接并投屏”。
          </div>
        </div>
      </div>

      <div className="card" style={{ marginTop: 10 }}>
        <div style={{ fontWeight: 600, marginBottom: 10 }}>手动 WiFi 连接</div>
        <div className="grid2" style={{ marginBottom: 10 }}>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>设备 IP</label>
            <input className="inp mono" value={manualHost} onChange={(event) => setManualHost(event.target.value)} placeholder="192.168.1.23" />
          </div>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>连接端口</label>
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
          手动连接并投屏
        </button>
      </div>

      <h2 className="sec-title">
        活动会话
        <span style={{ fontWeight: 400, marginLeft: 8, fontSize: 12, color: "var(--t2)" }}>
          ({runningSessions.length} 运行中)
        </span>
      </h2>

      {sessions.length === 0 ? (
        <div className="empty">
          <Monitor size={32} />
          <span>暂无投屏会话</span>
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
                    {STATUS_NAMES[session.status] ?? session.status}
                  </Badge>
                </div>
                <div style={{ color: "var(--t2)", fontSize: 11, marginTop: 2 }}>
                  {session.config.maxSize} / {session.config.videoBitRate} / {session.config.maxFps}fps / {session.config.videoCodec.toUpperCase()}
                  {" · "}
                  启动于 {formatTimeAgo(session.startedAt)}
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
                  {isStopping === session.id ? "停止中..." : "停止"}
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

function SettingToggle({
  title,
  description,
  value,
  onChange,
}: {
  title: string;
  description: string;
  value: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <div className="setting-row">
      <div>
        <div style={{ fontWeight: 500 }}>{title}</div>
        <div style={{ color: "var(--t2)", fontSize: 11 }}>{description}</div>
      </div>
      <Toggle on={value} onChange={onChange} />
    </div>
  );
}

function serviceOption(service: WirelessAdbService) {
  return {
    value: service.id,
    label: `${service.name || "Android"} (${service.host}:${service.port})`,
  };
}
