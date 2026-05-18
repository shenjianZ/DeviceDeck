# Changelog

本文件记录 DeviceDeck 的版本更新历史。格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)。

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
