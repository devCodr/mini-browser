# MiniBrowser 🚀

<p align="center">
  <img src="docs/icon.png" width="96" height="96" alt="MiniBrowser Logo" style="border-radius: 20px; box-shadow: 0 10px 25px rgba(0,0,0,0.3);" />
</p>

<p align="center">
  <b>A blazing-fast, lightweight, open-source multi-session desktop browser for managing multiple accounts simultaneously.</b><br />
  Built natively with Rust & Tauri v2 • Zero cookie collisions • System Tray • Smart notification routing
</p>

<p align="center">
  <a href="https://github.com/devCodr/mini-browser/releases"><img src="https://img.shields.io/github/v/release/devCodr/mini-browser?style=flat-square&color=6366f1&label=Release" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-10b981?style=flat-square" alt="MIT License"></a>
  <a href="https://rust-lang.org"><img src="https://img.shields.io/badge/Built%20With-Rust%20%26%20Tauri%20v2-f59e0b?style=flat-square" alt="Rust & Tauri"></a>
  <a href="https://github.com/devCodr/mini-browser/releases"><img src="https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-blue?style=flat-square" alt="Platforms"></a>
  <a href="https://devcodr.github.io/mini-browser/"><img src="https://img.shields.io/badge/Website-Live-3b82f6?style=flat-square" alt="GitHub Pages"></a>
</p>

---

## 🌐 Live Website & Documentation
- 👉 **[Official Website & Documentation](https://devcodr.github.io/mini-browser/)**
- 👉 **[Comprehensive User Guide](https://devcodr.github.io/mini-browser/#docs)**
- 👉 **[Changelog](CHANGELOG.md)**
- 👉 **[Contribution Guidelines](CONTRIBUTING.md)**

---

## ✨ Why MiniBrowser?

Standard web browsers share cookies and session caches across all tabs and windows. MiniBrowser isolates every account into its own independent disk partition (`$APP_DATA/sessions/<partition_id>/`), preventing cookie collisions and account sign-outs.

- 👥 **Multi-Session Isolation**: Run multiple WhatsApp Web, Gmail, Facebook, X (Twitter), and Notion accounts concurrently without interference.
- ⚡ **Cross-Platform & Ultra-Compact**: Fully supported on **macOS**, **Windows**, and **Linux**.
- 🖥️ **Adaptive Window Controls**:
  - **macOS**: Seamless traffic lights integrated directly into the glassmorphic header.
  - **Windows & Linux**: Custom frameless title bar with native-style Minimize, Maximize/Restore, and Close controls.
- 📌 **System Tray Icon**: MiniBrowser lives in your macOS menu bar. Click the tray icon to show/hide the window even when running in the background. Right-click for Show / Lock / Quit.
- 🔔 **Smart Notification Routing**: When a notification arrives from WhatsApp W1, WhatsApp W2, Gmail, etc., clicking it automatically navigates to the exact tab that sent it — even after a PIN lock.
- 🎯 **4 Flexible Ways to Reorder Tabs**:
  - **Right-click context menu** on any tab (*Move Left / Move Right / Manage / Close*).
  - **Global shortcuts**: `Cmd + Option + ← / →` (macOS) or `Ctrl + Alt + ← / →` (Windows/Linux).
  - **Fluid drag & drop** with live visual reordering.
  - **Session Manager (`Cmd + M` / `Ctrl + M`)**: 1-click `[ ↑ ]` and `[ ↓ ]` reordering buttons.
- 📜 **Horizontal Tabs Wheel Scrolling**: Scroll through dozens of tabs with your trackpad or mouse wheel.
- 🪟 **Window Drag from Tab Bar**: Drag the window from the empty space in the tab bar — anywhere that isn't a tab or button.
- 🔒 **PIN Security Shield**: Glassmorphic blur overlay with 4 or 6-digit PIN lock, inactivity auto-lock timer, temporary session-unlock mode ("Keep unlocked for this session"), security question recovery, and quiet launch in minimized mode.
- 💤 **Memory Saver & Tab Standby (Hibernation)**: Frees 100% of RAM consumed by heavy background tabs (WhatsApp Web, Facebook, Gmail) by placing inactive tabs into standby, with 1-click wake lock (`⚡ Keep Tab Awake`) to prevent sleep on critical communication accounts.
- ⌨️ **Native Menu Bar & Global Accelerators**: Complete macOS app menu with tab switching shortcuts (`Cmd+1`–`Cmd+9`), Settings (`Cmd+,`), and About — all functioning even while focused inside external websites.
- 🎨 **Custom Sessions & Badges**: Assign unique labels, colored badges, and custom icons to each account.

---

## 📂 Repository Structure

```text
mini-browser/
├── docs/                 🌐 GitHub Pages website & interactive documentation center (#docs)
├── src/                  ⚡ Frontend UI (HTML5, Vanilla CSS Glassmorphism, JS)
├── src-tauri/            🦀 Rust backend (Tauri v2, system webviews, multi-session partitions)
├── .github/workflows/    🤖 Multi-platform build & release workflows (macOS, Windows, Linux)
├── AGENTS.md             🤖 Mandatory workflow guidelines for AI agents & developers
├── CHANGELOG.md          📋 Version history and release notes
├── CONTRIBUTING.md       🤝 Guidelines for open-source contributors
├── CODE_OF_CONDUCT.md    📜 Contributor Covenant Code of Conduct
├── LICENSE               📄 MIT License
└── package.json          📦 Scripts & Tauri dependencies
```

---

## 🚀 Quick Start

### Prerequisites
- [Rust](https://rustup.rs/) (1.75+)
- [Node.js](https://nodejs.org/) (v18+) and [pnpm](https://pnpm.io/) (`npm install -g pnpm`)

#### Platform Requirements
- **macOS**: Xcode Command Line Tools (`xcode-select --install`).
- **Windows**: [C++ Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and Microsoft Edge WebView2 (installed by default on Windows 10/11).
- **Linux (Ubuntu / Debian)**:
  ```bash
  sudo apt update && sudo apt install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libsoup-3.0-dev libjavascriptcoregtk-4.1-dev build-essential
  ```

### Development
Clone the repository and start the development environment:

```bash
git clone https://github.com/devCodr/mini-browser.git
cd mini-browser
pnpm install
pnpm dev
```

### Production Build
Compile optimized native release binaries:

```bash
pnpm build
```

The compiled release packages will be created in `src-tauri/target/release/bundle/`:
- **macOS**: Apple Silicon `.dmg` / `.app`
- **Windows**: `.msi` and `.exe` installers via WebView2
- **Linux**: `.deb` and `.AppImage` packages via WebKitGTK

> [!TIP]
> **macOS Installation Note**: If macOS displays `"MiniBrowser.app" is damaged and can't be opened`, run this in Terminal to unlock it:
> ```bash
> xattr -cr /Applications/MiniBrowser.app
> ```

---

## ⌨️ Essential Keyboard Shortcuts

| Shortcut (macOS) | Shortcut (Windows/Linux) | Action | Description |
| :--- | :--- | :--- | :--- |
| **`Cmd + 1` .. `Cmd + 9`** | **`Ctrl + 1` .. `Ctrl + 9`** | Switch Tab | Direct jump to tab #1 through #9 |
| **`Cmd + Option + ← / →`** | **`Ctrl + Alt + ← / →`** | Reorder Tab | Shift active tab left or right |
| **`Cmd + T`** | **`Ctrl + T`** | New Session | Launch a new isolated private session |
| **`Cmd + W`** | **`Ctrl + W`** | Close Session | Close the active tab immediately |
| **`Cmd + M`** | **`Ctrl + M`** | Manage Sessions | Open reordering, editing & badge modal |
| **`Cmd + ,`** | **`Ctrl + ,`** | Settings | Open browser settings & preferences |
| **`Cmd + R`** | **`Ctrl + R`** | Reload | Reload active webview partition |
| **`Cmd + H`** | **`Ctrl + H`** | Home | Return to dashboard home |
| **`Cmd + L`** | **`Ctrl + L`** | Address Bar | Focus address bar for rapid navigation |
| **`Cmd + Option + L`** | **`Ctrl + Alt + L`** | Lock Browser | Instantly engage PIN security shield |
| **`Cmd + Option + P`** | **`Ctrl + Alt + P`** | Pause Auto-Lock | Temporarily bypass inactivity lock for current session |
| **`Cmd + /`** | **`Ctrl + /`** | Help & Guide | Open interactive shortcuts & user manual |

---

## 🔒 Security & Privacy

- **Zero Data Harvesting**: MiniBrowser communicates strictly between your machine and the sites you visit. No telemetry, no third-party tracking.
- **Dedicated Disk Partitions**: Each session stores cache, cookies, and local data under `$APP_DATA/com.larico.minibrowser/sessions/<partition_id>/`.
- **PIN Shield**: Set a 4 or 6-digit PIN to prevent unauthorized local access with configurable auto-lock on inactivity, one-click session bypass ("Keep unlocked for this session"), security question recovery, and mandatory lock on launch & system sleep.

---

## 🤝 Contributing

Contributions are welcome! Please check out [CONTRIBUTING.md](CONTRIBUTING.md) for details on code style, commit standards, and pull request procedures.

---

## 📄 License
MiniBrowser is an open-source project licensed under the [MIT License](LICENSE).  
Created with ❤️ by **Chris Larico** ([larico.dev](https://larico.dev)).
