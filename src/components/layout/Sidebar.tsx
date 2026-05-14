import { FileText, LayoutGrid, Monitor, Settings, Smartphone } from "lucide-react";
import type { Page } from "../../types";

const NAV: { id: Page; icon: typeof LayoutGrid; label: string }[] = [
  { id: "dashboard", icon: LayoutGrid, label: "仪表盘" },
  { id: "devices", icon: Smartphone, label: "设备" },
  { id: "mirror", icon: Monitor, label: "投屏" },
  { id: "logs", icon: FileText, label: "日志" },
  { id: "settings", icon: Settings, label: "设置" },
];

interface SidebarProps {
  page: Page;
  onNav: (page: Page) => void;
}

export function Sidebar({ page, onNav }: SidebarProps) {
  return (
    <div className="sidebar">
      {NAV.map((n) => {
        const Icon = n.icon;
        const active = page === n.id;
        return (
          <button
            key={n.id}
            className={`sb-item${active ? " active" : ""}`}
            onClick={() => onNav(n.id)}
            title={n.label}
            type="button"
          >
            <Icon size={18} />
            <span className="sb-tip">{n.label}</span>
          </button>
        );
      })}
    </div>
  );
}
