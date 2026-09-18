# MiniBrowser — Changelog

All notable changes to this project will be documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

---

## [1.5.2] — 2026-09-17

### Fixed
- **Lock Screen Bypass via Shortcut (`Cmd+W` / `CmdOrCtrl+W`) & Full Shortcut Shielding**:
  - Resolved security vulnerability where pressing `Cmd+W` while the application was locked triggered the native macOS application menu accelerator for `close_session`, closing the active tab and activating the adjacent session webview directly over the PIN overlay without authentication.
  - **Full Lock Isolation in Rust Core**: Added synchronized `is_locked` state guard inside `AppStateWrapper`. Rust now strictly drops all native menu events (`app.on_menu_event`) when locked, and rejects backend commands (`activate_session`, `add_bookmark`, `remove_bookmark`, and navigation) during locked states.
  - **Keydown Event Capture & Interception**: Upgraded global `keydown` event listener to capture phase (`useCapture: true`). Intercepts and blocks all keyboard shortcuts (such as `Cmd+W`, `Cmd+T`, `Cmd+R`, `Cmd+1..9`, `Cmd+,`, `Cmd+M`) and default browser actions while locked. Only digit PIN entry (`0-9`), `Backspace`, `Escape`, `Enter`, and standard OS quit (`Cmd+Q`) are permitted.
  - **Frontend UI & Listener Guards**: Added strict `state.isLocked` checks across `activateSession`, `removeSession`, `createSession`, `goHome`, `openNewSessionModal`, `openManageSessionsModal`, `toggleShortcutsModal`, `openAboutModal`, Settings button, `menu-shortcut`, and child webview forwarded shortcuts (`trigger-shortcut`).

---

## [1.5.1] — 2026-09-17

### Fixed
- **Tab Context Menu Z-Index & Native View Clipping**: Resolved issue where the tab bar right-click context menu was partially hidden/cut off behind native child `WKWebView` instances on macOS. MiniBrowser now dynamically hides the active webview while the tab menu is open, ensuring full foreground visibility, and automatically restores it upon dismissal.
- **Strict English-Only Interface**: Purged remaining Spanish localization strings across all menus, notifications, file fallbacks, and tooltips (e.g., `'Abrir carpeta Descargas'` → `'Open Downloads Folder'`, `'Descarga completada'` → `'Download Completed'`, `'Descargar archivo'` → `'Download File'`).

### Added
- **Webview In-Page Context Menu Actions**: Added tab management options directly into the right-click menu inside web pages (WhatsApp Web, Facebook, Gmail, etc.):
  - **`Sleep This Tab 💤`**: Immediately puts current tab into standby and switches to an adjacent tab.
  - **`Toggle Keep Tab Awake ⚡`**: Exclude or include the current tab from automatic hibernation.
  - **`Manage Sessions... (Cmd+M)`**: Opens the session management modal.
  - **`Close Tab (Cmd+W)`**: Closes the current session directly from inside the page.

---

## [1.5.0] — 2026-09-17

### Added
- **Memory Saver & Intelligent Tab Hibernation (Standby System)**:
  - Eliminates excessive RAM consumption (800MB–1.4GB per tab on heavy web applications like WhatsApp Web, Facebook, and Gmail) caused by multiple concurrent WebKit processes on macOS.
  - **Automatic tab standby**: Background tabs inactive for longer than the configured threshold (default 15 minutes) are put into standby by cleanly terminating their child Webview, instantly reclaiming 100% of their RAM memory.
  - **Seamless 1-click wake-up**: Clicking any hibernated tab immediately re-creates the WebKit instance with the exact partition, preserved login session (`data_store_identifier`), and navigated URL.
  - **Manual tab sleep**: Right-click context menu option *"Sleep Tab Now 💤"* to immediately free memory for specific tabs.
  - **Selective Wake Lock ("Keep Tab Awake ⚡")**: Right-click context menu toggle allows users to exclude critical communication tabs (such as WhatsApp Web or primary email) from ever sleeping, ensuring uninterrupted live notifications and active WebSockets.
  - **Tab bar visual indicators**: Sleeping tabs display a subtle dashed border and `💤` badge; protected tabs display an active `⚡` wake lock badge.
  - **Configurable Settings**: Added Memory Saver toggle and customizable sleep timeout (5 min, 10 min, 15 min, 30 min, 1 hour) in Browser Settings (`⌘,`).
  - **Navigation URL persistence**: In-memory tab URLs are automatically synced on navigation so waking tabs always restore to the exact active page.

---

## [1.4.0] — 2026-09-15

### Added
- **Session Inactivity Auto-Lock Bypass ("Keep unlocked for this session")**:
  - Ability to temporarily pause the inactivity lock for the current working session without modifying persistent settings stored on disk.
  - **Lock screen integration**: Added *"Keep unlocked for this session"* checkbox directly below PIN keypad on the lock screen overlay.
  - **Header status badge**: Dynamic amber pill (`Session Unlocked`) displayed next to the lock button when auto-lock is paused; 1-click resumes normal auto-lock.
  - **macOS App Menu integration**: Added `MiniBrowser > Pause Inactivity Lock (This Session)` / `Resume Inactivity Lock` with native accelerator (`CmdOrCtrl+Alt+P`).
  - **System Tray integration**: Added tray menu item to toggle session auto-lock pause directly from macOS menu bar / Windows tray.
  - **Settings modal integration**: Added session auto-lock status indicator and quick pause/resume action under the Inactivity PIN Lock section.
  - **Global shortcut**: `Cmd + Alt + P` (macOS) / `Ctrl + Alt + P` (Windows/Linux) to instantly toggle session auto-lock pause.
  - **Safety guarantees**: Auto-lock pause is kept purely in-memory; restarting the application, system sleep/suspend, or manual lock (`Cmd+Alt+L`) safely enforces standard security policies.

---

## [1.3.1] — 2026-09-11

### Fixed
- **macOS Launch at Login**: Migrated auto-launch mechanism to native `AppleScript` Login Items (`MacOSLaunchMode::AppleScript`). MiniBrowser now registers properly under macOS System Settings **"Open at Login"** (alongside apps like Maccy, Espanso, Rectangle Pro) instead of failing under "App Background Activity".
- **Launchd failure (exit code 78)**: Resolved bug where LaunchAgent plist attempted to execute the `.app` bundle directory directly, which caused `launchd` to fail on boot and prevented the app, Tray Icon, and Dock active dot from appearing.
- **Legacy LaunchAgent cleanup**: Automatically unloads and purges legacy `~/Library/LaunchAgents/MiniBrowser.plist` on startup and settings update.
- **macOS Close-to-Tray behavior**: Closing the main window on macOS now hides the window (`api.prevent_close()`) instead of killing the app process, keeping the System Tray icon and background sessions alive.
- **macOS Dock click reopen**: Clicking the Dock icon when the window is hidden or minimized reopens, unminimizes, and focuses the main window (`RunEvent::Reopen`).

---

## [1.3.0] — 2026-09-10

### Added
- **About modal — dynamic version**: Version badge now reads from `Cargo.toml` via `get_app_version` command. Never shows stale hardcoded version again.
- **Native macOS menu — complete restructure**: App menu now follows macOS HIG conventions:
  - `MiniBrowser > About MiniBrowser vX.Y.Z` — opens About modal
  - `MiniBrowser > Settings / Preferences... (⌘,)` — opens Settings modal
  - `View > Open Downloads Folder` — opens system Downloads folder
- **Menu-shortcut handlers**: All native menu actions wire to frontend (`about_minibrowser`, `preferences`, `open_downloads`).
- **Landing Page & Documentation (`docs/`)**: Updated GitHub Pages landing site with System Tray, Smart Notifications, window drag, native menu shortcuts (`⌘,`), and dynamic GitHub release tag fetcher.
- **AGENTS.md workflow rules**: Added mandatory rule to synchronize `README.md` and `docs/index.html` on every modification and release.

### Removed
- **About modal — static stats removed**: Removed hardcoded "Memory Footprint ~50 MB RAM (Idle)" and "Bundle Size ~4.8 MB (Ultra-compact)" misleading fields from About modal and documentation.

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
