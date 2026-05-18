import { useEffect, useState } from "react";
import { Moon, Sun, Minus, Square, X, Pin } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useTranslation } from "react-i18next";
import type { EnvironmentStatus } from "../../types";

let appWindow: ReturnType<typeof getCurrentWindow> | null = null;
try {
  appWindow = getCurrentWindow();
} catch {
  // non-Tauri environment
}

interface TopbarProps {
  theme: "dark" | "light";
  onToggleTheme: () => void;
  environment: EnvironmentStatus | null;
}

export function Topbar({ theme, onToggleTheme, environment }: TopbarProps) {
  const { t } = useTranslation(["topbar"]);
  const adbOk = environment?.adb?.available ?? false;
  const scrcpyOk = environment?.scrcpy?.available ?? false;
  const [isMaximized, setIsMaximized] = useState(false);
  const [isPinned, setIsPinned] = useState(false);

  useEffect(() => {
    if (!appWindow) return;
    appWindow.isMaximized().then(setIsMaximized);
    const unlisten = appWindow.onResized(() => {
      appWindow?.isMaximized().then(setIsMaximized);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleMinimize = () => {
    appWindow?.minimize();
  };

  const handleToggleMaximize = () => {
    appWindow?.toggleMaximize();
  };

  const handleClose = () => {
    appWindow?.close();
  };

  const handleTogglePin = async () => {
    if (!appWindow) return;
    const pinned = await appWindow.isAlwaysOnTop();
    await appWindow.setAlwaysOnTop(!pinned);
    setIsPinned(!pinned);
  };

  return (
    <header className="topbar" data-tauri-drag-region>
      <div className="topbar-left">
        <div className="topbar-brand">DeviceDeck</div>
      </div>

      <div className="topbar-center" />

      <div className="topbar-right">
        <div className="topbar-status">
          <span className={`topbar-dot${adbOk ? " ok" : " err"}`} />
          ADB
          <span className={`topbar-dot${scrcpyOk ? " ok" : " err"}`} style={{ marginLeft: 4 }} />
          Scrcpy
        </div>

        <button
          className="topbar-btn"
          onClick={onToggleTheme}
          title={theme === "dark" ? t("topbar:switchLight") : t("topbar:switchDark")}
          type="button"
        >
          {theme === "dark" ? <Sun size={14} /> : <Moon size={14} />}
        </button>

        <div className="topbar-win-controls">
          <button
            className={`topbar-win-btn${isPinned ? " active" : ""}`}
            onClick={handleTogglePin}
            title={isPinned ? t("topbar:unpin") : t("topbar:pin")}
            type="button"
          >
            <Pin size={12} />
          </button>
          <button
            className="topbar-win-btn"
            onClick={handleMinimize}
            title={t("topbar:minimize")}
            type="button"
          >
            <Minus size={12} />
          </button>
          <button
            className="topbar-win-btn"
            onClick={handleToggleMaximize}
            title={isMaximized ? t("topbar:restore") : t("topbar:maximize")}
            type="button"
          >
            {isMaximized ? (
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                <rect x="3" y="3" width="7" height="7" rx="1" stroke="currentColor" strokeWidth="1.2" />
                <path d="M3 3H2C1.45 1 1 1.45 1 2V8" stroke="currentColor" strokeWidth="1.2" />
                <rect x="2" y="2" width="7" height="7" rx="1" stroke="currentColor" strokeWidth="1.2" fill="var(--bg-0)" />
              </svg>
            ) : (
              <Square size={12} />
            )}
          </button>
          <button
            className="topbar-win-btn close"
            onClick={handleClose}
            title={t("topbar:close", "Close")}
            type="button"
          >
            <X size={12} />
          </button>
        </div>
      </div>
    </header>
  );
}
