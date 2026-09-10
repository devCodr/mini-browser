# MiniBrowser — Changelog

All notable changes to this project will be documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

---

## [1.3.0] — 2026-09-10

### Added
- **About modal — dynamic version**: Version badge now reads from `Cargo.toml` via `get_app_version` command. Never shows stale hardcoded version again.
- **Native macOS menu — complete restructure**: App menu now follows macOS HIG conventions:
  - `MiniBrowser > About MiniBrowser vX.Y.Z` — opens About modal
  - `MiniBrowser > Settings / Preferences... (⌘,)` — opens Settings modal
  - `View > Open Downloads Folder` — opens system Downloads folder
- **Menu-shortcut handlers**: All native menu actions wire to frontend (`about_minibrowser`, `preferences`, `open_downloads`).

### Removed
- **About modal — static stats removed**: Removed hardcoded "Memory Footprint ~50 MB RAM (Idle)" and "Bundle Size ~4.8 MB (Ultra-compact)" misleading fields.

---

## [1.2.0] — 2026-09-10

### Added
- **System Tray Icon**: MiniBrowser now shows an icon in the macOS menu bar at all times. Left-click → shows/focuses window. Right-click → Show / Lock / Quit.
- **Notification → Tab Navigation**: Clicking a system notification automatically navigates to the originating tab (WhatsApp W1/W2, Gmail, etc.). Works even after PIN lock.
- **Tab notification pulse**: Background tabs that receive a notification show an indigo glow pulse animation.
- **Window drag from empty tab bar area**: Previously blocked by over-broad CSS selector; now the empty space in the tabs bar allows window dragging.
- `get_pending_notification` Tauri command.
- `app-focused` Tauri event for navigation on window focus.

### Fixed
- `WindowEvent::Focused` works on all platforms (not just macOS) to emit `app-focused`.
- `unlockApp()` now navigates to pending notification tab after PIN unlock.

---

## [1.1.0] — 2026-09-09

### Added
- Multi-account WebKit data store isolation via `data_store_identifier`.
- Targeted User-Agent: Chrome UA for WhatsApp; Apple WebKit UA for Gmail/Google.
- PIN Lock: 4 or 6 digits, lock on launch, inactivity timer, system-sleep lock.
- Security question & answer recovery flow for forgotten PIN.
- Factory Reset.
- Session tabs: add, remove, reorder (drag & drop), badge & color per tab.
- Native macOS app menu with tab switching shortcuts (⌘1–⌘9).
- Download manager with toast notifications.
- Keyboard shortcuts panel.
- Auto-launch on system startup.
- Start minimized option.
- Custom favicon/icon per tab (SVG, image URL, data URI).
- Zoom control per session.
- Context menu: open in browser, open in new session, copy/paste.

---

## [1.0.0] — 2026-09-01

### Added
- Initial release of MiniBrowser (Rust & Tauri v2).
- Multi-session browser with isolated WebKit data stores.
- PIN lock overlay with inactivity timeout.
- Bookmark/session persistence across restarts.
