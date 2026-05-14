import { useState } from "react";
import { Dropdown } from "../ui/Dropdown";
import { Toggle } from "../ui/Toggle";
import { useApp } from "../../context/AppContext";
import { OPT_BR, OPT_FPS, OPT_RES } from "../../data/constants";
import type { AppSettings } from "../../types";

export function SettingsPage() {
  const { state, dispatch } = useApp();
  const [settings, setSettings] = useState<AppSettings>({ ...state.settings });
  const [saved, setSaved] = useState(false);

  const save = () => {
    dispatch({ type: "SAVE_SETTINGS", settings });
    setSaved(true);
    setTimeout(() => setSaved(false), 1500);
  };

  return (
    <div style={{ maxWidth: 560 }}>
      <div className="sec-title flush">工具路径</div>
      <div className="card" style={{ marginBottom: 16 }}>
        <div className="setting-row">
          <div>
            <div style={{ fontWeight: 500, fontSize: 13 }}>使用内置 ADB</div>
            <div style={{ fontSize: 11, color: "var(--t2)" }}>随应用附带的 adb 可执行文件</div>
          </div>
          <Toggle
            on={settings.useBundledAdb}
            onChange={(value) => setSettings({ ...settings, useBundledAdb: value })}
          />
        </div>
        {!settings.useBundledAdb && (
          <div className="setting-row">
            <div style={{ flex: 1 }}>
              <label style={{ fontSize: 12, color: "var(--t2)" }}>自定义 ADB 路径</label>
              <input
                className="inp"
                style={{ marginTop: 4 }}
                value={settings.customAdbPath}
                onChange={(event) => setSettings({ ...settings, customAdbPath: event.target.value })}
              />
            </div>
          </div>
        )}
        <div className="setting-row">
          <div>
            <div style={{ fontWeight: 500, fontSize: 13 }}>使用内置 Scrcpy</div>
            <div style={{ fontSize: 11, color: "var(--t2)" }}>随应用附带的 scrcpy 可执行文件</div>
          </div>
          <Toggle
            on={settings.useBundledScrcpy}
            onChange={(value) => setSettings({ ...settings, useBundledScrcpy: value })}
          />
        </div>
        {!settings.useBundledScrcpy && (
          <div className="setting-row">
            <div style={{ flex: 1 }}>
              <label style={{ fontSize: 12, color: "var(--t2)" }}>自定义 Scrcpy 路径</label>
              <input
                className="inp"
                style={{ marginTop: 4 }}
                value={settings.customScrcpyPath}
                onChange={(event) => setSettings({ ...settings, customScrcpyPath: event.target.value })}
              />
            </div>
          </div>
        )}
      </div>

      <div className="sec-title">默认投屏配置</div>
      <div className="card" style={{ marginBottom: 16 }}>
        <div className="grid3" style={{ gap: 12 }}>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)" }}>分辨率</label>
            <Dropdown
              value={settings.defaultMirrorConfig.maxSize}
              onChange={(value) =>
                setSettings({
                  ...settings,
                  defaultMirrorConfig: { ...settings.defaultMirrorConfig, maxSize: value },
                })
              }
              options={OPT_RES}
            />
          </div>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)" }}>码率</label>
            <Dropdown
              value={settings.defaultMirrorConfig.videoBitRate}
              onChange={(value) =>
                setSettings({
                  ...settings,
                  defaultMirrorConfig: { ...settings.defaultMirrorConfig, videoBitRate: value },
                })
              }
              options={OPT_BR}
            />
          </div>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)" }}>帧率</label>
            <Dropdown
              value={settings.defaultMirrorConfig.maxFps}
              onChange={(value) =>
                setSettings({
                  ...settings,
                  defaultMirrorConfig: { ...settings.defaultMirrorConfig, maxFps: value },
                })
              }
              options={OPT_FPS}
            />
          </div>
        </div>
      </div>

      <div className="sec-title">日志</div>
      <div className="card" style={{ marginBottom: 16 }}>
        <div className="setting-row">
          <div>
            <div style={{ fontWeight: 500, fontSize: 13 }}>日志保留天数</div>
            <div style={{ fontSize: 11, color: "var(--t2)" }}>超过保留期的日志将自动清理</div>
          </div>
          <input
            className="inp"
            style={{ width: 60 }}
            type="number"
            value={settings.logRetentionDays}
            onChange={(event) =>
              setSettings({ ...settings, logRetentionDays: parseInt(event.target.value, 10) || 7 })
            }
          />
        </div>
      </div>

      <button className="btn btn-p" onClick={save} type="button">
        {saved ? "已保存" : "保存设置"}
      </button>
    </div>
  );
}
