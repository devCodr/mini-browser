// preload.js
const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("electronAPI", {
  getState: () => ipcRenderer.invoke("state:get"),
  addBookmark: (b) => ipcRenderer.invoke("bookmarks:add", b),
  removeBookmark: (key) => ipcRenderer.invoke("bookmarks:remove", key),
  listBookmarks: () => ipcRenderer.invoke("bookmarks:list"),
  ensureSession: (p) => ipcRenderer.invoke("session:create", p),
  signalActivity: () => ipcRenderer.invoke("lock:activity"),
  showLock: () => ipcRenderer.invoke("lock:show"),
  verifyPin: (pin) => ipcRenderer.invoke("lock:verify", pin),
  setPin: (pin) => ipcRenderer.invoke("lock:setpin", pin),
  toggleLock: () => ipcRenderer.invoke("lock:toggle"),
  setInactivityMs: (ms) => ipcRenderer.invoke("settings:setInactivity", ms),

  onNavigateTo: (callback) =>
    ipcRenderer.on("navigate-to", (e, url) => callback(url)),

  onZoom: (callback) => ipcRenderer.on("zoom", (_e, dir) => callback(dir)),
  onTabReload: (callback) => ipcRenderer.on("tab:reload", () => callback()),
});
