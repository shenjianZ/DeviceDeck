import { useState } from "react";
import { AlertTriangle, Play, RefreshCw, StopCircle } from "lucide-react";
import { Badge } from "../ui/Badge";
import { Dropdown } from "../ui/Dropdown";
import { Toggle } from "../ui/Toggle";
import { useApp } from "../../context/AppContext";
import { OPT_BR, OPT_FPS, OPT_RES } from "../../data/constants";
import { PRESETS } from "../../data/mock";
import type { MirrorConfig } from "../../types";

export function MirrorPage() {
  const { state, dispatch } = useApp();
  const { devices, sessions } = state;
  const onlineDevices = devices.filter((d) => d.status === "online");

  const [selectedPreset, setSelectedPreset] = useState<string | null>(null);
  const [selectedDevice, setSelectedDevice] = useState(onlineDevices[0]?.id ?? "");
  const [starting, setStarting] = useState(false);
  const [config, setConfig] = useState<MirrorConfig>({
    maxSize: "1080",
    videoBitRate: "8M",
    maxFps: "60",
    noControl: false,
    stayAwake: true,
    turnScreenOff: false,
  });

  const deviceSerial = devices.find((d) => d.id === selectedDevice)?.serial;
  const runningForDevice = sessions.find(
    (s) => s.status === "running" && s.deviceSerial === deviceSerial
  );

  const applyPreset = (preset: (typeof PRESETS)[number]) => {
    setSelectedPreset(preset.id);
    setConfig({ ...preset.config });
  };

  const startMirror = () => {
    if (!selectedDevice || runningForDevice) return;
    setStarting(true);
    setTimeout(() => {
      dispatch({ type: "START_SESSION", deviceId: selectedDevice, config: { ...config } });
      setStarting(false);
    }, 600);
  };

  return (
    <div className="col" style={{ gap: 14 }}>
      <div className="sec-title flush">选择设备</div>
      <Dropdown
        value={selectedDevice}
        onChange={(value) => {
          setSelectedDevice(value);
          setSelectedPreset(null);
        }}
        options={[
          { value: "", label: "— 选择设备 —" },
          ...onlineDevices.map((device) => ({
            value: device.id,
            label: `${device.name} (${device.serial})`,
          })),
        ]}
        placeholder="— 选择设备 —"
        className="max-w-[320px]"
      />

      {selectedDevice && runningForDevice && (
        <div className="detail-notice" style={{ background: "var(--wrn-s)", color: "var(--wrn)" }}>
          <AlertTriangle size={14} />
          该设备已有运行中的投屏会话
        </div>
      )}

      <div className="sec-title">投屏预设</div>
      <div className="grid4">
        {PRESETS.map((preset) => (
          <div
            key={preset.id}
            className={`preset-card${selectedPreset === preset.id ? " selected" : ""}`}
            onClick={() => applyPreset(preset)}
          >
            <div style={{ fontWeight: 600, fontSize: 12, marginBottom: 2 }}>{preset.name}</div>
            <div style={{ fontSize: 11, color: "var(--t2)" }}>{preset.desc}</div>
          </div>
        ))}
      </div>

      <div className="sec-title">自定义配置</div>
      <div className="card">
        <div className="grid3" style={{ marginBottom: 10 }}>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)" }}>最大分辨率</label>
            <Dropdown
              value={config.maxSize}
              onChange={(value) => setConfig({ ...config, maxSize: value })}
              options={OPT_RES}
            />
          </div>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)" }}>视频码率</label>
            <Dropdown
              value={config.videoBitRate}
              onChange={(value) => setConfig({ ...config, videoBitRate: value })}
              options={OPT_BR}
            />
          </div>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)" }}>最大帧率</label>
            <Dropdown
              value={config.maxFps}
              onChange={(value) => setConfig({ ...config, maxFps: value })}
              options={OPT_FPS}
            />
          </div>
        </div>
        <div style={{ display: "flex", gap: 24, flexWrap: "wrap" }}>
          <div className="row">
            <Toggle on={config.noControl} onChange={(value) => setConfig({ ...config, noControl: value })} />
            <span style={{ fontSize: 12 }}>禁用控制</span>
          </div>
          <div className="row">
            <Toggle on={config.stayAwake} onChange={(value) => setConfig({ ...config, stayAwake: value })} />
            <span style={{ fontSize: 12 }}>保持唤醒</span>
          </div>
          <div className="row">
            <Toggle
              on={config.turnScreenOff}
              onChange={(value) => setConfig({ ...config, turnScreenOff: value })}
            />
            <span style={{ fontSize: 12 }}>关闭屏幕</span>
          </div>
        </div>
      </div>

      <div className="row" style={{ marginTop: 4 }}>
        <button
          className="btn btn-p"
          disabled={!selectedDevice || !!runningForDevice || starting}
          onClick={startMirror}
          type="button"
        >
          {starting ? <RefreshCw size={14} className="animate-spin" /> : <Play size={14} />}
          {starting ? "启动中..." : "启动投屏"}
        </button>
      </div>

      <div className="sec-title">活动会话</div>
      {sessions.length === 0 ? (
        <div className="empty">
          <StopCircle size={28} />
          暂无投屏会话
        </div>
      ) : (
        sessions.map((session) => {
          const device = devices.find((d) => d.serial === session.deviceSerial);
          return (
            <div key={session.id} className="session-row">
              <Badge variant={session.status === "running" ? "online" : "offline"}>
                {session.status === "running" ? "运行中" : "已停止"}
              </Badge>
              <span style={{ fontWeight: 500, fontSize: 13 }}>{device?.name ?? session.deviceSerial}</span>
              <span className="mono" style={{ fontSize: 11, color: "var(--t2)" }}>
                PID {session.processId}
              </span>
              <span style={{ fontSize: 11, color: "var(--t2)" }}>
                {session.config.maxSize}p · {session.config.videoBitRate} · {session.config.maxFps}fps
              </span>
              <span className="mono" style={{ marginLeft: "auto", fontSize: 11, color: "var(--t2)" }}>
                {Math.round((Date.now() - session.startedAt) / 60000)} 分钟前
              </span>
              {session.status === "running" && (
                <button
                  className="btn btn-d"
                  style={{ padding: "4px 10px", fontSize: 11 }}
                  onClick={() => dispatch({ type: "STOP_SESSION", sessionId: session.id })}
                  type="button"
                >
                  <StopCircle size={12} />
                  停止
                </button>
              )}
            </div>
          );
        })
      )}
    </div>
  );
}
