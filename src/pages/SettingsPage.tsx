import { useState } from "react";
import {
  Code2,
  Download,
  Info,
  MessageSquare,
  Monitor,
  Moon,
  Palette,
  RefreshCw,
  Sun,
  Terminal,
  FileText,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { useSettingsStore } from "../stores/settingsStore";
import { useDeviceStore } from "../stores/deviceStore";
import { useUpdaterStore } from "../stores/updaterStore";
import { useMirrorStore } from "../stores/mirrorStore";
import { Toggle } from "../components/ui/Toggle";
import { Dropdown } from "../components/ui/Dropdown";
import { Badge } from "../components/ui/Badge";
import { OPT_BR, OPT_FPS, getOptCodec, getOptRes, getPresets } from "../lib/presets";
import type { MirrorConfig } from "../types";
import { open } from "@tauri-apps/plugin-dialog";

const FONT_SIZE_OPTIONS = [
  { value: "12", label: "12 px" },
  { value: "13", label: "13 px" },
  { value: "14", label: "14 px" },
  { value: "15", label: "15 px" },
  { value: "16", label: "16 px" },
];

function getRecordModeOptions(t: (key: string) => string) {
  return [
    { value: "off", label: t("mirror:recordModes.off") },
    { value: "window", label: t("mirror:recordModes.window") },
    { value: "background", label: t("mirror:recordModes.background") },
  ];
}

const RECORD_FORMAT_OPTIONS = [
  { value: "mp4", label: "MP4" },
  { value: "mkv", label: "MKV" },
];

function getOrientationOptions(t: (key: string) => string) {
  return [
    { value: "unlocked", label: t("mirror:orientations.unlocked") },
    { value: "0", label: "0°" },
    { value: "90", label: "90°" },
    { value: "180", label: "180°" },
    { value: "270", label: "270°" },
  ];
}

function getAudioSourceOptions(t: (key: string) => string) {
  return [
    { value: "output", label: t("mirror:audioSources.output") },
    { value: "playback", label: t("mirror:audioSources.playback") },
    { value: "mic", label: t("mirror:audioSources.mic") },
    { value: "mic-camcorder", label: t("mirror:audioSources.mic-camcorder") },
    { value: "voice-recognition", label: t("mirror:audioSources.voice-recognition") },
    { value: "voice-communication", label: t("mirror:audioSources.voice-communication") },
    { value: "voice-performance", label: t("mirror:audioSources.voice-performance") },
  ];
}

const AUDIO_CODEC_OPTIONS = [
  { value: "opus", label: "Opus" },
  { value: "aac", label: "AAC" },
  { value: "flac", label: "FLAC" },
  { value: "raw", label: "Raw" },
];

export function SettingsPage() {
  const { t, i18n } = useTranslation(["settings", "common", "mirror"]);
  const settings = useSettingsStore((s) => s.settings);
  const updateSetting = useSettingsStore((s) => s.updateSetting);
  const environment = useDeviceStore((s) => s.environment);
  const updateMirrorConfig = useMirrorStore((s) => s.updateConfig);
  const applyMirrorPreset = useMirrorStore((s) => s.applyPreset);

  const updateState = useUpdaterStore((s) => s.updateState);
  const checkForUpdates = useUpdaterStore((s) => s.checkForUpdates);
  const downloadUpdate = useUpdaterStore((s) => s.downloadUpdate);
  const installUpdate = useUpdaterStore((s) => s.installUpdate);

  const [activeSection, setActiveSection] = useState(0);

  const recordModeOptions = getRecordModeOptions(t);
  const orientationOptions = getOrientationOptions(t);
  const audioSourceOptions = getAudioSourceOptions(t);

  const settingsMenu = [
    { label: t("settings:menu.appearance"), icon: Palette },
    { label: t("settings:menu.tools"), icon: Terminal },
    { label: t("settings:menu.mirror"), icon: Monitor },
    { label: t("settings:menu.logs"), icon: FileText },
    { label: t("settings:menu.about"), icon: Info },
  ];

  function renderSection() {
    if (activeSection === 0) {
      return (
        <section className="settings-section">
          <div className="settings-row">
            <span className="settings-label">{t("settings:appearance.theme")}</span>
            <div className="settings-theme-btns">
              <button
                className={`settings-theme-btn${settings.theme === "light" ? " active" : ""}`}
                onClick={() => updateSetting("theme", "light")}
                type="button"
              >
                <Sun size={14} />
                {t("settings:appearance.light")}
              </button>
              <button
                className={`settings-theme-btn${settings.theme === "dark" ? " active" : ""}`}
                onClick={() => updateSetting("theme", "dark")}
                type="button"
              >
                <Moon size={14} />
                {t("settings:appearance.dark")}
              </button>
            </div>
          </div>

          <div className="settings-row">
            <span className="settings-label">{t("settings:appearance.language")}</span>
            <Dropdown
              className="settings-compact-select"
              value={settings.locale || "zh-CN"}
              onChange={(v) => {
                updateSetting("locale", v as "zh-CN" | "en");
                i18n.changeLanguage(v);
              }}
              options={[
                { value: "zh-CN", label: t("settings:localeOptions.zh-CN") },
                { value: "en", label: t("settings:localeOptions.en") },
              ]}
            />
          </div>

          <div className="settings-row">
            <span className="settings-label">{t("settings:appearance.fontSize")}</span>
            <Dropdown
              className="settings-compact-select"
              value={String(settings.fontSize || 14)}
              onChange={(v) => updateSetting("fontSize", Number(v))}
              options={FONT_SIZE_OPTIONS}
            />
          </div>

          <div className="settings-row">
            <div>
              <span className="settings-label">{t("settings:appearance.autostart")}</span>
              <div className="settings-desc">{t("settings:appearance.autostartDesc")}</div>
            </div>
            <Toggle
              on={settings.autoStart || false}
              onChange={(v) => updateSetting("autoStart", v)}
            />
          </div>
        </section>
      );
    }

    if (activeSection === 1) {
      return (
        <>
          {environment && (
            <section className="settings-section">
              <div className="card" style={{ marginBottom: 4 }}>
                <div className="env-row" style={{ border: "none", padding: "8px 0" }}>
                  <div className="env-icon" style={{ background: "var(--acc-s)", color: "var(--acc)" }}>
                    <Terminal size={16} />
                  </div>
                  <div className="col" style={{ flex: 1 }}>
                    <div className="row">
                      <span style={{ fontWeight: 600 }}>{t("settings:tools.adb")}</span>
                      {environment.adb.available ? (
                        <Badge variant="online">{t("common:status.available")}</Badge>
                      ) : (
                        <Badge variant="offline">{t("common:status.unavailable")}</Badge>
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
                      <span style={{ fontWeight: 600 }}>{t("settings:tools.scrcpy")}</span>
                      {environment.scrcpy.available ? (
                        <Badge variant="online">{t("common:status.available")}</Badge>
                      ) : (
                        <Badge variant="offline">{t("common:status.unavailable")}</Badge>
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
            </section>
          )}

          <section className="settings-section">
            <div className="settings-row">
              <div style={{ flex: 1 }}>
                <div style={{ fontWeight: 500 }}>{t("settings:tools.useBundledAdb")}</div>
                <div className="settings-desc">{t("settings:tools.useBundledAdbDesc")}</div>
              </div>
              <Toggle
                on={settings.useBundledAdb}
                onChange={(v) => updateSetting("useBundledAdb", v)}
              />
            </div>
            {!settings.useBundledAdb && (
              <div style={{ paddingBottom: 10 }}>
                <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600, display: "block", marginBottom: 4 }}>
                  {t("settings:tools.customAdbPath")}
                </label>
                <input
                  className="inp mono"
                  value={settings.customAdbPath}
                  onChange={(e) => updateSetting("customAdbPath", e.target.value)}
                  placeholder="C:\path\to\adb.exe"
                />
              </div>
            )}
            <div className="settings-row">
              <div style={{ flex: 1 }}>
                <div style={{ fontWeight: 500 }}>{t("settings:tools.useBundledScrcpy")}</div>
                <div className="settings-desc">{t("settings:tools.useBundledScrcpyDesc")}</div>
              </div>
              <Toggle
                on={settings.useBundledScrcpy}
                onChange={(v) => updateSetting("useBundledScrcpy", v)}
              />
            </div>
            {!settings.useBundledScrcpy && (
              <div style={{ paddingBottom: 10 }}>
                <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600, display: "block", marginBottom: 4 }}>
                  {t("settings:tools.customScrcpyPath")}
                </label>
                <input
                  className="inp mono"
                  value={settings.customScrcpyPath}
                  onChange={(e) => updateSetting("customScrcpyPath", e.target.value)}
                  placeholder="C:\path\to\scrcpy.exe"
                />
              </div>
            )}
          </section>

          <section className="settings-section">
            <div className="settings-row">
              <div>
                <div style={{ fontWeight: 500 }}>{t("settings:tools.autoScan")}</div>
                <div className="settings-desc">{t("settings:tools.autoScanDesc")}</div>
              </div>
              <Toggle
                on={settings.autoScanDevices}
                onChange={(v) => updateSetting("autoScanDevices", v)}
              />
            </div>
            <div className="settings-row">
              <div>
                <div style={{ fontWeight: 500 }}>{t("settings:tools.scanInterval")}</div>
                <div className="settings-desc">{t("settings:tools.scanIntervalDesc")}</div>
              </div>
              <input
                className="inp"
                type="text"
                inputMode="numeric"
                pattern="[0-9]*"
                value={String(settings.deviceScanIntervalSeconds)}
                onChange={(e) => {
                  const digits = e.target.value.replace(/\D/g, "");
                  if (!digits) return;
                  updateSetting("deviceScanIntervalSeconds", clampScanInterval(parseInt(digits, 10)));
                }}
                style={{ width: 88, textAlign: "center" }}
                disabled={!settings.autoScanDevices}
              />
            </div>
          </section>
        </>
      );
    }

    if (activeSection === 2) {
      const updateDefaultConfig = (patch: Partial<MirrorConfig>) => {
        updateSetting("defaultMirrorConfig", { ...settings.defaultMirrorConfig, ...patch });
        updateMirrorConfig(patch);
      };
      const chooseRecordDirectory = async () => {
        const selected = await open({ directory: true, multiple: false });
        if (typeof selected === "string") {
          updateDefaultConfig({ recordDirectory: selected });
        }
      };
      const applyDefaultPreset = (config: MirrorConfig) => {
        const patch = {
          maxSize: config.maxSize,
          videoBitRate: config.videoBitRate,
          maxFps: config.maxFps,
          videoCodec: config.videoCodec,
        };
        updateSetting("defaultMirrorConfig", { ...settings.defaultMirrorConfig, ...patch });
        applyMirrorPreset(config);
      };
      const presets = getPresets((key) => t(`mirror:${key}`));
      const optRes = getOptRes(t);
      const optCodec = getOptCodec(t);
      const activePreset = presets.find(
        (preset) =>
          preset.config.maxSize === settings.defaultMirrorConfig.maxSize &&
          preset.config.videoBitRate === settings.defaultMirrorConfig.videoBitRate &&
          preset.config.maxFps === settings.defaultMirrorConfig.maxFps &&
          preset.config.videoCodec === settings.defaultMirrorConfig.videoCodec
      );

      return (
        <section className="settings-section">
          <h2 className="settings-section-title">{t("mirror:title")}</h2>

          <div className="grid4 preset-grid" style={{ marginBottom: 12 }}>
            {presets.map((preset) => (
              <button
                key={preset.id}
                className={`preset-card${activePreset?.id === preset.id ? " selected" : ""}`}
                onClick={() => applyDefaultPreset(preset.config)}
                type="button"
              >
                <span style={{ fontWeight: 600, fontSize: 13, marginBottom: 2 }}>{preset.name}</span>
                <span style={{ color: "var(--t2)", fontSize: 11 }}>{preset.description}</span>
              </button>
            ))}
          </div>

          <div className="card">
          <div className="grid4 config-grid" style={{ marginBottom: 12 }}>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:resolution")}</label>
              <Dropdown
                className="settings-config-select"
                value={settings.defaultMirrorConfig.maxSize}
                onChange={(v) => updateDefaultConfig({ maxSize: v })}
                options={optRes}
              />
            </div>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:bitrate")}</label>
              <Dropdown
                className="settings-config-select"
                value={settings.defaultMirrorConfig.videoBitRate}
                onChange={(v) => updateDefaultConfig({ videoBitRate: v })}
                options={OPT_BR}
              />
            </div>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:fps")}</label>
              <Dropdown
                className="settings-config-select"
                value={settings.defaultMirrorConfig.maxFps}
                onChange={(v) => updateDefaultConfig({ maxFps: v })}
                options={OPT_FPS}
              />
            </div>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:codec")}</label>
              <Dropdown
                className="settings-config-select"
                value={settings.defaultMirrorConfig.videoCodec}
                onChange={(v) => updateDefaultConfig({ videoCodec: v })}
                options={optCodec}
              />
            </div>
          </div>
          <div className="settings-row">
            <div>
              <div style={{ fontWeight: 500 }}>{t("mirror:readOnly")}</div>
              <div className="settings-desc">{t("mirror:readOnlyDesc")}</div>
            </div>
            <Toggle
              on={settings.defaultMirrorConfig.noControl}
              onChange={(v) => updateDefaultConfig({ noControl: v })}
            />
          </div>
          <div className="settings-row">
            <div>
              <div style={{ fontWeight: 500 }}>{t("mirror:stayAwake")}</div>
              <div className="settings-desc">{t("mirror:stayAwakeDesc")}</div>
            </div>
            <Toggle
              on={settings.defaultMirrorConfig.stayAwake}
              onChange={(v) => updateDefaultConfig({ stayAwake: v })}
            />
          </div>
          <div className="settings-row">
            <div>
              <div style={{ fontWeight: 500 }}>{t("mirror:turnScreenOff")}</div>
              <div className="settings-desc">{t("mirror:turnScreenOffDesc")}</div>
            </div>
            <Toggle
              on={settings.defaultMirrorConfig.turnScreenOff}
              onChange={(v) => updateDefaultConfig({ turnScreenOff: v })}
            />
          </div>
          <div className="grid4 config-grid" style={{ marginTop: 12, marginBottom: 12 }}>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:recordMode")}</label>
              <Dropdown
                className="settings-config-select"
                value={settings.defaultMirrorConfig.recordMode}
                onChange={(v) => updateDefaultConfig({ recordMode: v as MirrorConfig["recordMode"] })}
                options={recordModeOptions}
              />
            </div>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:recordFormat")}</label>
              <Dropdown
                className="settings-config-select"
                value={settings.defaultMirrorConfig.recordFormat}
                onChange={(v) => updateDefaultConfig({ recordFormat: v as MirrorConfig["recordFormat"] })}
                options={RECORD_FORMAT_OPTIONS}
              />
            </div>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:orientation")}</label>
              <Dropdown
                className="settings-config-select"
                value={settings.defaultMirrorConfig.orientation}
                onChange={(v) => updateDefaultConfig({ orientation: v as MirrorConfig["orientation"] })}
                options={orientationOptions}
              />
            </div>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:recordDirectory")}</label>
              <button className="btn btn-s" type="button" onClick={chooseRecordDirectory}>
                {settings.defaultMirrorConfig.recordDirectory ? t("mirror:changeDirectory") : t("mirror:selectDirectory")}
              </button>
            </div>
          </div>
          {settings.defaultMirrorConfig.recordDirectory && (
            <div className="mono" style={{ color: "var(--t2)", fontSize: 11, marginBottom: 12, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              {settings.defaultMirrorConfig.recordDirectory}
            </div>
          )}
          <div className="settings-row">
            <div>
              <div style={{ fontWeight: 500 }}>{t("mirror:alwaysOnTop")}</div>
              <div className="settings-desc">{t("mirror:alwaysOnTopDesc")}</div>
            </div>
            <Toggle
              on={settings.defaultMirrorConfig.alwaysOnTop}
              onChange={(v) => updateDefaultConfig({ alwaysOnTop: v })}
            />
          </div>
          <div className="settings-row">
            <div>
              <div style={{ fontWeight: 500 }}>{t("mirror:windowBorderless")}</div>
              <div className="settings-desc">{t("mirror:windowBorderlessDesc")}</div>
            </div>
            <Toggle
              on={settings.defaultMirrorConfig.windowBorderless}
              onChange={(v) => updateDefaultConfig({ windowBorderless: v })}
            />
          </div>
          <div className="settings-row">
            <div>
              <div style={{ fontWeight: 500 }}>{t("mirror:printFps")}</div>
              <div className="settings-desc">{t("mirror:printFpsDesc")}</div>
            </div>
            <Toggle
              on={settings.defaultMirrorConfig.printFps}
              onChange={(v) => updateDefaultConfig({ printFps: v })}
            />
          </div>
          <div className="settings-row">
            <div>
              <div style={{ fontWeight: 500 }}>{t("mirror:audioEnabled")}</div>
              <div className="settings-desc">{t("mirror:audioEnabledDesc")}</div>
            </div>
            <Toggle
              on={settings.defaultMirrorConfig.audioEnabled}
              onChange={(v) => updateDefaultConfig({ audioEnabled: v })}
            />
          </div>
          <div className="grid4 config-grid" style={{ marginTop: 12 }}>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:audioSource")}</label>
              <Dropdown
                className="settings-config-select"
                value={settings.defaultMirrorConfig.audioSource}
                onChange={(v) => updateDefaultConfig({ audioSource: v as MirrorConfig["audioSource"] })}
                options={audioSourceOptions}
              />
            </div>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:audioCodec")}</label>
              <Dropdown
                className="settings-config-select"
                value={settings.defaultMirrorConfig.audioCodec}
                onChange={(v) => updateDefaultConfig({ audioCodec: v as MirrorConfig["audioCodec"] })}
                options={AUDIO_CODEC_OPTIONS}
              />
            </div>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:audioDuplicate")}</label>
              <Toggle
                on={settings.defaultMirrorConfig.audioDuplicate}
                onChange={(v) => updateDefaultConfig({ audioDuplicate: v })}
              />
            </div>
            <div className="col">
              <label style={{ fontSize: 11, color: "var(--t2)", fontWeight: 600 }}>{t("mirror:requireAudio")}</label>
              <Toggle
                on={settings.defaultMirrorConfig.requireAudio}
                onChange={(v) => updateDefaultConfig({ requireAudio: v })}
              />
            </div>
          </div>
          </div>
        </section>
      );
    }

    if (activeSection === 3) {
      return (
        <section className="settings-section">
          <div className="settings-row">
            <div>
              <div style={{ fontWeight: 500 }}>{t("settings:logs.retention")}</div>
              <div className="settings-desc">{t("settings:logs.retentionDesc")}</div>
            </div>
            <input
              className="inp"
              type="text"
              inputMode="numeric"
              pattern="[0-9]*"
              value={String(settings.logRetentionDays)}
              onChange={(e) => {
                const digits = e.target.value.replace(/\D/g, "");
                if (!digits) return;
                const value = parseInt(digits, 10);
                updateSetting("logRetentionDays", Math.min(365, Math.max(1, value)));
              }}
              style={{ width: 80, textAlign: "center" }}
            />
          </div>
        </section>
      );
    }

    // About section (activeSection === 4)
    return (
      <section>
        <h2 className="settings-section-title" style={{ marginBottom: 12 }}>
          <span className="settings-badge-accent">DeviceDeck</span>
          <span className="settings-badge-version">v{updateState.currentVersion}</span>
        </h2>
        <div className="settings-desc" style={{ marginBottom: 16 }}>
          {t("settings:about.description")}
        </div>
        <div className="settings-info-lines">
          <div>{t("settings:about.author")}</div>
          <div>{t("settings:about.techStack")}</div>
        </div>
        <div className="settings-links">
          <a
            className="settings-link-btn"
            href="https://github.com/shenjianZ/DeviceDeck"
            target="_blank"
            rel="noopener noreferrer"
          >
            <Code2 size={14} />
            {t("settings:about.github")}
          </a>
          <a
            className="settings-link-btn"
            href="https://github.com/shenjianZ/DeviceDeck/issues"
            target="_blank"
            rel="noopener noreferrer"
          >
            <MessageSquare size={14} />
            {t("settings:about.feedback")}
          </a>
        </div>

        <section className="settings-section" style={{ marginTop: 24 }}>
          <h2 className="settings-section-title">{t("settings:about.updateSection")}</h2>
          <div className="settings-row">
            <div>
              <span style={{ fontWeight: 500 }}>{t("settings:about.autoUpdateEnabled")}</span>
              <div className="settings-desc">{t("settings:about.autoUpdateEnabledDesc")}</div>
            </div>
            <Toggle
              on={settings.autoUpdateEnabled || false}
              onChange={(v) => updateSetting("autoUpdateEnabled", v)}
            />
          </div>
          <div style={{ padding: "8px 0" }}>
            <div className="settings-row-inner">
              <span>{t("settings:about.currentVersion")}</span>
              <span className="settings-value">v{updateState.currentVersion}</span>
            </div>
            {updateState.latestVersion && (
              <div style={{ fontSize: 11, color: "var(--t2)", marginTop: 2 }}>
                v{updateState.latestVersion}
              </div>
            )}
          </div>
          <div style={{ padding: "8px 0" }}>
            <span style={{ color: "var(--t2)", fontSize: 13 }}>
              {(() => {
                switch (updateState.status) {
                  case "idle": return t("settings:about.updateStatusIdle");
                  case "checking": return t("settings:about.updateStatusChecking");
                  case "available": return t("settings:about.updateStatusAvailable");
                  case "downloading": return t("settings:about.updateStatusDownloading");
                  case "downloaded": return t("settings:about.updateStatusDownloaded");
                  case "up-to-date": return t("settings:about.updateStatusUpToDate");
                  case "error": return updateState.error || t("settings:about.updateStatusError");
                  default: return "";
                }
              })()}
            </span>
            {updateState.status === "downloaded" && (
              <div style={{ fontSize: 11, color: "var(--acc)", marginTop: 4 }}>
                {t("settings:about.updateReadyToInstall")}
              </div>
            )}
          </div>
          {updateState.status === "downloading" && updateState.contentLength !== null && (
            <div style={{ marginBottom: 12 }}>
              <div className="settings-progress-bar">
                <div
                  className="settings-progress-fill"
                  style={{
                    width: `${Math.min(100, Math.round((updateState.downloadedBytes / updateState.contentLength) * 100))}%`,
                  }}
                />
              </div>
              <div style={{ fontSize: 11, color: "var(--t2)", marginTop: 4 }}>
                {Math.round((updateState.downloadedBytes / updateState.contentLength) * 100)}%
              </div>
            </div>
          )}
          <div style={{ display: "flex", gap: 8 }}>
            <button
              className="btn btn-p"
              type="button"
              disabled={updateState.status === "checking" || updateState.status === "downloading"}
              onClick={() => checkForUpdates()}
            >
              <RefreshCw size={14} className={updateState.status === "checking" ? "spin" : ""} />
              {t("settings:about.updateCheck")}
            </button>
            <button
              className="btn btn-s"
              type="button"
              disabled={updateState.status !== "available" && updateState.status !== "downloaded"}
              onClick={() =>
                updateState.status === "downloaded" ? installUpdate() : downloadUpdate()
              }
            >
              <Download size={14} />
              {updateState.status === "downloaded"
                ? t("settings:about.updateInstall")
                : t("settings:about.updateDownload")}
            </button>
          </div>
        </section>
      </section>
    );
  }

  return (
    <div className="settings-page">
      <nav className="settings-nav">
        {settingsMenu.map((item, index) => {
          const Icon = item.icon;
          return (
            <button
              key={item.label}
              className={`settings-nav-item${index === activeSection ? " active" : ""}`}
              onClick={() => setActiveSection(index)}
              type="button"
            >
              <Icon size={14} />
              {item.label}
            </button>
          );
        })}
      </nav>

      <main className="settings-content">
        {renderSection()}
      </main>

      <style>{`
        @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
        .spin { animation: spin 1s linear infinite; }
      `}</style>
    </div>
  );
}

function clampScanInterval(value: number): number {
  if (!Number.isFinite(value)) return 30;
  return Math.min(600, Math.max(5, Math.round(value)));
}
