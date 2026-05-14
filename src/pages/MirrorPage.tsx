import { useEffect, useState } from "react";
import { Monitor, Play, RefreshCw, Square, Wifi } from "lucide-react";
import { Badge } from "../components/ui/Badge";
import { Dropdown } from "../components/ui/Dropdown";
import { Toggle } from "../components/ui/Toggle";
import { useDeviceStore } from "../stores/deviceStore";
import { useMirrorStore } from "../stores/mirrorStore";
import { OPT_BR, OPT_FPS, OPT_RES, PRESETS, STATUS_NAMES } from "../lib/presets";
import { formatTimeAgo } from "../lib/format";

export function MirrorPage() {
  const devices = useDeviceStore((s) => s.devices);
  const scanDevices = useDeviceStore((s) => s.scanDevices);
  const isScanning = useDeviceStore((s) => s.isScanning);
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

  const onlineDevices = devices.filter((device) => device.status === "online");
  const deviceOptions = onlineDevices.map((device) => ({
    value: device.serial,
    label: `${device.name || device.model || device.serial} (${device.connectionType === "wifi" ? "WiFi" : "USB"})`,
  }));

  const [selectedSerial, setSelectedSerial] = useState("");
  const [wirelessPort, setWirelessPort] = useState(5555);
  const [manualHost, setManualHost] = useState("");
  const [manualPort, setManualPort] = useState(5555);
  const [pairHost, setPairHost] = useState("");
  const [pairPort, setPairPort] = useState(37000);
  const [pairCode, setPairCode] = useState("");

  useEffect(() => {
    if (!selectedSerial && onlineDevices.length > 0) {
      setSelectedSerial(onlineDevices[0].serial);
    }
  }, [onlineDevices, selectedSerial]);

  const activePreset = PRESETS.find(
    (preset) =>
      preset.config.maxSize === config.maxSize &&
      preset.config.videoBitRate === config.videoBitRate &&
      preset.config.maxFps === config.maxFps &&
      preset.config.noControl === config.noControl &&
      preset.config.stayAwake === config.stayAwake &&
      preset.config.turnScreenOff === config.turnScreenOff
  );

  const selectedDevice = onlineDevices.find((device) => device.serial === selectedSerial);
  const runningSessions = sessions.filter((session) => session.status === "running");
  const canStart = selectedSerial !== "" && !isStarting;
  const error = mirrorError ?? deviceError;

  const handlePair = async () => {
    const ok = await pairWirelessDevice(pairHost, pairPort, pairCode);
    if (ok) {
      setPairCode("");
    }
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

      <h2 className="sec-title flush">预设</h2>
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

      <h2 className="sec-title">参数配置</h2>
      <div className="card">
        <div className="grid3" style={{ marginBottom: 12 }}>
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
        </div>

        <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
          <div className="setting-row">
            <div>
              <div style={{ fontWeight: 500 }}>只读模式 (No Control)</div>
              <div style={{ color: "var(--t2)", fontSize: 11 }}>禁用鼠标键盘控制</div>
            </div>
            <Toggle on={config.noControl} onChange={(value) => updateConfig({ noControl: value })} />
          </div>
          <div className="setting-row">
            <div>
              <div style={{ fontWeight: 500 }}>保持唤醒 (Stay Awake)</div>
              <div style={{ color: "var(--t2)", fontSize: 11 }}>设备不会自动息屏</div>
            </div>
            <Toggle on={config.stayAwake} onChange={(value) => updateConfig({ stayAwake: value })} />
          </div>
          <div className="setting-row">
            <div>
              <div style={{ fontWeight: 500 }}>关闭屏幕 (Turn Screen Off)</div>
              <div style={{ color: "var(--t2)", fontSize: 11 }}>投屏时关闭设备屏幕</div>
            </div>
            <Toggle on={config.turnScreenOff} onChange={(value) => updateConfig({ turnScreenOff: value })} />
          </div>
        </div>
      </div>

      <h2 className="sec-title">启动投屏</h2>
      <div className="card">
        <div className="row" style={{ gap: 10 }}>
          <div style={{ flex: 1, minWidth: 0 }}>
            <Dropdown
              value={selectedSerial}
              onChange={setSelectedSerial}
              options={deviceOptions}
              placeholder={onlineDevices.length === 0 ? "无在线设备" : "选择设备"}
            />
          </div>
          <button className="btn btn-p" onClick={() => startMirror(selectedSerial)} disabled={!canStart} type="button">
            {isStarting ? <RefreshCw size={14} className="spin" /> : <Play size={14} />}
            {isStarting ? "启动中..." : "USB/当前连接投屏"}
          </button>
          <button className="btn btn-s" onClick={scanDevices} disabled={isScanning} type="button">
            <RefreshCw size={14} className={isScanning ? "spin" : ""} />
          </button>
        </div>
        {onlineDevices.length === 0 && (
          <div style={{ color: "var(--wrn)", fontSize: 12, marginTop: 6 }}>
            没有检测到在线设备，请先连接设备并扫描
          </div>
        )}
      </div>

      <h2 className="sec-title">无线局域网投屏</h2>
      <div className="grid2">
        <div className="card">
          <div className="row" style={{ marginBottom: 10 }}>
            <Wifi size={16} style={{ color: "var(--acc)" }} />
            <div>
              <div style={{ fontWeight: 600 }}>一键无线投屏</div>
              <div style={{ color: "var(--t2)", fontSize: 11 }}>USB 设备会自动切换到 WiFi ADB 后启动 scrcpy</div>
            </div>
          </div>
          <div className="grid2" style={{ marginBottom: 10 }}>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>目标设备</label>
              <Dropdown
                value={selectedSerial}
                onChange={setSelectedSerial}
                options={deviceOptions}
                placeholder="选择 USB 或 WiFi 设备"
              />
            </div>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>ADB 端口</label>
              <input
                className="inp"
                type="number"
                min={1}
                max={65535}
                value={wirelessPort}
                onChange={(event) => setWirelessPort(parseInt(event.target.value, 10) || 5555)}
              />
            </div>
          </div>
          <button
            className="btn btn-p"
            onClick={() => startWirelessMirror(selectedSerial, wirelessPort)}
            disabled={!selectedDevice || isStarting || isWirelessBusy}
            type="button"
          >
            {isStarting ? <RefreshCw size={14} className="spin" /> : <Wifi size={14} />}
            {selectedDevice?.connectionType === "wifi" ? "无线投屏" : "切换无线并投屏"}
          </button>
        </div>

        <div className="card">
          <div style={{ fontWeight: 600, marginBottom: 10 }}>手动连接并投屏</div>
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
            onClick={() => connectWirelessAndStartMirror(manualHost, manualPort)}
            disabled={!manualHost.trim() || isStarting}
            type="button"
          >
            <Play size={14} />
            连接并投屏
          </button>
        </div>
      </div>

      <div className="card" style={{ marginTop: 10 }}>
        <div style={{ fontWeight: 600, marginBottom: 10 }}>Android 11+ 无线调试配对</div>
        <div className="grid3" style={{ marginBottom: 10 }}>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>配对 IP</label>
            <input className="inp mono" value={pairHost} onChange={(event) => setPairHost(event.target.value)} placeholder="192.168.1.23" />
          </div>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>配对端口</label>
            <input
              className="inp"
              type="number"
              min={1}
              max={65535}
              value={pairPort}
              onChange={(event) => setPairPort(parseInt(event.target.value, 10) || 37000)}
            />
          </div>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>配对码</label>
            <input className="inp mono" value={pairCode} onChange={(event) => setPairCode(event.target.value)} placeholder="6 位配对码" />
          </div>
        </div>
        <button
          className="btn btn-s"
          onClick={handlePair}
          disabled={!pairHost.trim() || !pairCode.trim() || isWirelessBusy}
          type="button"
        >
          {isWirelessBusy ? <RefreshCw size={14} className="spin" /> : <Wifi size={14} />}
          配对无线调试
        </button>
        <div style={{ color: "var(--t2)", fontSize: 11, marginTop: 8 }}>
          配对成功后，在上方“手动连接并投屏”输入无线调试页面显示的连接端口。
        </div>
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
                  {session.config.maxSize} / {session.config.videoBitRate} / {session.config.maxFps}fps
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
