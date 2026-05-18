import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettingsStore } from "../stores/settingsStore";

interface WelcomeOverlayProps {
  onComplete: () => void;
}

export function WelcomeOverlay({ onComplete }: WelcomeOverlayProps) {
  const [locale, setLocale] = useState<"zh-CN" | "en">(
    () => (localStorage.getItem("dd-locale") as "zh-CN" | "en") || "zh-CN"
  );
  const updateSetting = useSettingsStore((s) => s.updateSetting);
  const { t, i18n } = useTranslation("welcome");

  const handleContinue = async () => {
    localStorage.setItem("dd-locale", locale);
    i18n.changeLanguage(locale);
    await updateSetting("locale", locale);
    await updateSetting("firstRun", false);
    onComplete();
  };

  const selectLocale = (lang: "zh-CN" | "en") => {
    setLocale(lang);
    i18n.changeLanguage(lang);
  };

  return (
    <div className="welcome-overlay">
      <div className="welcome-card">
        <div className="welcome-logo">DD</div>
        <h1 className="welcome-title">{t("title")}</h1>
        <p className="welcome-subtitle">{t("subtitle")}</p>
        <div className="welcome-lang-options">
          <button
            className={`welcome-lang-btn${locale === "zh-CN" ? " active" : ""}`}
            onClick={() => selectLocale("zh-CN")}
          >
            <span className="welcome-lang-label">简体中文</span>
          </button>
          <button
            className={`welcome-lang-btn${locale === "en" ? " active" : ""}`}
            onClick={() => selectLocale("en")}
          >
            <span className="welcome-lang-label">English</span>
          </button>
        </div>
        <button className="btn btn-p welcome-continue" onClick={handleContinue}>
          {t("continue")}
        </button>
      </div>
    </div>
  );
}
