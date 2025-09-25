import {
  app,
  BrowserWindow,
  ipcMain,
  session,
  Menu,
  shell,
  dialog,
} from "electron";

import path, { dirname } from "path";
import fs from "fs";
import crypto from "crypto";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const INACTIVITY_DEFAULT_MS = 5 * 60 * 1000;
let mainWindow;
let overlayWindow;
let inactivityTimer = null;
let settings = null;
let bookmarks = [];

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

app.whenReady().then(() => {
  loadAllState();
  createWindow();
  createMainMenu();
  resetInactivityTimer();

  setTimeout(() => {
    showOverlayLock();
  }, 300);

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

// ✅ Enganchar menú contextual a cada webview
app.on("web-contents-created", (event, contents) => {
  if (contents.getType() === "webview") {
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

ipcMain.handle("lock:check", (e, pin) => {
  if (!settings.pinHash || !settings.pinSalt)
    return { ok: false, needsSetup: true };
  const ok = verifyPin(pin);
  return { ok, needsSetup: false };
});

ipcMain.handle("lock:setpin", (e, pin) => {
  setPin(pin);
  closeOverlay();
  dialog.showMessageBox({
    type: "warning",
    title: "Importante",
    message:
      "Has cambiado tu PIN.\n\n⚠️ Si lo olvidas, no hay manera de recuperarlo.\nLa única opción será desinstalar y volver a instalar la aplicación.",
    buttons: ["Entendido"],
  });
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
