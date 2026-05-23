<div align="center">

# DeviceDeck

**🌐 Language**: [English](#) | [中文](./README_ZH.md)

![Tauri](https://img.shields.io/badge/Tauri-2.0-FFC131?logo=tauri&logoColor=000000)
![React](https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=000000)
![TypeScript](https://img.shields.io/badge/TypeScript-5.8-3178C6?logo=typescript&logoColor=FFFFFF)
![Rust](https://img.shields.io/badge/Rust-1.70+-000000?logo=rust&logoColor=FFFFFF)
![Vite](https://img.shields.io/badge/Vite-7.0-646CFF?logo=vite&logoColor=FFFFFF)
![Tailwind CSS](https://img.shields.io/badge/Tailwind-4.0-06B6D4?logo=tailwindcss&logoColor=FFFFFF)
![Zustand](https://img.shields.io/badge/Zustand-5.0-FFB84D?logoColor=FFFFFF)
![Axum](https://img.shields.io/badge/Axum-0.8-3B82F6?logoColor=FFFFFF)
![License](https://img.shields.io/badge/License-MIT-green)

A modern Android screen mirroring and device management workbench built with Tauri 2 and Rust. Lightweight, efficient, and native-feeling.

**[Features](#-features)** • **[Screenshots](#-screenshots)** • **[Quick Start](#-quick-start)** • **[Tech Stack](#-tech-stack)** • **[Download](#-download)**

<img src="site/public/app-img/dashboard.png" alt="DeviceDeck Dashboard" width="800" />

</div>

## Features

- **Multi-Protocol Connection** — USB, WiFi direct, auto device discovery, and Android 11+ wireless pairing — connect in one click
- **HD Screen Mirroring** — Scrcpy-powered real-time mirroring at 480p to 1080p+, 15/30/60 fps, with latency as low as 12ms
- **Real-time Monitoring** — Online, offline, and unauthorized status at a glance. Manage multiple devices across their full lifecycle
- **Custom Encoding** — Switch between H.264/H.265/AV1, fine-tune bitrate from 1-16 Mbps. Four presets: Performance, Balanced, Quality, Ultra
- **Multi-Source Logging** — Aggregated System/ADB/Scrcpy logs with keyword filtering and auto-cleanup. Never miss a debug message
- **File Transfer** — USB wired transfer with full file browser, and WiFi wireless transfer via mobile browser with QR code access
- **Environment Detection** — Automatic detection of ADB and Scrcpy toolchain availability. Quickly diagnose environment issues on first use

## Screenshots

<div align="center">
<table>
  <tr>
    <td align="center"><img src="site/public/app-img/dashboard.png" alt="Dashboard" width="400" /><br /><b>Dashboard</b></td>
    <td align="center"><img src="site/public/app-img/device.png" alt="Device Details" width="400" /><br /><b>Device Details</b></td>
  </tr>
  <tr>
    <td align="center"><img src="site/public/app-img/mirror.png" alt="Screen Mirror" width="400" /><br /><b>Screen Mirror</b></td>
    <td align="center"><img src="site/public/app-img/transfer-usb.png" alt="USB Transfer" width="400" /><br /><b>USB Transfer</b></td>
  </tr>
  <tr>
    <td align="center"><img src="site/public/app-img/transfer-wifi.png" alt="WiFi Transfer" width="400" /><br /><b>WiFi Transfer</b></td>
    <td align="center"><img src="site/public/app-img/settings-mirror.png" alt="Mirror Settings" width="400" /><br /><b>Mirror Settings</b></td>
  </tr>
</table>
</div>

## Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | React 19, TypeScript 5.8, Zustand 5, TailwindCSS 4, Vite 7, i18next |
| Backend | Tauri 2, Rust, Axum 0.8, tokio, rusqlite 0.31 |
| Database | SQLite (logs persistence) |
| External | ADB, Scrcpy (bundled or custom path) |
| Testing | Vitest, cargo test |
| Package Manager | pnpm |

## Quick Start

```bash
# Install dependencies
pnpm install

# Development mode (frontend + Rust backend)
pnpm tauri dev

# Frontend only (Vite, port 1420)
pnpm dev
```

## Build

```bash
# Production build (Windows: .msi + .exe, Linux: .deb + .AppImage, macOS: .dmg + .app)
pnpm tauri build

# Type check
pnpm build && cargo check --manifest-path src-tauri/Cargo.toml
```

## Download

Download installers for your platform from [GitHub Releases](https://github.com/shenjianZ/DeviceDeck/releases).

## License

[MIT](./LICENSE)
