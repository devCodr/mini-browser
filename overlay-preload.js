// overlay-preload.js
const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("overlay", {
  verify: (pin) => ipcRenderer.invoke("lock:verify", pin),
  check: (pin) => ipcRenderer.invoke("lock:check", pin),
  setPin: (pin) => ipcRenderer.invoke("lock:setpin", pin),
  getState: () => ipcRenderer.invoke("state:get"),
});
