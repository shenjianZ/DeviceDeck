import {
  createContext,
  useContext,
  useReducer,
  type ReactNode,
} from "react";
import { DEVICES, INIT_LOGS, INIT_SESSIONS } from "../data/mock";
import type { AppSettings, Device, LogEntry, MirrorConfig, Page, Session } from "../types";

interface AppState {
  page: Page;
  theme: "dark" | "light";
  devices: Device[];
  sessions: Session[];
  logs: LogEntry[];
  selectedDeviceId: string | null;
  settings: AppSettings;
}

type Action =
  | { type: "SET_PAGE"; page: Page }
  | { type: "TOGGLE_THEME" }
  | { type: "SELECT_DEVICE"; id: string | null }
  | { type: "START_SESSION"; deviceId: string; config: MirrorConfig }
  | { type: "STOP_SESSION"; sessionId: string }
  | { type: "CLEAR_LOGS" }
  | { type: "SAVE_SETTINGS"; settings: AppSettings };

const initialState: AppState = {
  page: "dashboard",
  theme: "dark",
  devices: DEVICES,
  sessions: INIT_SESSIONS,
  logs: INIT_LOGS,
  selectedDeviceId: null,
  settings: {
    useBundledAdb: true,
    useBundledScrcpy: false,
    customAdbPath: "C:\\platform-tools\\adb.exe",
    customScrcpyPath: "C:\\scrcpy\\scrcpy.exe",
    defaultMirrorConfig: {
      maxSize: "1080",
      videoBitRate: "8M",
      maxFps: "60",
      noControl: false,
      stayAwake: true,
      turnScreenOff: false,
    },
    theme: "dark",
    logRetentionDays: 7,
  },
};

function reducer(state: AppState, action: Action): AppState {
  switch (action.type) {
    case "SET_PAGE":
      return { ...state, page: action.page };

    case "TOGGLE_THEME":
      return { ...state, theme: state.theme === "dark" ? "light" : "dark" };

    case "SELECT_DEVICE":
      return { ...state, selectedDeviceId: action.id };

    case "START_SESSION": {
      const device = state.devices.find((item) => item.id === action.deviceId);
      if (!device) return state;

      const session: Session = {
        id: `s${Date.now()}`,
        deviceSerial: device.serial,
        platform: "android",
        processId: Math.floor(Math.random() * 50000 + 10000),
        status: "running",
        startedAt: Date.now(),
        config: { ...action.config },
      };
      const log: LogEntry = {
        id: `l${Date.now()}`,
        time: new Date(),
        source: "scrcpy",
        level: "info",
        deviceSerial: device.serial,
        message: `投屏已启动 — ${action.config.maxSize}p / ${action.config.videoBitRate} / ${action.config.maxFps}fps`,
      };

      return {
        ...state,
        sessions: [...state.sessions, session],
        logs: [...state.logs, log],
      };
    }

    case "STOP_SESSION": {
      const session = state.sessions.find((item) => item.id === action.sessionId);
      if (!session) return state;

      const device = state.devices.find((item) => item.serial === session.deviceSerial);
      const log: LogEntry = {
        id: `l${Date.now()}`,
        time: new Date(),
        source: "scrcpy",
        level: "info",
        deviceSerial: session.deviceSerial,
        message: `${device?.name ?? session.deviceSerial} 投屏已停止`,
      };

      return {
        ...state,
        sessions: state.sessions.map((item) =>
          item.id === action.sessionId ? { ...item, status: "stopped" as const } : item
        ),
        logs: [...state.logs, log],
      };
    }

    case "CLEAR_LOGS":
      return { ...state, logs: [] };

    case "SAVE_SETTINGS":
      return { ...state, settings: action.settings };

    default:
      return state;
  }
}

const AppContext = createContext<{
  state: AppState;
  dispatch: React.Dispatch<Action>;
} | null>(null);

export function AppProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(reducer, initialState);
  return (
    <AppContext.Provider value={{ state, dispatch }}>
      {children}
    </AppContext.Provider>
  );
}

export function useApp() {
  const context = useContext(AppContext);
  if (!context) throw new Error("useApp must be used within AppProvider");
  return context;
}
