import { app, BrowserWindow, ipcMain, session, Menu } from "electron";
import path, { dirname } from "path";
import fs from "fs";
import crypto from "crypto";
import { fileURLToPath } from "url";

// === Fix para __dirname en ESM ===
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// === Configuración ===
const INACTIVITY_DEFAULT_MS = 5 * 60 * 1000;
let mainWindow;
let overlayWindow;
let inactivityTimer = null;
let settings = null;
let bookmarks = [];
let pinnedTabs = [];

const userDataDir = app.getPath("userData");
const settingsPath = path.join(userDataDir, "settings.json");
const bookmarksPath = path.join(userDataDir, "bookmarks.json");
const pinnedPath = path.join(userDataDir, "pinned.json");

// === Helpers ===
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

  // Forzar lock habilitado
  settings.lockEnabled = true;

  // Si no hay PIN configurado, setear "123456"
  if (!settings.pinHash || !settings.pinSalt) {
    setPin("123456");
  }

  bookmarks = readJson(bookmarksPath, []);
  pinnedTabs = readJson(pinnedPath, []);
}
function saveSettings() {
  writeJson(settingsPath, settings);
}
function saveBookmarks() {
  writeJson(bookmarksPath, bookmarks);
}
function savePinned() {
  writeJson(pinnedPath, pinnedTabs);
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

// === Menú principal ===
function createMainMenu() {
  const bookmarksMenu = bookmarks.map((b) => ({
    label: b.title || b.url,
    click: () => {
      if (mainWindow && !mainWindow.isDestroyed()) {
        mainWindow.webContents.send("navigate-to", b.url);
      }
    },
  }));

  const template = [
    {
      label: app.name,
      submenu: [
        { role: "about" },
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
      label: "Bookmarks",
      submenu: bookmarksMenu.length
        ? bookmarksMenu
        : [{ label: "No bookmarks" }],
    },
    {
      label: "View",
      submenu: [
        { role: "reload" },
        { role: "toggleDevTools" },
        { type: "separator" },
        { role: "resetZoom" },
        { role: "zoomIn" },
        { role: "zoomOut" },
        { type: "separator" },
        { role: "togglefullscreen" },
      ],
    },
  ];
  const menu = Menu.buildFromTemplate(template);
  Menu.setApplicationMenu(menu);
}

// === Lock ===
function toggleLockEnabled() {
  settings.lockEnabled = !settings.lockEnabled;
  saveSettings();
  createMainMenu();
}
function resetInactivityTimer() {
  if (!settings.lockEnabled) return;
  if (
    overlayWindow &&
    !overlayWindow.isDestroyed() &&
    overlayWindow.isVisible()
  )
    return;
  if (inactivityTimer) clearTimeout(inactivityTimer);
  inactivityTimer = setTimeout(() => {
    showOverlayLock();
  }, settings.inactivityMs || INACTIVITY_DEFAULT_MS);
}
function showOverlayLock() {
  if (overlayWindow && !overlayWindow.isDestroyed()) return;
  if (!mainWindow || mainWindow.isDestroyed()) return;

  const bounds = mainWindow.getBounds();
  overlayWindow = new BrowserWindow({
    parent: mainWindow,
    modal: true,
    frame: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    resizable: false,
    movable: false,
    width: bounds.width,
    height: bounds.height,
    x: bounds.x,
    y: bounds.y,
    webPreferences: {
      contextIsolation: true,
      nodeIntegration: false,
      preload: path.join(__dirname, "overlay-preload.js"),
    },
  });

  overlayWindow.setAlwaysOnTop(true, "screen-saver");
  overlayWindow.setFullScreenable(false);
  overlayWindow.loadFile("overlay.html");

  mainWindow.on("resize", () => {
    if (overlayWindow && !overlayWindow.isDestroyed()) {
      overlayWindow.setBounds(mainWindow.getBounds());
    }
  });
  mainWindow.on("move", () => {
    if (overlayWindow && !overlayWindow.isDestroyed()) {
      overlayWindow.setBounds(mainWindow.getBounds());
    }
  });

  overlayWindow.on("closed", () => {
    overlayWindow = null;
    resetInactivityTimer();
  });
}

function closeOverlay() {
  if (overlayWindow && !overlayWindow.isDestroyed()) {
    overlayWindow.close();
    overlayWindow = null;
  }
}

// === Ventana principal ===
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
  mainWindow.loadFile("index.html");
  mainWindow.on("focus", () => resetInactivityTimer());
  mainWindow.on("blur", () => resetInactivityTimer());
  mainWindow.on("show", () => resetInactivityTimer());
  mainWindow.on("hide", () => resetInactivityTimer());
}

// === Ciclo de vida de la app ===
app.whenReady().then(() => {
  loadAllState();
  createWindow();
  createMainMenu();
  resetInactivityTimer();

  // 🔹 Forzar que arranque bloqueado
  setTimeout(() => {
    showOverlayLock();
  }, 300);

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});
app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});

// === IPC ===
ipcMain.handle("state:get", () => {
  return { settings, bookmarks, pinnedTabs };
});

ipcMain.handle("bookmarks:add", (e, payload) => {
  const exists = bookmarks.find((b) => b.url === payload.url);
  if (!exists) {
    bookmarks.push(payload);
    saveBookmarks();
    createMainMenu(); // 🔹 refrescar menú al agregar
  }
  return bookmarks;
});

ipcMain.handle("bookmarks:remove", (e, url) => {
  bookmarks = bookmarks.filter((b) => b.url !== url);
  saveBookmarks();
  createMainMenu(); // 🔹 refrescar menú al eliminar
  return bookmarks;
});

ipcMain.handle("bookmarks:list", () => bookmarks);

ipcMain.handle("pinned:set", (e, arr) => {
  pinnedTabs = Array.isArray(arr) ? arr : [];
  savePinned();
  return pinnedTabs;
});

ipcMain.handle("session:create", (e, partitionName) => {
  session.fromPartition(partitionName);
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
  if (ok) closeOverlay();
  return { ok, needsSetup: false };
});
ipcMain.handle("lock:setpin", (e, pin) => {
  setPin(pin);
  closeOverlay();
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
