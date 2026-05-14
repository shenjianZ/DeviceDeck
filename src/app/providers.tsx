import { useEffect, useRef, useState } from "react";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { useDeviceStore } from "../stores/deviceStore";
import { useLogStore } from "../stores/logStore";
import { useMirrorStore } from "../stores/mirrorStore";
import { useSettingsStore } from "../stores/settingsStore";
import { AppShell } from "../components/layout/AppShell";

export function AppProviders() {
  const checkEnvironment = useDeviceStore((s) => s.checkEnvironment);
  const loadLogs = useLogStore((s) => s.loadLogs);
  const startLogListening = useLogStore((s) => s.startListening);
  const startSessionListening = useMirrorStore((s) => s.startListening);
  const loadSettings = useSettingsStore((s) => s.loadSettings);
  const refreshSessions = useMirrorStore((s) => s.refreshSessions);

  const [theme, setTheme] = useState<"dark" | "light">(() => {
    const saved = localStorage.getItem("dd-theme");
    return (saved === "light" || saved === "dark") ? saved : "dark";
  });

  const unlistenRefs = useRef<UnlistenFn[]>([]);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("dd-theme", theme);
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
    };

    init();

    return () => {
      cancelled = true;
      unlistenRefs.current.forEach((fn) => fn());
      unlistenRefs.current = [];
    };
  }, [checkEnvironment, loadLogs, loadSettings, refreshSessions, startLogListening, startSessionListening]);

  const toggleTheme = () => {
    setTheme((prev) => (prev === "dark" ? "light" : "dark"));
  };

  return <AppShell theme={theme} onToggleTheme={toggleTheme} />;
}
