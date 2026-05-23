# Changelog

本文件记录 DeviceDeck 的版本更新历史。格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)。

## [0.1.3] - 2026-05-23

### Added

- WebSocket 实时文件事件通知，连接客户端在文件创建、删除、清空时即时收到推送
- 文件预览支持，扩展 MIME 类型识别范围
- 分片上传与断点续传能力，附带 SHA-256 校验验证
- 传输历史记录追踪
- 批量下载端点（batch download）
- 文件筛选/排序下拉框自定义组件，替代原生 `<select>`，统一视觉风格并支持中英文本地化

### Changed

- WiFi 传输模块大幅增强（+1761 行），引入 tokio broadcast channel 驱动事件广播

## [0.1.2] - 2026-05-22

### Added

- 项目官方网站（site/），基于 React 19 + TypeScript + Vite 构建，包含首页、功能介绍、截图、下载、架构、FAQ 等完整模块
- WiFi 文件传输独立 Web 页面（devicedeck-wifi-transfer.html），支持验证码认证、文件上传/下载，适配移动端浏览器
- GitHub Actions 自动部署工作流（site.yml），site/ 目录变更时自动构建并部署到 GitHub Pages
- 扩展 `bump-version.mjs`，版本发布时同步更新 site 中的版本号（translations.ts、Download.tsx、Hero.tsx）

### Changed

- 优化所有平台图标文件（icon.ico/icns/png 及 Windows Square 系列），显著减小文件体积
- 更新 GitNexus 索引数据（2069 symbols, 3954 relationships, 171 execution flows）

## [0.1.1] - 2026-05-18

### Added

- 首次启动语言选择欢迎界面（WelcomeOverlay）
- i18n errors 命名空间，后端错误码映射为前端本地化消息
- i18n welcome 命名空间
- `core::data_dir` 模块，统一数据目录管理

### Changed

- 应用数据目录从 Tauri `app_data_dir` 迁移至 `~/.devicedeck/`
- 配置文件从 `app_config.json` 重命名为 `config.json`
- 后端所有硬编码中文消息（日志、错误、操作结果）改为英文
- 前端所有硬编码中文 UI 文本改为 i18n 翻译调用
- Dropdown 组件改用 Portal + fixed 定位，避免在滚动容器内产生滚动条
- `bundled_file_names` 使用 PathBuf 构建路径，确保平台原生分隔符

### Fixed

- 修复 Windows 上工具路径显示混合斜杠（`windows-x64/adb.exe` → `windows-x64\adb.exe`）
- 修复设置页 Tools 区域卡片首次渲染时边框颜色闪烁

## [0.1.0] - 2026-05-14

### Added

#### 核心功能
- Android 设备扫描与管理（USB / WiFi）
- Scrcpy 投屏控制，支持多种预设配置
- 无线调试设备发现、配对与连接
- 设备自动扫描与定时刷新

#### 界面与交互
- 自定义标题栏，支持窗口拖拽、最小化、最大化、关闭、置顶
- Noto Sans SC 字体，支持字号选择（12-16px）
- 全局 Toast 通知系统（成功、错误、警告、信息）
- 左侧导航 + 右侧内容分区设置页面布局

#### 国际化
- i18n 中英文切换支持
- 8 个翻译命名空间：common, settings, topbar, sidebar, mirror, devices, dashboard, logs

#### 系统集成
- 开机自启支持（tauri-plugin-autostart）
- 自动更新支持（tauri-plugin-updater），从 GitHub Release 检查和安装更新
- 关于页面：应用信息、GitHub 链接、更新检查

#### 工程化
- GitHub Actions Release workflow：多平台构建、updater manifest 合并、自动发布
- GitHub Actions PR Check workflow：前端和 Rust 并行检查
- CHANGELOG.md 版本变更日志
