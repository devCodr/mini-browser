let state = { settings: null, bookmarks: [], pinnedTabs: [] };
let tabs = [];
let activeTabId = null;

const tabsEl = document.getElementById("tabs");
const webviewsEl = document.getElementById("webviews");
const urlEl = document.getElementById("url");
const backEl = document.getElementById("back");
const fwdEl = document.getElementById("fwd");
const reloadEl = document.getElementById("reload");
const newtabEl = document.getElementById("newtab");
const pinTabEl = document.getElementById("pinTab");
const starEl = document.getElementById("star");
const sessionSel = document.getElementById("sessionSel");

function uuid() {
  return Math.random().toString(36).slice(2) + Date.now().toString(36);
}

function renderTabs() {
  tabsEl.innerHTML = "";
  const sorted = [...tabs].sort((a, b) => Number(b.pinned) - Number(a.pinned));
  sorted.forEach((t) => {
    const d = document.createElement("div");
    d.className = "tab" + (t.id === activeTabId ? " active" : "");
    d.dataset.id = t.id;

    const p = document.createElement("span");
    p.className = "pin";
    p.textContent = t.pinned ? "📌" : "";

    const title = document.createElement("span");
    title.textContent = t.title || "New Tab";

    const x = document.createElement("button");
    x.textContent = "×";
    x.addEventListener("click", (e) => {
      e.stopPropagation();
      closeTab(t.id);
    });

    d.appendChild(p);
    d.appendChild(title);
    d.appendChild(x);
    d.addEventListener("click", () => activateTab(t.id));

    tabsEl.appendChild(d);
  });
}

function renderWebviews() {
  webviewsEl.querySelectorAll("webview").forEach((w) => w.remove());
  tabs.forEach((t) => {
    let w = document.createElement("webview");
    w.setAttribute("partition", t.partition);
    w.setAttribute("allowpopups", "true");
    w.setAttribute("preload", "");
    w.className = t.id === activeTabId ? "active" : "";
    w.src = t.url || "https://www.google.com";

    w.addEventListener("page-title-updated", (e) => {
      t.title = e.title;
      if (t.id === activeTabId) document.title = e.title;
      renderTabs();
    });

    w.addEventListener("did-navigate", (e) => {
      if (t.id === activeTabId) urlEl.value = e.url;
      t.url = e.url;
      api.signalActivity();
    });

    w.addEventListener("did-navigate-in-page", () => api.signalActivity());
    w.addEventListener("dom-ready", () => api.signalActivity());
    w.addEventListener("ipc-message", () => api.signalActivity());

    webviewsEl.appendChild(w);
    t.webview = w;
  });
}

function activateTab(id) {
  activeTabId = id;
  document
    .querySelectorAll("webview")
    .forEach((w) => w.classList.remove("active"));
  const t = tabs.find((x) => x.id === id);
  if (t && t.webview) {
    t.webview.classList.add("active");
    urlEl.value = t.url || "";
    document.title = t.title || "Mini Browser";
  }
  renderTabs();
}

function closeTab(id) {
  const idx = tabs.findIndex((t) => t.id === id);
  if (idx < 0) return;
  const t = tabs[idx];
  if (t.webview) {
    t.webview.remove();
  }
  tabs.splice(idx, 1);
  if (activeTabId === id) {
    activeTabId = tabs[0] ? tabs[0].id : null;
    if (activeTabId) activateTab(activeTabId);
  }
  renderTabs();
}

function newTab(partition, url) {
  const id = uuid();
  const t = {
    id,
    partition,
    url: url || "https://www.google.com",
    title: "New Tab",
    pinned: false,
    webview: null,
  };
  tabs.push(t);
  renderTabs();
  renderWebviews();
  activateTab(id);
}

function togglePinActive() {
  const t = tabs.find((x) => x.id === activeTabId);
  if (!t) return;
  t.pinned = !t.pinned;
  persistPinned();
  renderTabs();
}

function persistPinned() {
  const arr = tabs
    .filter((t) => t.pinned)
    .map((t) => ({
      url: t.url,
      title: t.title || "",
      partition: t.partition,
    }));
  state.pinnedTabs = arr;
  api.setPinned(arr);
}

function restorePinned() {
  const arr = state.pinnedTabs || [];
  arr.forEach((p) => {
    newTab(p.partition, p.url);
    const t = tabs[tabs.length - 1];
    t.pinned = true;
  });
  renderTabs();
}

function navigateActive(url) {
  const t = tabs.find((x) => x.id === activeTabId);
  if (!t) return;
  if (!/^https?:\/\//i.test(url)) url = "https://" + url;
  t.webview.loadURL(url);
}

function goBack() {
  const t = tabs.find((x) => x.id === activeTabId);
  if (t && t.webview.canGoBack()) t.webview.goBack();
}

function goFwd() {
  const t = tabs.find((x) => x.id === activeTabId);
  if (t && t.webview.canGoForward()) t.webview.goForward();
}

function reload() {
  const t = tabs.find((x) => x.id === activeTabId);
  if (t) t.webview.reload();
}

async function addBookmarkFromActive() {
  const t = tabs.find((x) => x.id === activeTabId);
  if (!t) return;
  const b = await api.addBookmark({
    title: t.title || t.url || "Page",
    url: t.url,
  });
  state.bookmarks = b;
}

function attachActivityListeners() {
  const reset = () => api.signalActivity();
  window.addEventListener("mousemove", reset);
  window.addEventListener("keydown", reset);
  window.addEventListener("mousedown", reset);
  window.addEventListener("wheel", reset);
  window.addEventListener("focus", reset);
}

async function init() {
  state = await api.getState();
  restorePinned();
  if (tabs.length === 0) newTab("persist:web1", "https://www.google.com");
}

backEl.addEventListener("click", goBack);
fwdEl.addEventListener("click", goFwd);
reloadEl.addEventListener("click", reload);

newtabEl.addEventListener("click", async () => {
  const p = sessionSel.value;
  await api.ensureSession(p);
  newTab(p, "https://www.google.com");
});

pinTabEl.addEventListener("click", () => togglePinActive());
starEl.addEventListener("click", () => addBookmarkFromActive());

urlEl.addEventListener("keydown", (e) => {
  if (e.key === "Enter") navigateActive(urlEl.value);
});

tabsEl.addEventListener("dblclick", () => newtabEl.click());

attachActivityListeners();
init();

// 🔹 escuchar navegación desde el menú de favoritos
api.onNavigateTo((url) => {
  if (!activeTabId) {
    newTab("persist:web1", url);
  } else {
    navigateActive(url);
  }
});
