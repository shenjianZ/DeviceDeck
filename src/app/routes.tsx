import { useEffect, useRef } from "react";
import { usePageStore } from "../stores/pageStore";
import { DashboardPage } from "../pages/DashboardPage";
import { DevicesPage } from "../pages/DevicesPage";
import { MirrorPage } from "../pages/MirrorPage";
import { LogsPage } from "../pages/LogsPage";
import { SettingsPage } from "../pages/SettingsPage";
import { FileTransferPage } from "../pages/FileTransferPage";

import type { Page } from "../types";

const PAGE_COMPONENTS: { id: Page; Component: React.ComponentType }[] = [
  { id: "dashboard", Component: DashboardPage },
  { id: "devices", Component: DevicesPage },
  { id: "mirror", Component: MirrorPage },
  { id: "transfer", Component: FileTransferPage },
  { id: "logs", Component: LogsPage },
  { id: "settings", Component: SettingsPage },
];

export function Routes() {
  const page = usePageStore((s) => s.page);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    ref.current?.closest(".content")?.scrollTo({ top: 0 });
  }, [page]);

  return (
    <div ref={ref}>
      {PAGE_COMPONENTS.map(({ id, Component }) => (
        <div key={id} style={{ display: page === id ? "contents" : "none" }}>
          <Component />
        </div>
      ))}
    </div>
  );
}

export { usePageStore as usePage };
