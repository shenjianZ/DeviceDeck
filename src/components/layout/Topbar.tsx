import { Moon, Sun } from "lucide-react";
import type { EnvironmentStatus } from "../../types";

interface TopbarProps {
  title: string;
  theme: "dark" | "light";
  onToggleTheme: () => void;
  environment: EnvironmentStatus | null;
}

export function Topbar({ title, theme, onToggleTheme, environment }: TopbarProps) {
  const adbOk = environment?.adb?.available ?? false;
  const scrcpyOk = environment?.scrcpy?.available ?? false;

  return (
    <div className="topbar">
      <div className="topbar-title">{title}</div>
      <div className="topbar-status">
        <span className={`topbar-dot${adbOk ? " ok" : " err"}`} />
        ADB
        <span className={`topbar-dot${scrcpyOk ? " ok" : " err"}`} style={{ marginLeft: 4 }} />
        Scrcpy
      </div>
      <button
        className="topbar-btn"
        onClick={onToggleTheme}
        title={theme === "dark" ? "切换亮色" : "切换暗色"}
        type="button"
      >
        {theme === "dark" ? <Sun size={16} /> : <Moon size={16} />}
      </button>
    </div>
  );
}
