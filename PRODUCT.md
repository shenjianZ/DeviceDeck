# DeviceDeck

> 现代化的 Android 屏幕镜像与设备管理桌面工作台

---

## 产品简介

DeviceDeck 是一款基于 Tauri 2 构建的高性能桌面应用程序，专为 Android 设备管理和屏幕镜像而设计。它将 ADB 和 Scrcpy 的强大功能封装在一个现代化、用户友好的界面中，为开发者、测试人员和设备管理人员提供一站式设备管理解决方案。

---

## 核心特性

### 🔌 多连接方式
- **USB 有线连接**：即插即用，稳定可靠
- **WiFi 无线连接**：摆脱线缆束缚，支持 Android 11+ 无线调试
- **自动设备发现**：智能扫描局域网内的无线调试设备
- **设备配对**：支持 Android 11+ 无线调试配对码配对

### 📱 设备管理
- **实时设备列表**：自动检测已连接的 Android 设备
- **设备状态监控**：在线/离线/未授权状态实时显示
- **详细设备信息**：型号、品牌、Android 版本、屏幕尺寸、电量等
- **能力标签**：快速识别设备支持的功能特性

### 🖥️ 屏幕镜像
- **高清投屏**：基于 Scrcpy 的低延迟屏幕镜像
- **多分辨率支持**：从 480p 到 1080p+ 灵活选择
- **可调帧率**：15/30/60 fps 自由切换
- **视频编码选择**：H.264 / H.265 / AV1 编码支持
- **码率控制**：1Mbps 到 16Mbps+ 多档可选

### ⚙️ 投屏预设
- **性能模式**：低资源占用，适合日常使用
- **均衡模式**：画质与性能的最佳平衡
- **质量模式**：高画质，适合演示展示
- **极致模式**：最高画质，适合专业录制

### 🎮 投屏控制
- **只读模式**：禁用鼠标键盘控制，安全观察
- **保持唤醒**：投屏时阻止设备自动息屏
- **关闭屏幕**：投屏时关闭手机屏幕，节省电量

### 📊 实时日志
- **多来源日志**：系统、ADB、Scrcpy 分类记录
- **日志过滤**：按来源、级别快速筛选
- **自动清理**：可配置日志保留天数

### 🔧 环境检测
- **ADB 状态**：自动检测 ADB 工具可用性和版本
- **Scrcpy 状态**：自动检测 Scrcpy 工具可用性和版本
- **路径配置**：支持使用内置工具或自定义路径

---

## 应用界面

### 仪表盘
- 环境状态一目了然
- 设备数量和投屏会话概览
- 快捷操作入口
- 最近日志快速查看

### 设备管理
- 卡片式设备列表
- 设备详情侧边栏
- 一键进入投屏
- USB 转 WiFi 快捷操作

### 投屏控制
- 直观的参数配置面板
- 多种连接方式分区展示
- 活动会话实时管理
- 无线设备自动发现

### 运行日志
- 表格式日志展示
- 多维度过滤筛选
- 时间戳精确记录
- 一键清空功能

### 设置中心
- 工具路径配置
- 自动扫描设置
- 默认投屏参数
- 日志保留策略

---

## 技术架构

### 前端技术栈
- **React 19**：现代化 UI 框架
- **TypeScript**：类型安全的开发体验
- **TailwindCSS 4**：原子化 CSS 样式方案
- **Zustand**：轻量级状态管理
- **Lucide React**：精美图标库

### 后端技术栈
- **Rust**：高性能系统编程语言
- **Tauri 2**：跨平台桌面应用框架
- **SQLite**：轻量级嵌入式数据库
- **Tokio**：异步运行时

### 核心依赖
- **ADB (Android Debug Bridge)**：Android 设备调试工具
- **Scrcpy**：开源屏幕镜像工具

---

## 系统要求

### 操作系统
- Windows 10/11 (64-bit)
- macOS 10.15+ (计划支持)
- Linux (计划支持)

### 运行环境
- ADB 工具（可选，应用内置）
- Scrcpy 工具（可选，应用内置）

### 设备要求
- Android 5.0+ (API 21+)
- USB 调试已启用
- Android 11+ 支持无线调试

---

## 使用场景

### 开发者
- 实时查看应用运行效果
- 快速调试 UI 问题
- 多设备同时管理
- 无需频繁拿起手机

### 测试人员
- 批量设备测试
- 录制测试过程
- 远程设备操作
- 测试结果展示

### 设备管理人员
- 设备状态监控
- 批量设备管理
- 远程设备演示
- 设备信息采集

### 内容创作者
- 高清屏幕录制
- 游戏直播
- 教程制作
- 产品演示

---

## 产品优势

### 🚀 高性能
- Rust 后端保证极致性能
- 低延迟屏幕镜像
- 资源占用极低

### 🎨 现代化 UI
- 简洁美观的界面设计
- 流畅的交互动画
- 响应式布局

### 🔒 安全可靠
- 本地运行，数据不上传
- 开源透明
- 稳定可靠

### 🛠️ 易于使用
- 开箱即用
- 直观的操作界面
- 丰富的配置选项

### 📦 轻量级
- 小体积安装包
- 无需额外依赖
- 快速启动

---

## 下载安装

选择适合您操作系统的版本下载：

<div align="center">

| 平台 | 下载链接 | 文件格式 | 说明 |
|:---:|:---:|:---:|:---|
| **Windows** | [⬇️ 下载 Windows 版](https://github.com/yourusername/devicedeck/releases/latest/download/devicedeck_x64-setup.exe) | `.exe` | Windows 10/11 64-bit，推荐使用 |
| **macOS** | [⬇️ 下载 macOS 版](https://github.com/yourusername/devicedeck/releases/latest/download/devicedeck_x64.dmg) | `.dmg` | macOS 10.15+，支持 Intel/Apple Silicon |
| **Linux** | [⬇️ 下载 Linux 版](https://github.com/yourusername/devicedeck/releases/latest/download/devicedeck_amd64.AppImage) | `.AppImage` | Ubuntu 20.04+ / Debian 10+ / Fedora 35+ |

</div>

> 📌 所有版本均为免费开源，访问 [GitHub Releases](https://github.com/yourusername/devicedeck/releases) 查看所有版本和更新日志。

### 安装说明

#### Windows
1. 下载 `.exe` 安装包
2. 双击运行安装程序
3. 按照向导完成安装
4. 从开始菜单或桌面快捷方式启动

#### macOS
1. 下载 `.dmg` 文件
2. 双击打开磁盘映像
3. 将 DeviceDeck 拖动到 Applications 文件夹
4. 首次打开可能需要在"系统偏好设置 > 安全性与隐私"中允许运行

#### Linux
1. 下载 `.AppImage` 文件
2. 添加执行权限：`chmod +x devicedeck_amd64.AppImage`
3. 双击运行或在终端中执行：`./devicedeck_amd64.AppImage`

---

## 快速开始

### 1. 启动应用
安装完成后，启动 DeviceDeck 应用程序。

### 2. 连接设备
- **USB 连接**：使用数据线连接手机和电脑
- **WiFi 连接**：在手机上启用无线调试

### 3. 扫描设备
点击"扫描设备"按钮，自动检测已连接的设备。

### 4. 开始投屏
选择设备，配置投屏参数，点击"开始投屏"。

---

## 版本信息

- **当前版本**：0.1.0
- **开发状态**：Alpha
- **许可证**：MIT

---

## 联系方式

- **GitHub**：https://github.com/yourusername/devicedeck
- **问题反馈**：https://github.com/yourusername/devicedeck/issues
- **邮箱**：devicedeck@example.com

---

## 致谢

DeviceDeck 基于以下优秀的开源项目构建：

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [Scrcpy](https://github.com/Genymobile/scrcpy) - 屏幕镜像工具
- [ADB](https://developer.android.com/tools/adb) - Android 调试桥
- [React](https://react.dev/) - UI 框架
- [TailwindCSS](https://tailwindcss.com/) - CSS 框架

---

*DeviceDeck - 让 Android 设备管理更简单*
