// overlay-preload.js
const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("overlay", {
  verify: (pin) => ipcRenderer.invoke("lock:verify", pin),
  setPin: (pin) => ipcRenderer.invoke("lock:setpin", pin),
  getState: () => ipcRenderer.invoke("state:get"),
});
