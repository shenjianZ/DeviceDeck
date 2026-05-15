import { useEffect, useRef } from "react";
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
  const ref = useRef<HTMLDivElement>(null);
  const PageComponent = PAGES[page];

  useEffect(() => {
    ref.current?.closest(".content")?.scrollTo({ top: 0 });
  }, [page]);

  return <div ref={ref}><PageComponent /></div>;
}

export { usePageStore as usePage };
