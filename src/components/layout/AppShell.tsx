import { usePageStore } from "../../stores/pageStore";
import { useDeviceStore } from "../../stores/deviceStore";
import { Sidebar } from "./Sidebar";
import { Topbar } from "./Topbar";
import { Routes } from "../../app/routes";
import { PAGE_TITLES } from "../../lib/presets";

interface AppShellProps {
  theme: "dark" | "light";
  onToggleTheme: () => void;
}

export function AppShell({ theme, onToggleTheme }: AppShellProps) {
  const page = usePageStore((s) => s.page);
  const setPage = usePageStore((s) => s.setPage);
  const environment = useDeviceStore((s) => s.environment);

  const title = PAGE_TITLES[page] ?? page;

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
    </div>
  );
}
