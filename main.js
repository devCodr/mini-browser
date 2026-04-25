import {
  app,
  BrowserWindow,
  ipcMain,
  session,
  Menu,
  shell,
  dialog,
} from "electron";

// Add command line switches for SSL bypass (development only)
app.commandLine.appendSwitch('ignore-certificate-errors');
app.commandLine.appendSwitch('ignore-ssl-errors');
app.commandLine.appendSwitch('ignore-certificate-errors-spki-list');
app.commandLine.appendSwitch('ignore-certificate-errors-spki-list');

import path, { dirname } from "path";
import fs from "fs";
import crypto from "crypto";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const INACTIVITY_DEFAULT_MS = 5 * 60 * 1000;
let mainWindow;
let inactivityTimer = null;
let settings = null;
let bookmarks = [];
let isLocked = false;

const userDataDir = app.getPath("userData");
const settingsPath = path.join(userDataDir, "settings.json");
const bookmarksPath = path.join(userDataDir, "bookmarks.json");

function readJson(p, fallback) {
  try {
    if (!fs.existsSync(p)) return fallback;
    const raw = fs.readFileSync(p, "utf-8");
    return JSON.parse(raw);
  } catch {
    return fallback;
  }
}
function writeJson(p, data) {
  try {
    fs.writeFileSync(p, JSON.stringify(data, null, 2));
  } catch {}
}
function loadAllState() {
  settings = readJson(settingsPath, {
    lockEnabled: true,
    inactivityMs: INACTIVITY_DEFAULT_MS,
    pinSalt: null,
    pinHash: null,
  });

  settings.lockEnabled = true;

  if (!settings.pinHash || !settings.pinSalt) {
    setPin("123456");
  }

  bookmarks = readJson(bookmarksPath, []);
  bookmarks = bookmarks.filter(
    (b) =>
      b && typeof b === "object" && b.partition && /^persist:/.test(b.partition)
  );
}
function saveSettings() {
  writeJson(settingsPath, settings);
}
function saveBookmarks() {
  writeJson(bookmarksPath, bookmarks);
}

function sha256(s) {
  return crypto.createHash("sha256").update(s).digest("hex");
}
function setPin(pin) {
  const salt = crypto.randomBytes(16).toString("hex");
  settings.pinSalt = salt;
  settings.pinHash = sha256(pin + ":" + salt);
  saveSettings();
}
function verifyPin(pin) {
  if (!settings.pinHash || !settings.pinSalt) return false;
  const h = sha256(pin + ":" + settings.pinSalt);
  return h === settings.pinHash;
}

function createMainMenu() {
  const template = [
    {
      label: "MiniBrowser",
      submenu: [
        {
          label: "About",
          click: () => {
            dialog.showMessageBox({
              type: "info",
              title: "About MiniBrowser",
              message:
                `MiniBrowser v${app.getVersion()}\n\n` +
                "Proyecto creado con Electron\n" +
                "Crafted by: Chris Larico\n" +
                "Website: https://larico.dev\n\n" +
                "© 2025 MIT License",
              icon: path.join(__dirname, "icon.png"),
              buttons: ["Close"],
            });
          },
        },

        { type: "separator" },
        { label: "Lock now", click: () => showOverlayLock() },
        {
          label: settings.lockEnabled ? "Disable Lock" : "Enable Lock",
          click: () => toggleLockEnabled(),
        },
        { type: "separator" },
        { role: "quit" },
      ],
    },
    {
      label: "Edit",
      submenu: [
        { role: "undo" },
        { role: "redo" },
        { type: "separator" },
        { role: "cut" },
        { role: "copy" },
        { role: "paste" },
        { role: "selectAll" },
      ],
    },
    {
      label: "View",
      submenu: [
        {
          label: "Reload Active Tab",
          accelerator: "CmdOrCtrl+R",
          click: () => {
            if (mainWindow && !mainWindow.isDestroyed()) {
              mainWindow.webContents.send("tab:reload");
            }
          },
        },
        { role: "toggleDevTools" },
        { type: "separator" },
        {
          label: "Reset Zoom",
          accelerator: "CmdOrCtrl+0",
          click: () => mainWindow.webContents.send("zoom", "reset"),
        },
        {
          label: "Zoom In",
          accelerator: "CmdOrCtrl+=",
          click: () => mainWindow.webContents.send("zoom", "in"),
        },
        {
          label: "Zoom Out",
          accelerator: "CmdOrCtrl+-",
          click: () => mainWindow.webContents.send("zoom", "out"),
        },
        { type: "separator" },
        { role: "togglefullscreen" },
      ],
    },
  ];
  const menu = Menu.buildFromTemplate(template);
  Menu.setApplicationMenu(menu);
}

function toggleLockEnabled() {
  // Only allow toggling when app is unlocked
  if (isLocked) {
    return;
  }
  
  settings.lockEnabled = !settings.lockEnabled;
  saveSettings();
  createMainMenu();
  
  // If lock is disabled, hide overlay and reset timer
  if (!settings.lockEnabled && mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.webContents.send("lock:hide");
    if (inactivityTimer) {
      clearTimeout(inactivityTimer);
      inactivityTimer = null;
    }
  }
  
  // If lock is enabled again, start the inactivity timer
  if (settings.lockEnabled) {
    resetInactivityTimer();
  }
}
function resetInactivityTimer() {
  if (!settings.lockEnabled) return;
  if (inactivityTimer) clearTimeout(inactivityTimer);
  inactivityTimer = setTimeout(() => {
    showOverlayLock();
  }, settings.inactivityMs || INACTIVITY_DEFAULT_MS);
}
function showOverlayLock() {
  if (!mainWindow || mainWindow.isDestroyed()) return;
<<<<<<< HEAD
  if (!settings.lockEnabled) return;
  
  isLocked = true;
  // Send message to renderer to show lock overlay
  mainWindow.webContents.send("lock:show");
}

=======

  const bounds = mainWindow.getBounds();
  // Offset 28px down so macOS traffic-light buttons remain accessible
  const TITLEBAR_H = 28;
  overlayWindow = new BrowserWindow({
    parent: mainWindow,
    modal: false,
    frame: false,
    alwaysOnTop: false,
    skipTaskbar: true,
    resizable: false,
    movable: false,
    width: bounds.width,
    height: bounds.height - TITLEBAR_H,
    x: bounds.x,
    y: bounds.y + TITLEBAR_H,
    webPreferences: {
      contextIsolation: true,
      nodeIntegration: false,
      preload: path.join(__dirname, "overlay-preload.js"),
    },
  });

  overlayWindow.setFullScreenable(false);
  overlayWindow.loadFile("overlay.html");

  // Keep overlay covering content area on resize/move
  mainWindow.on("resize", syncOverlayBounds);
  mainWindow.on("move", syncOverlayBounds);

  overlayWindow.on("closed", () => {
    overlayWindow = null;
    mainWindow.removeListener("resize", syncOverlayBounds);
    mainWindow.removeListener("move", syncOverlayBounds);
    resetInactivityTimer();
  });
}

function syncOverlayBounds() {
  if (!overlayWindow || overlayWindow.isDestroyed()) return;
  const TITLEBAR_H = 28;
  const b = mainWindow.getBounds();
  overlayWindow.setBounds({
    x: b.x,
    y: b.y + TITLEBAR_H,
    width: b.width,
    height: b.height - TITLEBAR_H,
  });
}
>>>>>>> f36ebf7 (update)
function closeOverlay() {
  // The overlay is now handled internally by the renderer
  // This function is kept for compatibility but doesn't need to do anything
}

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1280,
    height: 900,
    webPreferences: {
      contextIsolation: true,
      nodeIntegration: false,
      preload: path.join(__dirname, "preload.js"),
      webviewTag: true,
    },
  });
  mainWindow.loadFile(path.join(__dirname, "index.html"));
  mainWindow.on("focus", () => resetInactivityTimer());
  mainWindow.on("blur", () => resetInactivityTimer());
  mainWindow.on("show", () => resetInactivityTimer());
  mainWindow.on("hide", () => resetInactivityTimer());
<<<<<<< HEAD
  mainWindow.on("closed", () => {
    mainWindow = null;
=======

  // Allow closing the app even when the overlay (lock) is active
  mainWindow.on("close", () => {
    if (overlayWindow && !overlayWindow.isDestroyed()) {
      overlayWindow.destroy();
      overlayWindow = null;
    }
>>>>>>> f36ebf7 (update)
  });
}

app.whenReady().then(() => {
  loadAllState();
  createWindow();
  createMainMenu();
  resetInactivityTimer();

  setTimeout(() => {
    showOverlayLock();
  }, 300);

  // ✅ Enhanced User Agent and security configuration for Google services compatibility
  const chromeVersion = process.versions.chrome;
  const platform =
    process.platform === "darwin"
      ? "Macintosh; Intel Mac OS X 10_15_7"
      : "Windows NT 10.0; Win64; x64";
  
  // Enhanced User Agent specifically for Google personal account compatibility
  app.userAgentFallback =
    `Mozilla/5.0 (${platform}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/${chromeVersion} Safari/537.36`;
  
  // Additional headers for Google compatibility
  app.on('web-contents-created', (event, contents) => {
    contents.on('before-input-event', (event, input) => {
      // Add any additional browser behavior simulation here
    });
    
    // Set additional headers that Google expects
    contents.session.webRequest.onBeforeSendHeaders((details, callback) => {
      details.requestHeaders['User-Agent'] = app.userAgentFallback;
      details.requestHeaders['Accept'] = 'text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9';
      details.requestHeaders['Accept-Language'] = 'en-US,en;q=0.9,es;q=0.8';
      details.requestHeaders['Accept-Encoding'] = 'gzip, deflate, br';
      details.requestHeaders['DNT'] = '1';
      details.requestHeaders['Connection'] = 'keep-alive';
      details.requestHeaders['Upgrade-Insecure-Requests'] = '1';
      callback({ cancel: false, requestHeaders: details.requestHeaders });
    });
  });
  
  // Configure SSL/TLS settings - more permissive for development
  app.on('certificate-error', (event, webContents, url, error, certificate, callback) => {
    // Ignore certificate errors for development to allow HTTPS sites to load
    event.preventDefault();
    callback(true);
  });
  
  // Configure session for better compatibility
  session.defaultSession.setUserAgent(app.userAgentFallback);
  
  // Enable spell checking and other features
  session.defaultSession.setSpellCheckerLanguages(['en-US', 'es-ES']);

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    } else if (mainWindow) {
      if (mainWindow.isMinimized()) mainWindow.restore();
      mainWindow.show();
      mainWindow.focus();
    }
  });
});

app.on("web-contents-created", (event, contents) => {
  if (contents.getType() === "webview") {
    // Intercept Google personal account login attempts
    contents.on('will-navigate', (event, navigationUrl) => {
      if (navigationUrl.includes('accounts.google.com') && 
          (navigationUrl.includes('gmail.com') || navigationUrl.includes('signin'))) {
        // For personal Gmail accounts, use system browser
        if (navigationUrl.includes('gmail.com') && !navigationUrl.includes('larico.net')) {
          event.preventDefault();
          shell.openExternal(navigationUrl);
          return;
        }
      }
    });
    
    contents.on("context-menu", (event, params) => {
      const menuTemplate = [];

      if (params.linkURL) {
        menuTemplate.push({
          label: "Open in default browser",
          click: () => {
            shell.openExternal(params.linkURL);
          },
        });
        menuTemplate.push({ type: "separator" });
      }

      menuTemplate.push(
        { role: "copy" },
        { role: "paste" },
        { type: "separator" },
        { role: "selectAll" },
        { type: "separator" },
        {
          label: "Reload this tab",
          click: () => contents.reload(),
        },
        { type: "separator" },
        {
          label: "Open in system browser (for Gmail)",
          click: () => {
            const currentUrl = contents.getURL();
            if (currentUrl) {
              shell.openExternal(currentUrl);
            }
          },
        }
      );

      Menu.buildFromTemplate(menuTemplate).popup({ window: mainWindow });
    });
  }
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});

// === IPC ===

ipcMain.handle("bookmarks:reorder", (e, newOrder) => {
  if (Array.isArray(newOrder)) {
    bookmarks = newOrder.filter(
      (b) =>
        b &&
        typeof b === "object" &&
        b.partition &&
        /^persist:/.test(b.partition)
    );
    saveBookmarks();
  }
  return bookmarks;
});

ipcMain.handle("state:get", () => {
  return { settings, bookmarks };
});

ipcMain.handle("bookmarks:add", (e, payload) => {
  const exists = bookmarks.find((b) => b.partition === payload.partition);
  if (!exists) {
    bookmarks.push({
      title: payload.title || payload.url || "Page",
      url: payload.url,
      partition: payload.partition,
    });
    saveBookmarks();
    createMainMenu();
  }
  return bookmarks;
});

ipcMain.handle("bookmarks:remove", (e, key) => {
  if (typeof key === "string") {
    bookmarks = bookmarks.filter((b) => b.url !== key);
  } else if (key && key.partition) {
    bookmarks = bookmarks.filter((b) => b.partition !== key.partition);
  }
  saveBookmarks();
  createMainMenu();
  return bookmarks;
});

ipcMain.handle("bookmarks:list", () => bookmarks);

ipcMain.handle("bookmarks:update-icon", (e, payload) => {
  // payload: { partition, iconSvg }
  if (!payload || !payload.partition) return bookmarks;
  const b = bookmarks.find((x) => x.partition === payload.partition);
  if (b) {
    if (
      payload.iconSvg &&
      typeof payload.iconSvg === "string" &&
      payload.iconSvg.trim() !== ""
    ) {
      b.iconSvg = payload.iconSvg;
    } else {
      // clear custom icon if empty
      delete b.iconSvg;
    }
    saveBookmarks();
    createMainMenu();
  }
  return bookmarks;
});

ipcMain.handle("session:create", (e, partitionName) => {
  const ses = session.fromPartition(partitionName);
  
  // Set the same user agent as the main app
  ses.setUserAgent(app.userAgentFallback);
  
  // Handle permissions more gracefully
  ses.setPermissionRequestHandler((webContents, permission, callback) => {
    // Allow most permissions for better compatibility
    if (['notifications', 'geolocation', 'camera', 'microphone', 'midi'].includes(permission)) {
      callback(true);
    } else {
      callback(true); // Allow all for now, can be restricted later
    }
  });
  
  // Handle certificate errors for webviews - completely permissive for development
  ses.setCertificateVerifyProc((request, callback) => {
    // Ignore ALL certificate errors to allow HTTPS sites to load
    callback(0); // 0 = OK
  });
  
  // Additional SSL bypass for webviews
  ses.setPermissionRequestHandler((webContents, permission, callback) => {
    callback(true); // Allow all permissions
  });
  
  // Enable spell checking
  ses.setSpellCheckerLanguages(['en-US', 'es-ES']);
  
  return true;
});

ipcMain.handle("lock:activity", () => {
  resetInactivityTimer();
  return true;
});
ipcMain.handle("lock:show", () => {
  showOverlayLock();
  return true;
});
ipcMain.handle("lock:verify", (e, pin) => {
  if (!settings.pinHash || !settings.pinSalt)
    return { ok: false, needsSetup: true };
  const ok = verifyPin(pin);
  if (ok && mainWindow && !mainWindow.isDestroyed()) {
    isLocked = false;
    // Send message to renderer to hide lock overlay
    mainWindow.webContents.send("lock:hide");
    // Reset inactivity timer after successful unlock
    resetInactivityTimer();
  }
  return { ok, needsSetup: false };
});

ipcMain.handle("lock:check", (e, pin) => {
  if (!settings.pinHash || !settings.pinSalt)
    return { ok: false, needsSetup: true };
  const ok = verifyPin(pin);
  return { ok, needsSetup: false };
});

ipcMain.handle("lock:setpin", (e, pin) => {
  setPin(pin);
  isLocked = false;
  if (mainWindow && !mainWindow.isDestroyed()) {
    // Send message to renderer to hide lock overlay
    mainWindow.webContents.send("lock:hide");
  }
  dialog.showMessageBox({
    type: "warning",
    title: "Importante",
    message:
      "Has cambiado tu PIN.\n\n⚠️ Si lo olvidas, no hay manera de recuperarlo.\nLa única opción será desinstalar y volver a instalar la aplicación.",
    buttons: ["Entendido"],
  });
  // Reset inactivity timer after setting PIN
  resetInactivityTimer();
  return { ok: true };
});

ipcMain.handle("lock:toggle", () => {
  toggleLockEnabled();
  return settings.lockEnabled;
});
ipcMain.handle("settings:setInactivity", (e, ms) => {
  settings.inactivityMs = Math.max(60000, Number(ms) || INACTIVITY_DEFAULT_MS);
  saveSettings();
  resetInactivityTimer();
  return settings.inactivityMs;
});

ipcMain.handle("app:focus", () => {
  if (mainWindow) {
    if (mainWindow.isMinimized()) mainWindow.restore();
    mainWindow.show();
    mainWindow.focus();
  }
  return true;
});

ipcMain.handle("get-webview-preload-path", () => {
  return path.join(__dirname, "webview-preload.js");
});
