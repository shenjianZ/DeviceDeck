import { describe, expect, it } from "vitest";
import devicesPageSource from "./DevicesPage.tsx?raw";
import settingsPageSource from "./SettingsPage.tsx?raw";

describe("advanced QtScrcpy-compatible controls", () => {
  it("exposes recording, window, orientation, and audio settings", () => {
    expect(settingsPageSource).toContain("recordMode");
    expect(settingsPageSource).toContain("recordFormat");
    expect(settingsPageSource).toContain("alwaysOnTop");
    expect(settingsPageSource).toContain("windowBorderless");
    expect(settingsPageSource).toContain("printFps");
    expect(settingsPageSource).toContain("audioEnabled");
    expect(settingsPageSource).toContain("audioSource");
    expect(settingsPageSource).toContain("audioCodec");
  });

  it("exposes screenshot, apk install, file push, and shortcut actions", () => {
    expect(devicesPageSource).toContain("takeScreenshot");
    expect(devicesPageSource).toContain("installApk");
    expect(devicesPageSource).toContain("pushFile");
    expect(devicesPageSource).toContain("runKeyAction");
    expect(devicesPageSource).toContain("runShellCommand");
    expect(devicesPageSource).toContain("/sdcard/Download/DeviceDeck");
  });
});
