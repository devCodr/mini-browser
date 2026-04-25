// Base electronAPI configuration
// Shared between preload.js and webview-preload.js

const { contextBridge, ipcRenderer } = require("electron");

export const electronAPIConfig = {
  // State and bookmarks
  getState: () => ipcRenderer.invoke("state:get"),
  addBookmark: (b) => ipcRenderer.invoke("bookmarks:add", b),
  removeBookmark: (key) => ipcRenderer.invoke("bookmarks:remove", key),
  listBookmarks: () => ipcRenderer.invoke("bookmarks:list"),
  reorderBookmarks: (list) => ipcRenderer.invoke('bookmarks:reorder', list),
  updateBookmarkIcon: (payload) => ipcRenderer.invoke('bookmarks:update-icon', payload),
  
  // Session management
  ensureSession: (p) => ipcRenderer.invoke("session:create", p),
  getWebviewPreloadPath: () => ipcRenderer.invoke("get-webview-preload-path"),
  
  // Lock functionality
  signalActivity: () => ipcRenderer.invoke("lock:activity"),
  showLock: () => ipcRenderer.invoke("lock:show"),
  verifyPin: (pin) => ipcRenderer.invoke("lock:verify", pin),
  setPin: (pin) => ipcRenderer.invoke("lock:setpin", pin),
  toggleLock: () => ipcRenderer.invoke("lock:toggle"),
  setInactivityMs: (ms) => ipcRenderer.invoke("settings:setInactivity", ms),
  lockVerify: (pin) => ipcRenderer.invoke("lock:verify", pin),
  lockCheck: (pin) => ipcRenderer.invoke("lock:check", pin),
  lockSetPin: (pin) => ipcRenderer.invoke("lock:setpin", pin),
  
  // App functionality
  focusApp: () => ipcRenderer.invoke("app:focus"),
  
  // Events
  activateBookmark: (partition) => ipcRenderer.send('bookmark:activate', partition),
  onActivateBookmark: (cb) => ipcRenderer.on('bookmark:activate', (_e, partition) => cb(partition)),
  onNavigateTo: (callback) => ipcRenderer.on("navigate-to", (e, url) => callback(url)),
  onZoom: (callback) => ipcRenderer.on("zoom", (_e, dir) => callback(dir)),
  onTabReload: (callback) => ipcRenderer.on("tab:reload", () => callback()),
  onLockShow: (callback) => ipcRenderer.on("lock:show", () => callback()),
  onLockHide: (callback) => ipcRenderer.on("lock:hide", () => callback()),
  onBookmarksUpdated: (callback) => ipcRenderer.on("bookmarks-updated", (event, bookmarks) => callback(bookmarks)),
};

export function exposeElectronAPI(additionalAPIs = {}) {
  contextBridge.exposeInMainWorld("electronAPI", {
    ...electronAPIConfig,
    ...additionalAPIs
  });
}
