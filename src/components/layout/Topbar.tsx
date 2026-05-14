import { Moon, Sun } from "lucide-react";

interface TopbarProps {
  title: string;
  theme: "dark" | "light";
  onToggleTheme: () => void;
}

export function Topbar({ title, theme, onToggleTheme }: TopbarProps) {
  return (
    <div className="topbar">
      <div className="topbar-title">{title}</div>
      <div className="topbar-status">
        <span className="topbar-dot ok" />
        ADB
        <span className="topbar-dot ok" style={{ marginLeft: 4 }} />
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
