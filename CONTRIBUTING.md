# Contributing to MiniBrowser

Thank you for your interest in contributing to MiniBrowser! We welcome contributions from developers of all skill levels.

---

## 🛠️ Development Setup

MiniBrowser's primary version is written in **Rust** using **Tauri v2**.

### Prerequisites
1. **Rust**: Install via `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. **Node.js**: Node 18+ and `pnpm` (`npm i -g pnpm`)
3. **macOS / Linux / Windows Build Tools**:
   - macOS: Xcode Command Line Tools (`xcode-select --install`)
   - Linux: `libwebkit2gtk-4.1-dev`, `build-essential`, `curl`, `wget`, `file`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`
   - Windows: WebView2 Runtime & Visual Studio C++ Build Tools

### Running Locally
```bash
git clone https://github.com/devCodr/mini-browser.git
cd mini-browser/rust-version
pnpm install
pnpm dev
```

### Packaging & Release
```bash
cd mini-browser/rust-version
pnpm build
```

---

## 🤝 How to Contribute

1. **Fork the Repository** and clone your fork.
2. **Create a Feature Branch**: `git checkout -b feature/my-new-feature`
3. **Make your changes**: Adhere to clean, minimalistic UI patterns and type-safe Rust backend logic.
4. **Test your code**: Run `cargo check --manifest-path src-tauri/Cargo.toml` and `node --check src/main.js`.
5. **Commit your changes**: `git commit -m 'feat: add amazing feature'`
6. **Push to your branch**: `git push origin feature/my-new-feature`
7. **Open a Pull Request** against the `main` branch.

---

## 📜 Code of Conduct

Please adhere to our [Code of Conduct](CODE_OF_CONDUCT.md) in all project interactions.

---

## 📄 License
By contributing to MiniBrowser, you agree that your contributions will be licensed under the [MIT License](LICENSE).
