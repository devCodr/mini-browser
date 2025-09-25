// preload.js
const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("api", {
  getState: () => ipcRenderer.invoke("state:get"),
  addBookmark: (b) => ipcRenderer.invoke("bookmarks:add", b),
  // 🔹 ahora puede recibir string (legacy) o {partition, url}
  removeBookmark: (key) => ipcRenderer.invoke("bookmarks:remove", key),
  listBookmarks: () => ipcRenderer.invoke("bookmarks:list"),
  setPinned: (arr) => ipcRenderer.invoke("pinned:set", arr),
  ensureSession: (p) => ipcRenderer.invoke("session:create", p),
  signalActivity: () => ipcRenderer.invoke("lock:activity"),
  showLock: () => ipcRenderer.invoke("lock:show"),
  verifyPin: (pin) => ipcRenderer.invoke("lock:verify", pin),
  setPin: (pin) => ipcRenderer.invoke("lock:setpin", pin),
  toggleLock: () => ipcRenderer.invoke("lock:toggle"),
  setInactivityMs: (ms) => ipcRenderer.invoke("settings:setInactivity", ms),

  // 🔹 escuchar navegación enviada desde el menú (si algún día lo reactivas)
  onNavigateTo: (callback) =>
    ipcRenderer.on("navigate-to", (e, url) => callback(url)),
});
