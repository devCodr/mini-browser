# MiniBrowser 🚀

<p align="center">
  <img src="rust-version/src/icon.png" width="96" height="96" alt="MiniBrowser Logo" style="border-radius: 20px; box-shadow: 0 10px 25px rgba(0,0,0,0.3);" />
</p>

<p align="center">
  <b>A blazing-fast, lightweight, multi-session desktop browser for managing multiple accounts simultaneously.</b><br />
  Built natively with Rust & Tauri v2 • Ultra-low memory footprint • Zero cookie collisions
</p>

<p align="center">
  <a href="https://github.com/devCodr/mini-browser/releases"><img src="https://img.shields.io/badge/Release-v1.0.0-6366f1?style=flat-square" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-10b981?style=flat-square" alt="MIT License"></a>
  <a href="https://rust-lang.org"><img src="https://img.shields.io/badge/Built%20With-Rust%20%26%20Tauri%20v2-f59e0b?style=flat-square" alt="Rust & Tauri"></a>
  <a href="https://devcodr.github.io/mini-browser/"><img src="https://img.shields.io/badge/Website-Live%20Demo-3b82f6?style=flat-square" alt="GitHub Pages"></a>
</p>

---

## 🌐 Live Website & Documentation
Explore the interactive documentation and feature showcases:
👉 **[MiniBrowser Official Website & Documentation](https://devcodr.github.io/mini-browser/)**
👉 **[Full In-Depth User Guide (USER_GUIDE.md)](USER_GUIDE.md)**

---

## ✨ Why MiniBrowser?

Traditional web browsers share cookies and session caches globally across tabs. MiniBrowser strictly isolates every account into an independent disk partition (`$APP_DATA/sessions/<partition_id>/`).

- 👥 **Multi-Session Isolation**: Run multiple WhatsApp Web, Gmail, Facebook, and Notion accounts concurrently without interference.
- ⚡ **98% Smaller than Electron**: Native Rust build generates a compact **4.8 MB** installer vs standard 280MB+ Electron bundles.
- 🚀 **Low Memory Footprint**: Idles at ~50 MB RAM with native system WebKit (macOS) and lazy webview instantiation.
- 🎯 **4 Ways to Reorder Tabs**:
  - **Right-click context menu** on any tab (*Move Left / Move Right / Manage / Close*).
  - **Global shortcuts**: `Cmd + Option + ←` / `Cmd + Option + →`.
  - **Fluid drag & drop** with real-time live-swapping.
  - **Session Manager (`Cmd + M`)**: 1-click `[ ↑ ]` and `[ ↓ ]` reordering buttons.
- 📜 **Horizontal Tabs Wheel Scrolling**: Scroll smoothly left and right across limitless tabs with mouse wheel or trackpad.
- 🔒 **PIN Security Shield**: Glassmorphic blur overlay with 6-digit SHA-256 PIN lock, inactivity timer, and quiet launch in minimized mode.
- ⌨️ **Cross-Webview Global Shortcuts**: Native Cocoa application menu accelerators that trigger even while typing inside external websites.

---

## 📂 Repository Structure

```text
mini-browser/
├── src/                  ⚡ Frontend UI (HTML5, Vanilla CSS Glassmorphism, JS)
├── src-tauri/            🦀 Rust backend (Tauri v2, WebKit IPC, multi-session partitions)
├── docs/                 🌐 Official GitHub Pages landing page & documentation
├── USER_GUIDE.md         📖 Comprehensive User Guide & documentation
├── CONTRIBUTING.md       🤝 Guidelines for open-source contributors
├── CODE_OF_CONDUCT.md    📜 Contributor Covenant Code of Conduct
├── LICENSE               📄 MIT License
└── package.json          📦 Project scripts & Tauri CLI
```

---

## 🚀 Quick Start

### Prerequisites
- [Rust](https://rustup.rs/) (1.75+)
- [Node.js](https://nodejs.org/) (v18+) and `pnpm` (`npm install -g pnpm`)

### Development
```bash
git clone https://github.com/devCodr/mini-browser.git
cd mini-browser
pnpm install
pnpm dev
```

### Production Build
```bash
pnpm build
```
Optimized release bundles will be compiled to `src-tauri/target/release/bundle/`:
- **macOS**: Universal & Apple Silicon DMG / App (`~4.8 MB`)
- **Windows**: MSI & EXE via WebView2
- **Linux**: Deb & AppImage via WebKitGTK

---

## ⌨️ Essential Keyboard Shortcuts

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| **`Cmd + 1` .. `Cmd + 9`** | Switch Tab | Direct jump to tab #1 through #9 |
| **`Cmd + Option + ← / →`** | Reorder Tab | Shift active tab left or right |
| **`Cmd + T`** | New Session | Launch a new isolated private account |
| **`Cmd + W`** | Close Session | Close active tab immediately |
| **`Cmd + M`** | Manage Sessions | Open reordering & editing modal |
| **`Cmd + Option + L`** | Lock App | Engage PIN security shield |
| **`Cmd + /`** | Help & Guide | Open interactive shortcuts & manual |

---

## 🤝 Contributing

Contributions are welcome! Please check out [CONTRIBUTING.md](CONTRIBUTING.md) to get started.

---

## 📄 License
MiniBrowser is an open-source project licensed under the [MIT License](LICENSE).
Created with ❤️ by **Chris Larico** ([larico.dev](https://larico.dev)).
