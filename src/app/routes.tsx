import { usePageStore } from "../stores/pageStore";
import { DashboardPage } from "../pages/DashboardPage";
import { DevicesPage } from "../pages/DevicesPage";
import { MirrorPage } from "../pages/MirrorPage";
import { LogsPage } from "../pages/LogsPage";
import { SettingsPage } from "../pages/SettingsPage";

const PAGES = {
  dashboard: DashboardPage,
  devices: DevicesPage,
  mirror: MirrorPage,
  logs: LogsPage,
  settings: SettingsPage,
} as const;

export function Routes() {
  const page = usePageStore((s) => s.page);
  const PageComponent = PAGES[page];
  return <PageComponent />;
}

export { usePageStore as usePage };
