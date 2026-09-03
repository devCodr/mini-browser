# 📖 MiniBrowser Official User Guide

Welcome to **MiniBrowser**, a fast, privacy-centric, minimalist multi-session desktop browser built natively with **Rust and Tauri v2**.

This guide covers all features of the application, including **how to organize and reorder your tabs and sessions**, complete keyboard shortcuts, security settings, and multi-session architecture.

---

## 📑 Table of Contents
1. [Multi-Session Architecture](#-multi-session-architecture)
2. [How to Reorder Tabs & Sessions](#-how-to-reorder-tabs--sessions)
   - [Method 1: Tab Right-Click Context Menu](#method-1-tab-right-click-context-menu)
   - [Method 2: Fast Keyboard Shortcuts](#method-2-fast-keyboard-shortcuts)
   - [Method 3: Smooth Mouse/Trackpad Drag & Drop](#method-3-smooth-mousetrackpad-drag--drop)
   - [Method 4: Session Manager (1-Click Buttons)](#method-4-session-manager-1-click-buttons)
   - [Horizontal Tabs Wheel Scroll](#horizontal-tabs-wheel-scroll)
3. [Complete Keyboard Shortcuts Table](#-complete-keyboard-shortcuts-table)
4. [Managing Accounts & Custom Badges](#-managing-accounts--custom-badges)
5. [PIN Security & Auto-Lock](#-pin-security--auto-lock)
6. [Frequently Asked Questions (FAQ)](#-frequently-asked-questions-faq)

---

## 🧠 Multi-Session Architecture

Unlike standard browsers where cookies are shared across all open windows, **MiniBrowser strictly isolates each account in a dedicated data partition on disk** (`$APP_DATA/sessions/<partition_id>/`).

- **Simultaneous Multiple Accounts**: Run 3 WhatsApp Web accounts, 2 Gmail inboxes, and multiple Facebook profiles concurrently without cookie conflicts or session overrides.
- **Ultra-Lightweight**: Engineered with **Rust** and system-native WebKit (macOS), the release installer is only **4.8 MB** (over 98% smaller than typical 200MB+ Electron browsers) and idles at just ~50 MB RAM.

---

## 🎯 How to Reorder Tabs & Sessions

MiniBrowser offers **4 intuitive methods** to organize and reorder your tabs:

### Method 1: Tab Right-Click Context Menu
1. **Right-click** (or `Ctrl + Click` on macOS) on any tab in the top header bar.
2. A sleek context menu will appear:
   - **Move Left** (`⌥⌘←`): Shifts the tab one position to the left.
   - **Move Right** (`⌥⌘→`): Shifts the tab one position to the right.
   - **Manage Sessions...** (`⌘M`): Opens the Session Management modal.
   - **Close Session** (`⌘W`): Closes the active tab immediately.

---

### Method 2: Fast Keyboard Shortcuts
Reorder your tabs instantly without touching the mouse:
- **`Cmd + Option + ←`** *(Left Arrow)*: Move the active tab one slot to the left.
- **`Cmd + Option + →`** *(Right Arrow)*: Move the active tab one slot to the right.

> **Note**: These shortcuts are registered at the macOS Cocoa system level under the **Tabs** menu bar, guaranteeing they trigger seamlessly even when typing inside an external website.

---

### Method 3: Smooth Mouse/Trackpad Drag & Drop
1. Click and hold on any tab in the top bar.
2. Drag it horizontally left or right.
3. Adjacent tabs will smoothly shift in real-time (*Live-Swap*) to make space.
4. Release the mouse button to lock the tab into its new position, instantly syncing to persistent storage.

---

### Method 4: Session Manager (1-Click Buttons)
For organizing larger lists of accounts and sessions:
1. Open the Session Manager by clicking the **📑** button in the header or pressing **`Cmd + M`**.
2. Each session row provides convenient action controls:
   - **`[ ↑ ]` (Move Up)**: Shifts the session up by one slot with 1 click.
   - **`[ ↓ ]` (Move Down)**: Shifts the session down by one slot with 1 click.
   - **`[ 🚀 ]` (Open)**: Immediately activates and opens that session.
   - **`[ ✏ ]` (Edit)**: Allows renaming the title and changing the badge (e.g. `W1`, `G1`, `Work`).
   - **`[ 🗑 ]` (Delete)**: Removes the session instantly.
3. You can also drag and drop rows vertically. Click **Done** to close.

---

### Horizontal Tabs Wheel Scroll
When numerous tabs exceed the available window width:
- Hover your cursor over the tabs bar and **scroll your mouse wheel** (or two-finger vertical swipe on your trackpad).
- The bar scrolls horizontally with smooth gradient masking at the edges.

---

## ⌨️ Complete Keyboard Shortcuts Table

All shortcuts are available globally across macOS:

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| **`Cmd + 1`** to **`Cmd + 9`** | Switch Tab | Instantly jump to tab #1, #2, #3... up to #9. |
| **`Cmd + Option + ←`** | Move Tab Left | Shift the active tab one position left. |
| **`Cmd + Option + →`** | Move Tab Right | Shift the active tab one position right. |
| **`Cmd + T`** or **`Cmd + N`** | New Session | Open modal to create and launch a new account. |
| **`Cmd + W`** | Close Session | Close and remove the active session. |
| **`Cmd + M`** | Manage Sessions | Open the Session Manager center. |
| **`Cmd + L`** | Focus URL Bar | Focus the address and search bar. |
| **`Cmd + H`** | Dashboard / Home | Return to the welcome presets dashboard. |
| **`Cmd + R`** | Reload Page | Reload the current active webpage. |
| **`Cmd + [`** / **`Cmd + ]`** | Back / Forward | Navigate backwards or forwards in history. |
| **`Cmd + Option + L`** | Lock Browser | Instantly engage the PIN security overlay. |
| **`Cmd + /`** | Help & Guide | Open the interactive Shortcuts & Guide modal. |
| **`Cmd + +`** / **`Cmd + -`** | Zoom In / Out | Scale the page zoom factor. |
| **`Cmd + 0`** | Reset Zoom | Reset page zoom to 100%. |
| **`Esc`** | Dismiss | Close any open modal or context menu. |

---

## 🏷️ Managing Accounts & Custom Badges

Each session can be assigned an identifying **Badge** (up to 4–5 characters) with custom accent colors:
- **`W1`**, **`W2`**, **`W3`**: For differentiating Personal, Work, and Business WhatsApp accounts.
- **`G1`**, **`G2`**: For separate personal and corporate Gmail inboxes.
- **`FB`**, **`X`**, **`NOT`**: For various social profiles and productivity workspaces.

Edit titles and badges anytime via the **Session Manager (`Cmd + M`)** by clicking the pencil icon `[ ✏ ]`.

---

## 🔒 PIN Security & Auto-Lock

MiniBrowser includes privacy protections:

1. **Idle Inactivity Lock**:
   - In **Settings (`⚙️`)**, set an idle timeout (e.g. 1, 5, 15, or 30 minutes). If no mouse or keyboard activity is detected, the browser blurs with a glassmorphic shield and requires your 6-digit PIN.
2. **Lock Immediately on Launch**:
   - Require the PIN before any open sessions are revealed when starting the application.
3. **Start Minimized**:
   - Launch quietly in the background without sudden window popups.
4. **Change PIN**:
   - Change your 6-digit security PIN anytime in Settings (default factory PIN is `123456`).

---

## ❓ Frequently Asked Questions (FAQ)

#### 1. Why did macOS request permission for *"mini-browser WebCrypto Master Key"* in Keychain?
WhatsApp Web and end-to-end encrypted web apps utilize the W3C WebCrypto API. WebKit on macOS securely stores encryption keys in the **System Keychain**. Select **"Always Allow"** and enter your Mac password once so WhatsApp can resume your chats seamlessly.

#### 2. How to log into Gmail without receiving *"This browser or app may not be secure"*?
MiniBrowser presents a genuine Safari 18 User-Agent and strips automation flags. To sign in directly, type `gmail.com` into the URL pill or click the Gmail preset card; it navigates directly to Google's official sign-in endpoint (`accounts.google.com/ServiceLogin`).

#### 3. Where are cookies and session data stored?
All data is stored 100% locally on your machine in:
- **macOS**: `~/Library/Application Support/mini-browser/sessions/` (or `.data/` during local portable development).
- Zero telemetry, zero external servers. Complete privacy.

---

*MiniBrowser • Open Source under the MIT License • Created by Chris Larico ([larico.dev](https://larico.dev))*
