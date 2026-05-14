import { usePageStore } from "../../stores/pageStore";
import { useDeviceStore } from "../../stores/deviceStore";
import { Sidebar } from "./Sidebar";
import { Topbar } from "./Topbar";
import { Routes } from "../../app/routes";
import { Toast } from "../ui/Toast";

interface AppShellProps {
  theme: "dark" | "light";
  onToggleTheme: () => void;
}

export function AppShell({ theme, onToggleTheme }: AppShellProps) {
  const page = usePageStore((s) => s.page);
  const setPage = usePageStore((s) => s.setPage);
  const environment = useDeviceStore((s) => s.environment);

  return (
    <div className="app">
      <Topbar
        theme={theme}
        onToggleTheme={onToggleTheme}
        environment={environment}
      />
      <div className="app-body">
        <Sidebar page={page} onNav={setPage} />
        <main className="main">
        <div className="content">
          <Routes />
        </div>
        </main>
      </div>
      <Toast />
    </div>
  );
}
