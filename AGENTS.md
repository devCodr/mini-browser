# AGENTS.md — Agent & Developer Workflow Guidelines

This document provides mandatory operational instructions for any AI assistant or human developer making updates or modifications to **MiniBrowser**.

---

## 🚀 Mandatory Rule: Versioning & Release on Every Update

**CRITICAL**: On **EVERY** feature addition, bugfix, or functional modification, the agent **MUST**:
1. **Bump the application version** across all project configuration files.
2. **Update `CHANGELOG.md`** with a new entry describing the changes.
3. **Commit the changes** using conventional commit messages (`feat:`, `fix:`, `chore:`, etc.).
4. **Create an annotated Git tag** matching the new version (e.g., `v1.0.1`, `v1.1.0`).
5. **Compile the release binary and packages** (`pnpm run build`).
6. **Push to remote + push tags** to trigger GitHub Actions CI/CD: `git push && git push --tags`.

---

## 📦 Version Synchronization Locations

The application version **must always be identical** across all three files:

1. `package.json` → `"version": "X.Y.Z"`
2. `src-tauri/tauri.conf.json` → `"version": "X.Y.Z"`
3. `src-tauri/Cargo.toml` → `version = "X.Y.Z"` under `[package]`

> **CRITICAL — About Modal**: The version shown in the About modal (`#about-version-badge` in `index.html`)
> is populated **dynamically** at runtime via `get_app_version` Tauri command (reads `env!("CARGO_PKG_VERSION")`).
> **NEVER hardcode a version string in `index.html`**.
> The `openAboutModal()` function in `main.js` calls `invoke("get_app_version")` which always returns the real compiled version.
> The native menu label "About MiniBrowser vX.Y.Z" is also dynamic in `rebuild_menu()` via `env!("CARGO_PKG_VERSION")`.

---

## 📝 Mandatory: CHANGELOG.md Update on Every Modification

On every change, add an entry to `CHANGELOG.md` following this format:

```markdown
## [X.Y.Z] — YYYY-MM-DD

### Added
- Description of new features

### Fixed
- Description of bug fixes

### Removed
- Description of removed features/fields

### Changed
- Description of modified behavior
```

Use `[Unreleased]` section at top for staging before a version bump.

---

## ⚡ Automated Release Command

```bash
# Patch update (1.0.0 -> 1.0.1):
pnpm run release patch

# Minor feature release (1.0.0 -> 1.1.0):
pnpm run release minor

# Major breaking release (1.0.0 -> 2.0.0):
pnpm run release major

# Exact version:
pnpm run release 1.0.2
```

### What `pnpm run release` does:
1. Updates `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`.
2. Commits with `chore(release): vX.Y.Z`.
3. Creates the annotated Git tag `vX.Y.Z`.
4. Compiles the native production release bundle (`pnpm run build` / `tauri build`).

After running release, push manually:
```bash
git push && git push --tags
```

---

## 🛠️ Manual Release Procedure

1. Edit `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` → new version `X.Y.Z`.
2. Update `CHANGELOG.md` → move `[Unreleased]` entries to `[X.Y.Z] — YYYY-MM-DD`.
3. `git add -A && git commit -m "chore(release): vX.Y.Z"`
4. `git tag -a "vX.Y.Z" -m "Release vX.Y.Z"`
5. `pnpm run build`
6. `git push && git push --tags`  ← triggers `.github/workflows/release.yml`

Generated binaries: `src-tauri/target/release/bundle/` (`.dmg`, `.app`, `.exe`, `.deb`).

---

## 🌐 Mandatory: Documentation & Landing Page Synchronization

On **EVERY** feature addition, UI change, shortcut update, or release, the agent **MUST** keep all documentation and the GitHub Pages landing site fully synchronized:

1. **`README.md`**:
   - Feature highlights (Tray, smart notifications, window drag, native menu, etc.).
   - Keyboard shortcuts table (Preferences `⌘,`, Downloads `⌘⇧J`, etc.).
   - Project structure, architecture notes, and dynamic release badges.

2. **Landing Page (`docs/index.html`)**:
   - Download buttons and version tags must reflect the latest release version.
   - Feature cards & descriptions: System Tray, Smart Notifications, window drag, native menu bar, 4 or 6-digit PIN security, sleep/suspend auto-lock.
   - Documentation chapters: Multi-session architecture, tab workflows, security shield, System Tray & notifications chapter, complete shortcuts reference table, source build instructions.
   - Never retain obsolete or hardcoded hardware claims (e.g. `~50 MB RAM` or `4.8 MB`).
   - Ensure quick nav pills and anchor targets (`#docs-...`) remain aligned.

3. **`CHANGELOG.md`**:
   - Must document every change under the corresponding version header (`Added`, `Fixed`, `Changed`, `Removed`).

---

## 🏗️ Architectural Invariants to Maintain

1. **Multi-Account Storage Isolation (`data_store_identifier`)**:
   - On macOS, WebKit requires `.data_store_identifier(uuid_bytes)` on `WebviewBuilder`.
   - Never remove `data_store_identifier` or deterministic UUID derivation — accounts will leak cookies.

2. **Targeted User-Agent Handling**:
   - **WhatsApp & standard sites**: `CHROME_USER_AGENT` — ensures camera/mic/voice notes work.
   - **Gmail & Google Services**: `APPLE_WEBKIT_USER_AGENT` — prevents "browser not secure" blocks.

3. **Security & PIN Locking**:
   - PIN supports **4 or 6 digits**.
   - `Inactivity PIN Lock` controls only idle timeouts during an active session.
   - **Startup & Suspend**: The app **must always lock** on launch and on system sleep/suspend.

4. **About Modal — Dynamic Version**:
   - `openAboutModal()` in `main.js` calls `invoke("get_app_version")` → updates `#about-version-badge`.
   - `rebuild_menu()` in `lib.rs` uses `env!("CARGO_PKG_VERSION")` for the native menu label.
   - Never hardcode a version string in HTML or JS.

5. **System Tray Icon**:
   - Created in `lib.rs` setup via `tauri::tray::TrayIconBuilder`.
   - Menu: Show MiniBrowser / Lock Browser / separator / Quit MiniBrowser.
   - Left-click → shows and focuses main window + emits `app-focused`.
   - Required Cargo feature: `tray-icon` in `tauri` dependency (`Cargo.toml`).

6. **Notification → Tab Navigation**:
   - Child webview fires `sendHostAction('notify', {...})` → Rust stores `partition` in `AppStateWrapper.pending_notification` and emits `notification-received`.
   - Frontend highlights tab via `highlightNotificationTab(partition)`.
   - On `app-focused` event or after PIN unlock, frontend calls `get_pending_notification` and navigates.

7. **Window Drag Region**:
   - Empty space in the tab bar container uses `data-tauri-drag-region` allowing users to reposition the native window.

8. **Landing & Documentation Sync**:
   - The landing page in `docs/index.html` serves as the live public user manual and download portal on GitHub Pages (`https://devcodr.github.io/mini-browser/`). It must never fall behind the current repository state.

