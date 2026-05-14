import { useTranslation } from "react-i18next";
import { usePageStore } from "../../stores/pageStore";
import { useDeviceStore } from "../../stores/deviceStore";
import { Sidebar } from "./Sidebar";
import { Topbar } from "./Topbar";
import { Routes } from "../../app/routes";
import { getPageTitles } from "../../lib/presets";
import { Toast } from "../ui/Toast";

interface AppShellProps {
  theme: "dark" | "light";
  onToggleTheme: () => void;
}

export function AppShell({ theme, onToggleTheme }: AppShellProps) {
  const { t } = useTranslation(["sidebar"]);
  const page = usePageStore((s) => s.page);
  const setPage = usePageStore((s) => s.setPage);
  const environment = useDeviceStore((s) => s.environment);

  const titles = getPageTitles(t);
  const title = titles[page] ?? page;

  return (
    <div className="app">
      <Sidebar page={page} onNav={setPage} />
      <div className="main">
        <Topbar
          title={title}
          theme={theme}
          onToggleTheme={onToggleTheme}
          environment={environment}
        />
        <div className="content">
          <Routes />
        </div>
      </div>
      <Toast />
    </div>
  );
}
