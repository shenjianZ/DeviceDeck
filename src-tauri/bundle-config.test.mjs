import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, test } from "vitest";

function readTauriConfig(fileName) {
  const configPath = join(process.cwd(), "src-tauri", fileName);

  expect(existsSync(configPath)).toBe(true);

  return JSON.parse(readFileSync(configPath, "utf8"));
}

function expectBundledFile(relativePath) {
  expect(existsSync(join(process.cwd(), "src-tauri", relativePath))).toBe(true);
}

describe("Windows Tauri bundled tools", () => {
  test("maps adb, scrcpy, and runtime dependencies into resources/binaries", () => {
    const config = readTauriConfig("tauri.windows.conf.json");
    const resources = config.bundle?.resources;

    expect(config.bundle?.targets).toBeUndefined();
    expect(resources).toEqual(
      expect.objectContaining({
        "binaries/windows-x64/adb.exe": "binaries/windows-x64/adb.exe",
        "binaries/windows-x64/scrcpy.exe": "binaries/windows-x64/scrcpy.exe",
        "binaries/windows-x64/scrcpy-server": "binaries/windows-x64/scrcpy-server",
        "binaries/windows-x64/AdbWinApi.dll": "binaries/windows-x64/AdbWinApi.dll",
        "binaries/windows-x64/AdbWinUsbApi.dll": "binaries/windows-x64/AdbWinUsbApi.dll",
        "binaries/windows-x64/SDL3.dll": "binaries/windows-x64/SDL3.dll",
        "binaries/windows-x64/avcodec-62.dll": "binaries/windows-x64/avcodec-62.dll",
        "binaries/windows-x64/avformat-62.dll": "binaries/windows-x64/avformat-62.dll",
        "binaries/windows-x64/avutil-60.dll": "binaries/windows-x64/avutil-60.dll",
        "binaries/windows-x64/libusb-1.0.dll": "binaries/windows-x64/libusb-1.0.dll",
        "binaries/windows-x64/swresample-6.dll": "binaries/windows-x64/swresample-6.dll",
      }),
    );
  });
});

describe("Linux Tauri bundled tools", () => {
  test("maps target-specific adb and scrcpy into stable resource names", () => {
    const config = readTauriConfig("tauri.linux.conf.json");

    expect(config.bundle?.targets).toBeUndefined();
    expect(config.bundle?.resources).toEqual({
      "binaries/linux-x64/adb": "binaries/linux-x64/adb",
      "binaries/linux-x64/scrcpy": "binaries/linux-x64/scrcpy",
      "binaries/linux-x64/scrcpy-server": "binaries/scrcpy-server",
    });
  });
});

describe("macOS Tauri bundled tools", () => {
  test("maps target-specific adb and scrcpy into stable resource names", () => {
    const config = readTauriConfig("tauri.macos.conf.json");

    expect(config.bundle?.targets).toBeUndefined();
    expect(config.bundle?.resources).toEqual(
      expect.objectContaining({
        "binaries/macos-aarch64/adb": "binaries/macos-aarch64/adb",
        "binaries/macos-aarch64/scrcpy": "binaries/macos-aarch64/scrcpy",
        "binaries/macos-aarch64/scrcpy-server": "binaries/macos-aarch64/scrcpy-server",
        "binaries/macos-x64/adb": "binaries/macos-x64/adb",
        "binaries/macos-x64/scrcpy": "binaries/macos-x64/scrcpy",
        "binaries/macos-x64/scrcpy-server": "binaries/macos-x64/scrcpy-server",
      }),
    );
  });
});

describe("Bundled tool files", () => {
  test("includes platform-scoped adb, scrcpy, and scrcpy-server files", () => {
    for (const dir of ["windows-x64", "linux-x64", "macos-aarch64", "macos-x64"]) {
      const executableExtension = dir === "windows-x64" ? ".exe" : "";

      expectBundledFile(`binaries/${dir}/adb${executableExtension}`);
      expectBundledFile(`binaries/${dir}/scrcpy${executableExtension}`);
      expectBundledFile(`binaries/${dir}/scrcpy-server`);
    }
  });
});
