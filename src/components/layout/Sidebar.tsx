import { FileText, LayoutGrid, Monitor, Settings, Smartphone } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { Page } from "../../types";

const NAV_ITEMS: { id: Page; icon: typeof LayoutGrid; labelKey: string }[] = [
  { id: "dashboard", icon: LayoutGrid, labelKey: "sidebar:dashboard" },
  { id: "devices", icon: Smartphone, labelKey: "sidebar:devices" },
  { id: "mirror", icon: Monitor, labelKey: "sidebar:mirror" },
  { id: "logs", icon: FileText, labelKey: "sidebar:logs" },
  { id: "settings", icon: Settings, labelKey: "sidebar:settings" },
];

interface SidebarProps {
  page: Page;
  onNav: (page: Page) => void;
}

export function Sidebar({ page, onNav }: SidebarProps) {
  const { t } = useTranslation(["sidebar"]);

  return (
    <div className="sidebar">
      {NAV_ITEMS.map((n) => {
        const Icon = n.icon;
        const active = page === n.id;
        return (
          <button
            key={n.id}
            className={`sb-item${active ? " active" : ""}`}
            onClick={() => onNav(n.id)}
            title={t(n.labelKey)}
            type="button"
          >
            <Icon size={18} />
            <span className="sb-label">{t(n.labelKey)}</span>
          </button>
        );
      })}
    </div>
  );
}
