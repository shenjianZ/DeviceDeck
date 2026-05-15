import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, test } from "vitest";

const configPath = join(process.cwd(), "src-tauri", "tauri.windows.conf.json");

describe("Windows Tauri bundled tools", () => {
  test("maps adb, scrcpy, and runtime dependencies into resources/binaries", () => {
    expect(existsSync(configPath)).toBe(true);

    const config = JSON.parse(readFileSync(configPath, "utf8"));
    const resources = config.bundle?.resources;

    expect(config.bundle?.targets).toBeUndefined();
    expect(resources).toEqual(
      expect.objectContaining({
        "binaries/adb-x86_64-pc-windows-msvc.exe": "binaries/adb.exe",
        "binaries/scrcpy-x86_64-pc-windows-msvc.exe": "binaries/scrcpy.exe",
        "binaries/scrcpy-server": "binaries/scrcpy-server",
        "binaries/AdbWinApi.dll": "binaries/AdbWinApi.dll",
        "binaries/AdbWinUsbApi.dll": "binaries/AdbWinUsbApi.dll",
        "binaries/SDL3.dll": "binaries/SDL3.dll",
        "binaries/avcodec-62.dll": "binaries/avcodec-62.dll",
        "binaries/avformat-62.dll": "binaries/avformat-62.dll",
        "binaries/avutil-60.dll": "binaries/avutil-60.dll",
        "binaries/libusb-1.0.dll": "binaries/libusb-1.0.dll",
        "binaries/libwinpthread-1.dll": "binaries/libwinpthread-1.dll",
        "binaries/swresample-6.dll": "binaries/swresample-6.dll",
      }),
    );
  });
});
