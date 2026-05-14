import { useEffect, useRef, useState } from "react";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { useDeviceStore } from "../stores/deviceStore";
import { useLogStore } from "../stores/logStore";
import { useMirrorStore } from "../stores/mirrorStore";
import { useSettingsStore } from "../stores/settingsStore";
import { applyTheme } from "../lib/theme";
import { AppShell } from "../components/layout/AppShell";

export function AppProviders() {
  const checkEnvironment = useDeviceStore((s) => s.checkEnvironment);
  const scanDevices = useDeviceStore((s) => s.scanDevices);
  const discoverWirelessDevices = useDeviceStore((s) => s.discoverWirelessDevices);
  const loadLogs = useLogStore((s) => s.loadLogs);
  const startLogListening = useLogStore((s) => s.startListening);
  const startSessionListening = useMirrorStore((s) => s.startListening);
  const loadSettings = useSettingsStore((s) => s.loadSettings);
  const settings = useSettingsStore((s) => s.settings);
  const refreshSessions = useMirrorStore((s) => s.refreshSessions);

  const [theme, setTheme] = useState<"dark" | "light">(() => {
    const saved = localStorage.getItem("dd-theme");
    return (saved === "light" || saved === "dark") ? saved : "dark";
  });

  const unlistenRefs = useRef<UnlistenFn[]>([]);

  // Sync theme from store to local state (when settings load from backend)
  useEffect(() => {
    const t = settings.theme;
    if (t === "light" || t === "dark") {
      setTheme(t);
    }
  }, [settings.theme]);

  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  useEffect(() => {
    let cancelled = false;

    const init = async () => {
      if (cancelled) return;

      const unlistenLog = await startLogListening();
      if (cancelled) {
        unlistenLog();
        return;
      }
      const unlistenSession = await startSessionListening();
      if (cancelled) {
        unlistenLog();
        unlistenSession();
        return;
      }
      unlistenRefs.current = [unlistenLog, unlistenSession];

      await Promise.all([
        checkEnvironment(),
        loadLogs(),
        loadSettings(),
        refreshSessions(),
      ]);

      const currentSettings = useSettingsStore.getState().settings;
      if (!cancelled && currentSettings.autoScanDevices) {
        await Promise.all([
          scanDevices(true),
          discoverWirelessDevices(true),
        ]);
      }
    };

    init();

    return () => {
      cancelled = true;
      unlistenRefs.current.forEach((fn) => fn());
      unlistenRefs.current = [];
    };
  }, [checkEnvironment, discoverWirelessDevices, loadLogs, loadSettings, refreshSessions, scanDevices, startLogListening, startSessionListening]);

  useEffect(() => {
    if (!settings.autoScanDevices) return;

    const intervalSeconds = clampScanInterval(settings.deviceScanIntervalSeconds);
    const timer = window.setInterval(() => {
      scanDevices(true);
      discoverWirelessDevices(true);
    }, intervalSeconds * 1000);

    return () => window.clearInterval(timer);
  }, [
    discoverWirelessDevices,
    scanDevices,
    settings.autoScanDevices,
    settings.deviceScanIntervalSeconds,
  ]);

  const toggleTheme = () => {
    setTheme((prev) => (prev === "dark" ? "light" : "dark"));
  };

  return <AppShell theme={theme} onToggleTheme={toggleTheme} />;
}

function clampScanInterval(value: number): number {
  if (!Number.isFinite(value)) return 30;
  return Math.min(600, Math.max(5, Math.round(value)));
}
