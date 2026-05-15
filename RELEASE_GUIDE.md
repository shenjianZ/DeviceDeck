---

## 📥 Download Links

| Platform | Architecture | File | Size |
|----------|--------------|------|------|
| **Windows** | x64 | [DeviceDeck-__TAG__-windows-x64.exe](https://github.com/shenjianZ/DeviceDeck/releases/latest/download/DeviceDeck-__TAG__-windows-x64.exe) | ~10 MB |
| **Windows** | x64 | [DeviceDeck-__TAG__-windows-x64.msi](https://github.com/shenjianZ/DeviceDeck/releases/latest/download/DeviceDeck-__TAG__-windows-x64.msi) | ~12 MB |
| **macOS** | Apple Silicon (M1/M2/M3/M4) | [DeviceDeck-__TAG__-macos-aarch64.dmg](https://github.com/shenjianZ/DeviceDeck/releases/latest/download/DeviceDeck-__TAG__-macos-aarch64.dmg) | ~15 MB |
| **macOS** | Intel | [DeviceDeck-__TAG__-macos-x64.dmg](https://github.com/shenjianZ/DeviceDeck/releases/latest/download/DeviceDeck-__TAG__-macos-x64.dmg) | ~15 MB |
| **Linux** | x64 | [DeviceDeck-__TAG__-linux-x64.AppImage](https://github.com/shenjianZ/DeviceDeck/releases/latest/download/DeviceDeck-__TAG__-linux-x64.AppImage) | ~85 MB |
| **Linux** | x64 | [DeviceDeck-__TAG__-linux-x64.deb](https://github.com/shenjianZ/DeviceDeck/releases/latest/download/DeviceDeck-__TAG__-linux-x64.deb) | ~14 MB |
| **Linux** | x64 | [DeviceDeck-__TAG__-linux-x64.rpm](https://github.com/shenjianZ/DeviceDeck/releases/latest/download/DeviceDeck-__TAG__-linux-x64.rpm) | ~14 MB |

---

## 🍎 macOS 安装指南

### 问题 1：提示"已损坏"或"无法打开"

由于 DeviceDeck 未签名，macOS Gatekeeper 会阻止运行。请在终端执行：

```bash
xattr -dr com.apple.quarantine /Applications/DeviceDeck.app
```

### 问题 2："无法验证开发者" 警告

**方法 A：系统设置**
1. 打开 **系统设置** → **隐私与安全性** → **安全性**
2. 找到 DeviceDeck 相关的阻止信息
3. 点击 **"仍要打开"** 按钮

**方法 B：右键打开**
1. 在 Finder 中右键点击 DeviceDeck.app
2. 选择 **"打开"**
3. 在弹窗中点击 **"打开"**

---

## 🪟 Windows 安装指南

### 问题 1：Windows Defender SmartScreen 警告

首次运行时，Windows SmartScreen 可能会显示"Windows 已保护你的电脑"警告：

1. 点击 **"更多信息"**
2. 点击 **"仍要运行"**

### 问题 2：静默安装（高级用户/批量部署）

```powershell
DeviceDeck-__TAG__-windows-x64.exe /S
```

---

## 🐧 Linux 安装指南

### AppImage 使用方法

```bash
# 1. 添加执行权限
chmod +x DeviceDeck-__TAG__-linux-x64.AppImage

# 2. 运行
./DeviceDeck-__TAG__-linux-x64.AppImage
```

### Debian/Ubuntu (.deb)

```bash
sudo dpkg -i DeviceDeck-__TAG__-linux-x64.deb

# 如果遇到依赖问题
sudo apt-get install -f
```

### Fedora/RHEL (.rpm)

```bash
sudo rpm -i DeviceDeck-__TAG__-linux-x64.rpm

# 或使用 dnf 自动处理依赖
sudo dnf localinstall DeviceDeck-__TAG__-linux-x64.rpm
```

---

## 🔗 Resources

- 🐛 [Report Issues](https://github.com/shenjianZ/DeviceDeck/issues)
- 💬 [Discussions](https://github.com/shenjianZ/DeviceDeck/discussions)
