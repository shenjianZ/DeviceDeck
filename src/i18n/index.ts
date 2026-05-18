import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import zhCommon from "./locales/zh-CN/common.json";
import zhSettings from "./locales/zh-CN/settings.json";
import zhTopbar from "./locales/zh-CN/topbar.json";
import zhSidebar from "./locales/zh-CN/sidebar.json";
import zhMirror from "./locales/zh-CN/mirror.json";
import zhDevices from "./locales/zh-CN/devices.json";
import zhDashboard from "./locales/zh-CN/dashboard.json";
import zhLogs from "./locales/zh-CN/logs.json";
import zhWelcome from "./locales/zh-CN/welcome.json";
import zhErrors from "./locales/zh-CN/errors.json";

import enCommon from "./locales/en/common.json";
import enSettings from "./locales/en/settings.json";
import enTopbar from "./locales/en/topbar.json";
import enSidebar from "./locales/en/sidebar.json";
import enMirror from "./locales/en/mirror.json";
import enDevices from "./locales/en/devices.json";
import enDashboard from "./locales/en/dashboard.json";
import enLogs from "./locales/en/logs.json";
import enWelcome from "./locales/en/welcome.json";
import enErrors from "./locales/en/errors.json";

const resources = {
  "zh-CN": {
    common: zhCommon,
    settings: zhSettings,
    topbar: zhTopbar,
    sidebar: zhSidebar,
    mirror: zhMirror,
    devices: zhDevices,
    dashboard: zhDashboard,
    logs: zhLogs,
    welcome: zhWelcome,
    errors: zhErrors,
  },
  en: {
    common: enCommon,
    settings: enSettings,
    topbar: enTopbar,
    sidebar: enSidebar,
    mirror: enMirror,
    devices: enDevices,
    dashboard: enDashboard,
    logs: enLogs,
    welcome: enWelcome,
    errors: enErrors,
  },
};

i18n.use(initReactI18next).init({
  resources,
  lng: localStorage.getItem("dd-locale") || "zh-CN",
  fallbackLng: "zh-CN",
  ns: ["common", "settings", "topbar", "sidebar", "mirror", "devices", "dashboard", "logs", "welcome", "errors"],
  defaultNS: "common",
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
