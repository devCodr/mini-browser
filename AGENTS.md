# AGENTS.md — Agent & Developer Workflow Guidelines

This document provides mandatory operational instructions for any AI assistant or human developer making updates or modifications to **MiniBrowser**.

---

## 🚀 Mandatory Rule: Versioning & Release on Every Update

**CRITICAL**: On **EVERY** feature addition, bugfix, or functional modification, the agent **MUST**:
1. **Bump the application version** across all project configuration files.
2. **Commit the changes** using conventional commit messages (`feat:`, `fix:`, `chore:`, etc.).
3. **Create an annotated Git tag** matching the new version (e.g., `v1.0.1`, `v1.1.0`).
4. **Compile the release binary and packages** (`pnpm run build`).

---

## 📦 Version Synchronization Locations

The application version **must always be identical** across all three files:

1. [package.json](file:///Volumes/SSD/larico/mini-browser/package.json):
   ```json
   "version": "1.0.1"
   ```
2. [src-tauri/tauri.conf.json](file:///Volumes/SSD/larico/mini-browser/src-tauri/tauri.conf.json):
   ```json
   "version": "1.0.1"
   ```
3. [src-tauri/Cargo.toml](file:///Volumes/SSD/larico/mini-browser/src-tauri/Cargo.toml):
   ```toml
   [package]
   version = "1.0.1"
   ```

---

## ⚡ Automated Release Command

To automate all steps in one single command, use:

```bash
# For patch updates (1.0.0 -> 1.0.1):
pnpm run release patch

# For minor feature releases (1.0.0 -> 1.1.0):
pnpm run release minor

# For major breaking releases (1.0.0 -> 2.0.0):
pnpm run release major

# Or specify an exact version:
pnpm run release 1.0.2
```

### What `pnpm run release` does:
1. Updates `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml` with the new version.
2. Stages all modified files and commits them with `chore(release): vX.Y.Z`.
3. Creates the annotated Git tag `vX.Y.Z`.
4. Compiles the native production release bundle (`pnpm run build` / `tauri build`).

---

## 🛠️ Manual Release Procedure

If performing steps manually instead of using `pnpm run release`:

1. **Update Versions**:
   Edit `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml` to the new version `X.Y.Z`.

2. **Commit Changes**:
   ```bash
   git add -A
   git commit -m "chore(release): vX.Y.Z"
   ```

3. **Create Git Tag**:
   ```bash
   git tag -a "vX.Y.Z" -m "Release vX.Y.Z"
   ```

4. **Compile Release Build**:
   ```bash
   pnpm run build
   ```
   *Generated binaries will be placed in `src-tauri/target/release/bundle/` (`.dmg`, `.app`, `.exe`, `.deb`, etc.).*

5. **Push to Remote (Triggers GitHub Actions CI/CD)**:
   ```bash
   git push && git push --tags
   ```
   *This triggers `.github/workflows/release.yml`, building and uploading universal releases for macOS, Windows, and Linux.*

---

## 🏗️ Architectural Invariants to Maintain

When modifying the codebase, preserve these critical architectural guarantees:

1. **Multi-Account Storage Isolation (`data_store_identifier`)**:
   - On macOS, WebKit requires `.data_store_identifier(uuid_bytes)` on `WebviewBuilder` to allocate separate `WKWebsiteDataStore` instances.
   - Never remove `data_store_identifier` or deterministic UUID derivation, or accounts (`W1`, `W2`, etc.) will leak cookies and sessions.

2. **Targeted User-Agent Handling**:
   - **WhatsApp & standard sites**: Use Google Chrome User-Agent (`CHROME_USER_AGENT`) to ensure camera, microphone, voice notes, and WhatsApp Web compatibility without browser warnings.
   - **Gmail & Google Services**: Use Apple WebKit User-Agent (`APPLE_WEBKIT_USER_AGENT`) to prevent embedded webview blocks ("This browser or app may not be secure").

3. **Security & PIN Locking**:
   - The PIN supports **4 or 6 digits**.
   - `Inactivity PIN Lock` only controls idle timeouts during an active session.
   - **Startup & Suspend**: The application **must always lock** on launch/reboot and upon system sleep/suspend.
