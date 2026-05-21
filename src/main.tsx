import React from "react";
import ReactDOM from "react-dom/client";
import "./i18n";
import { initTheme } from "./lib/theme";
import { AppProviders } from "./app/providers";
import "./index.css";

initTheme();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <AppProviders />
  </React.StrictMode>
);
