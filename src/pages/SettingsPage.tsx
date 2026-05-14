import { Save, RotateCcw, Terminal, Monitor } from "lucide-react";
import { useEffect, useState } from "react";
import { useSettingsStore } from "../stores/settingsStore";
import { useDeviceStore } from "../stores/deviceStore";
import { Toggle } from "../components/ui/Toggle";
import { Dropdown } from "../components/ui/Dropdown";
import { Badge } from "../components/ui/Badge";
import { OPT_RES, OPT_BR, OPT_CODEC, OPT_FPS } from "../lib/presets";
import type { AppSettings, MirrorConfig } from "../types";

export function SettingsPage() {
  const storeSettings = useSettingsStore((s) => s.settings);
  const isLoading = useSettingsStore((s) => s.isLoading);
  const updateSettings = useSettingsStore((s) => s.updateSettings);
  const resetSettings = useSettingsStore((s) => s.resetSettings);
  const environment = useDeviceStore((s) => s.environment);

  const [local, setLocal] = useState<AppSettings>({ ...storeSettings });
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setLocal({ ...storeSettings });
  }, [storeSettings]);

  const updateLocal = (patch: Partial<AppSettings>) => {
    setLocal((prev) => ({ ...prev, ...patch }));
  };

  const updateDefaultConfig = (patch: Partial<MirrorConfig>) => {
    setLocal((prev) => ({
      ...prev,
      defaultMirrorConfig: { ...prev.defaultMirrorConfig, ...patch },
    }));
  };

  const handleSave = async () => {
    setSaving(true);
    await updateSettings({
      ...local,
      deviceScanIntervalSeconds: clampScanInterval(local.deviceScanIntervalSeconds),
    });
    setSaving(false);
  };

  const handleReset = async () => {
    await resetSettings();
    const fresh = useSettingsStore.getState().settings;
    setLocal({ ...fresh });
  };

  const hasChanges =
    JSON.stringify(local) !== JSON.stringify(storeSettings);

  return (
    <div>
      {environment && (
        <>
          <h2 className="sec-title flush">工具环境</h2>
          <div className="card" style={{ marginBottom: 4 }}>
            <div className="env-row" style={{ border: "none", padding: "8px 0" }}>
              <div className="env-icon" style={{ background: "var(--acc-s)", color: "var(--acc)" }}>
                <Terminal size={16} />
              </div>
              <div className="col" style={{ flex: 1 }}>
                <div className="row">
                  <span style={{ fontWeight: 600 }}>ADB</span>
                  {environment.adb.available ? (
                    <Badge variant="online">可用</Badge>
                  ) : (
                    <Badge variant="offline">不可用</Badge>
                  )}
                </div>
                {environment.adb.path && (
                  <div className="mono" style={{ color: "var(--t2)", fontSize: 11 }}>{environment.adb.path}</div>
                )}
                {environment.adb.version && (
                  <div style={{ color: "var(--t2)", fontSize: 11 }}>v{environment.adb.version}</div>
                )}
              </div>
            </div>
            <div className="env-row" style={{ border: "none", padding: "8px 0" }}>
              <div className="env-icon" style={{ background: "var(--acc-s)", color: "var(--acc)" }}>
                <Monitor size={16} />
              </div>
              <div className="col" style={{ flex: 1 }}>
                <div className="row">
                  <span style={{ fontWeight: 600 }}>Scrcpy</span>
                  {environment.scrcpy.available ? (
                    <Badge variant="online">可用</Badge>
                  ) : (
                    <Badge variant="offline">不可用</Badge>
                  )}
                </div>
                {environment.scrcpy.path && (
                  <div className="mono" style={{ color: "var(--t2)", fontSize: 11 }}>{environment.scrcpy.path}</div>
                )}
                {environment.scrcpy.version && (
                  <div style={{ color: "var(--t2)", fontSize: 11 }}>v{environment.scrcpy.version}</div>
                )}
              </div>
            </div>
          </div>
        </>
      )}

      <h2 className="sec-title">工具路径</h2>
      <div className="card">
        <div className="setting-row">
          <div style={{ flex: 1 }}>
            <div style={{ fontWeight: 500 }}>使用内置 ADB</div>
            <div style={{ color: "var(--t2)", fontSize: 11 }}>使用应用内置的 ADB 工具</div>
          </div>
          <Toggle
            on={local.useBundledAdb}
            onChange={(v) => updateLocal({ useBundledAdb: v })}
          />
        </div>
        {!local.useBundledAdb && (
          <div style={{ paddingBottom: 10 }}>
            <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600, display: "block", marginBottom: 4 }}>
              自定义 ADB 路径
            </label>
            <input
              className="inp mono"
              value={local.customAdbPath}
              onChange={(e) => updateLocal({ customAdbPath: e.target.value })}
              placeholder="C:\path\to\adb.exe"
            />
          </div>
        )}

        <div className="setting-row">
          <div style={{ flex: 1 }}>
            <div style={{ fontWeight: 500 }}>使用内置 Scrcpy</div>
            <div style={{ color: "var(--t2)", fontSize: 11 }}>使用应用内置的 Scrcpy 工具</div>
          </div>
          <Toggle
            on={local.useBundledScrcpy}
            onChange={(v) => updateLocal({ useBundledScrcpy: v })}
          />
        </div>
        {!local.useBundledScrcpy && (
          <div style={{ paddingBottom: 10 }}>
            <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600, display: "block", marginBottom: 4 }}>
              自定义 Scrcpy 路径
            </label>
            <input
              className="inp mono"
              value={local.customScrcpyPath}
              onChange={(e) => updateLocal({ customScrcpyPath: e.target.value })}
              placeholder="C:\path\to\scrcpy.exe"
            />
          </div>
        )}
      </div>

      <h2 className="sec-title">设备自动扫描</h2>
      <div className="card">
        <div className="setting-row">
          <div>
            <div style={{ fontWeight: 500 }}>启动并定时扫描设备</div>
            <div style={{ color: "var(--t2)", fontSize: 11 }}>自动刷新 USB 设备和 Android 11+ WiFi 无线调试设备</div>
          </div>
          <Toggle
            on={local.autoScanDevices}
            onChange={(v) => updateLocal({ autoScanDevices: v })}
          />
        </div>
        <div className="setting-row">
          <div>
            <div style={{ fontWeight: 500 }}>扫描间隔</div>
            <div style={{ color: "var(--t2)", fontSize: 11 }}>范围 5-600 秒，间隔越短刷新越及时</div>
          </div>
          <input
            className="inp"
            type="number"
            min={5}
            max={600}
            value={local.deviceScanIntervalSeconds}
            onChange={(e) => {
              const value = parseInt(e.target.value, 10);
              updateLocal({ deviceScanIntervalSeconds: Number.isFinite(value) ? value : 30 });
            }}
            style={{ width: 88, textAlign: "center" }}
            disabled={!local.autoScanDevices}
          />
        </div>
      </div>

      <h2 className="sec-title">默认投屏配置</h2>
      <div className="card">
        <div className="grid4" style={{ marginBottom: 10 }}>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>分辨率</label>
            <Dropdown
              value={local.defaultMirrorConfig.maxSize}
              onChange={(v) => updateDefaultConfig({ maxSize: v })}
              options={OPT_RES}
            />
          </div>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>码率</label>
            <Dropdown
              value={local.defaultMirrorConfig.videoBitRate}
              onChange={(v) => updateDefaultConfig({ videoBitRate: v })}
              options={OPT_BR}
            />
          </div>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>帧率</label>
            <Dropdown
              value={local.defaultMirrorConfig.maxFps}
              onChange={(v) => updateDefaultConfig({ maxFps: v })}
              options={OPT_FPS}
            />
          </div>
          <div className="col">
            <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>编码</label>
            <Dropdown
              value={local.defaultMirrorConfig.videoCodec}
              onChange={(v) => updateDefaultConfig({ videoCodec: v })}
              options={OPT_CODEC}
            />
          </div>
        </div>

        <div className="setting-row">
          <div>
            <div style={{ fontWeight: 500 }}>只读模式</div>
            <div style={{ color: "var(--t2)", fontSize: 11 }}>默认禁用控制</div>
          </div>
          <Toggle
            on={local.defaultMirrorConfig.noControl}
            onChange={(v) => updateDefaultConfig({ noControl: v })}
          />
        </div>
        <div className="setting-row">
          <div>
            <div style={{ fontWeight: 500 }}>保持唤醒</div>
            <div style={{ color: "var(--t2)", fontSize: 11 }}>默认保持唤醒</div>
          </div>
          <Toggle
            on={local.defaultMirrorConfig.stayAwake}
            onChange={(v) => updateDefaultConfig({ stayAwake: v })}
          />
        </div>
        <div className="setting-row">
          <div>
            <div style={{ fontWeight: 500 }}>关闭屏幕</div>
            <div style={{ color: "var(--t2)", fontSize: 11 }}>默认关闭设备屏幕</div>
          </div>
          <Toggle
            on={local.defaultMirrorConfig.turnScreenOff}
            onChange={(v) => updateDefaultConfig({ turnScreenOff: v })}
          />
        </div>
      </div>

      <h2 className="sec-title">日志</h2>
      <div className="card">
        <div className="setting-row">
          <div>
            <div style={{ fontWeight: 500 }}>日志保留天数</div>
            <div style={{ color: "var(--t2)", fontSize: 11 }}>超过保留期的日志将自动清理</div>
          </div>
          <input
            className="inp"
            type="number"
            min={1}
            max={365}
            value={local.logRetentionDays}
            onChange={(e) => updateLocal({ logRetentionDays: parseInt(e.target.value, 10) || 7 })}
            style={{ width: 80, textAlign: "center" }}
          />
        </div>
      </div>

      <div className="row" style={{ marginTop: 16, gap: 8 }}>
        <button
          className="btn btn-p"
          onClick={handleSave}
          disabled={!hasChanges || saving || isLoading}
          type="button"
        >
          <Save size={14} />
          {saving ? "保存中..." : "保存设置"}
        </button>
        <button
          className="btn btn-s"
          onClick={handleReset}
          type="button"
        >
          <RotateCcw size={14} />
          重置默认
        </button>
      </div>
    </div>
  );
}

function clampScanInterval(value: number): number {
  if (!Number.isFinite(value)) return 30;
  return Math.min(600, Math.max(5, Math.round(value)));
}
