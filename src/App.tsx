import { AppProvider, useApp } from "./context/AppContext";
import { Sidebar } from "./components/layout/Sidebar";
import { Topbar } from "./components/layout/Topbar";
import { DashboardPage } from "./components/pages/DashboardPage";
import { DevicesPage } from "./components/pages/DevicesPage";
import { MirrorPage } from "./components/pages/MirrorPage";
import { LogsPage } from "./components/pages/LogsPage";
import { SettingsPage } from "./components/pages/SettingsPage";
import { PAGE_TITLES } from "./data/constants";

function AppContent() {
  const { state, dispatch } = useApp();
  const { page, theme, devices, sessions } = state;

  const onlineCount = devices.filter((d) => d.status === "online").length;
  const runningCount = sessions.filter((s) => s.status === "running").length;

  return (
    <div className="app" data-theme={theme}>
      <Sidebar page={page} onNav={(p) => dispatch({ type: "SET_PAGE", page: p })} />
      <div className="main">
        <Topbar
          title={PAGE_TITLES[page]}
          theme={theme}
          onToggleTheme={() => dispatch({ type: "TOGGLE_THEME" })}
        />
        <div className="content">
          {page === "dashboard" && (
            <DashboardPage
              onlineCount={onlineCount}
              runningCount={runningCount}
              totalSessions={sessions.length}
              onNav={(p) => dispatch({ type: "SET_PAGE", page: p })}
            />
          )}
          {page === "devices" && <DevicesPage />}
          {page === "mirror" && <MirrorPage />}
          {page === "logs" && <LogsPage />}
          {page === "settings" && <SettingsPage />}
        </div>
      </div>
    </div>
  );
}

export default function App() {
  return (
    <AppProvider>
      <AppContent />
    </AppProvider>
  );
}
