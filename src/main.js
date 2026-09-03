// MiniBrowser - Rust & Tauri v2 Minimalist Frontend
const invoke = window.__TAURI__?.core?.invoke || (async (cmd, args) => {
  console.warn(`[Mock Tauri] Command: ${cmd}`, args);
  return null;
});

const listen = window.__TAURI__?.event?.listen || (() => {});

// === State ===
let state = {
  settings: {
    lockEnabled: true,
    inactivityMs: 300000,
    lockOnLaunch: false,
    startMinimized: false,
  },
  bookmarks: [],
  activePartition: null,
  isLocked: false,
  pinBuffer: "",
  inactivityTimer: null,
  zoomLevel: 1.0,
};

let draggedTabIndex = null;
let draggedManageIndex = null;

// === DOM Elements ===
const header = document.getElementById("header");
const btnBack = document.getElementById("btn-back");
const btnFwd = document.getElementById("btn-fwd");
const btnReload = document.getElementById("btn-reload");
const btnHome = document.getElementById("btn-home");
const urlPill = document.getElementById("url-pill");
const urlDomain = document.getElementById("url-domain");
const urlInput = document.getElementById("url-input");
const tabsBar = document.getElementById("tabs-bar");
const btnNewSession = document.getElementById("btn-new-session");
const btnManageSessions = document.getElementById("btn-manage-sessions");
const btnLock = document.getElementById("btn-lock");
const btnShortcuts = document.getElementById("btn-shortcuts");
const btnAbout = document.getElementById("btn-about");
const btnSettings = document.getElementById("btn-settings");
const welcomeView = document.getElementById("welcome-view");

// Modals
const modalNewSession = document.getElementById("modal-new-session");
const btnCloseModal = document.getElementById("btn-close-modal");
const btnCancelModal = document.getElementById("btn-cancel-modal");
const formNewSession = document.getElementById("form-new-session");

const modalManageSessions = document.getElementById("modal-manage-sessions");
const btnCloseManage = document.getElementById("btn-close-manage");
const btnManageDone = document.getElementById("btn-manage-done");
const btnManageAdd = document.getElementById("btn-manage-add");
const manageSessionsList = document.getElementById("manage-sessions-list");

const modalShortcuts = document.getElementById("modal-shortcuts");
const btnCloseShortcuts = document.getElementById("btn-close-shortcuts");

const modalAbout = document.getElementById("modal-about");
const btnCloseAbout = document.getElementById("btn-close-about");

const modalSettings = document.getElementById("modal-settings");
const btnCloseSettings = document.getElementById("btn-close-settings");
const settingLockToggle = document.getElementById("setting-lock-toggle");
const settingLockOnLaunch = document.getElementById("setting-lock-on-launch");
const settingStartMinimized = document.getElementById("setting-start-minimized");
const settingTimeout = document.getElementById("setting-timeout");
const timeoutDisplay = document.getElementById("timeout-display");
const formChangePin = document.getElementById("form-change-pin");
const inputNewPin = document.getElementById("input-new-pin");

// Lock Overlay
const lockOverlay = document.getElementById("lock-overlay");
const lockCard = document.querySelector(".lock-card");
const pinDots = document.querySelectorAll(".pin-dot");
const hiddenPinInput = document.getElementById("hidden-pin-input");
const pinErrorMsg = document.getElementById("pin-error-msg");
const keyButtons = document.querySelectorAll(".key-btn[data-key]");
const btnPinClear = document.getElementById("btn-pin-clear");
const btnPinDel = document.getElementById("btn-pin-del");

// Custom Launch on Dashboard
const formCustomLaunch = document.getElementById("form-custom-launch");
const inputCustomUrl = document.getElementById("input-custom-url");

// Tab Context Menu (Right Click)
const tabContextMenu = document.getElementById("tab-context-menu");
const ctxMoveLeft = document.getElementById("ctx-move-left");
const ctxMoveRight = document.getElementById("ctx-move-right");
const ctxManage = document.getElementById("ctx-manage");
const ctxClose = document.getElementById("ctx-close");
let contextTargetPartition = null;

function showTabContextMenu(x, y, partition, idx) {
  contextTargetPartition = partition;
  ctxMoveLeft.style.display = idx > 0 ? "flex" : "none";
  ctxMoveRight.style.display = idx < state.bookmarks.length - 1 ? "flex" : "none";
  tabContextMenu.style.left = `${Math.min(x, window.innerWidth - 210)}px`;
  tabContextMenu.style.top = `${y + 8}px`;
  tabContextMenu.classList.remove("hidden");
}

function hideTabContextMenu() {
  if (tabContextMenu) tabContextMenu.classList.add("hidden");
  contextTargetPartition = null;
}

window.addEventListener("click", (e) => {
  if (tabContextMenu && !tabContextMenu.contains(e.target)) {
    hideTabContextMenu();
  }
});

ctxMoveLeft.addEventListener("click", () => {
  if (contextTargetPartition) moveTabByPartition(contextTargetPartition, -1);
  hideTabContextMenu();
});

ctxMoveRight.addEventListener("click", () => {
  if (contextTargetPartition) moveTabByPartition(contextTargetPartition, 1);
  hideTabContextMenu();
});

ctxManage.addEventListener("click", () => {
  hideTabContextMenu();
  openManageSessionsModal();
});

ctxClose.addEventListener("click", () => {
  if (contextTargetPartition) removeSession(contextTargetPartition);
  hideTabContextMenu();
});

async function moveTabByPartition(partition, delta) {
  const idx = state.bookmarks.findIndex((b) => b.partition === partition);
  if (idx === -1) return;
  const targetIdx = idx + delta;
  if (targetIdx < 0 || targetIdx >= state.bookmarks.length) return;

  const item = state.bookmarks.splice(idx, 1)[0];
  state.bookmarks.splice(targetIdx, 0, item);

  try {
    const updated = await invoke("reorder_bookmarks", { newOrder: state.bookmarks });
    state.bookmarks = updated;
  } catch (err) {
    console.error("Error moving tab:", err);
  }

  renderTabs();
  if (!modalManageSessions.classList.contains("hidden")) {
    renderManageSessionsList();
  }
}

// === Window Dragging (Reliable native dragging on header empty space) ===
header.addEventListener("mousedown", async (e) => {
  // Never start window dragging if clicking inside tabs, buttons, or inputs
  if (e.target.closest("button, input, select, .tab-item, a, .key-btn, .preset-card, .session-manage-row, .tabs-bar")) return;
  try {
    await invoke("start_dragging");
  } catch (err) {
    try {
      if (window.__TAURI__?.window?.getCurrentWindow) {
        await window.__TAURI__.window.getCurrentWindow().startDragging();
      }
    } catch (_) {}
  }
});

// === Horizontal Scroll on Tabs Bar with Mouse Wheel ===
tabsBar.addEventListener("wheel", (e) => {
  if (e.deltaY !== 0) {
    e.preventDefault();
    tabsBar.scrollLeft += e.deltaY;
  }
}, { passive: false });

tabsBar.addEventListener("dragover", (e) => {
  e.preventDefault();
  e.dataTransfer.dropEffect = "move";
});

manageSessionsList.addEventListener("dragover", (e) => {
  e.preventDefault();
  e.dataTransfer.dropEffect = "move";
});

// === Modal Overlay Management (Hides native child webview so modals are 100% in front) ===
async function showModal(modalEl) {
  modalEl.classList.remove("hidden");
  if (state.activePartition) {
    try {
      await invoke("hide_active_session");
    } catch (_) {}
  }
}

async function hideModal(modalEl) {
  modalEl.classList.add("hidden");
  const anyModalOpen =
    !modalNewSession.classList.contains("hidden") ||
    !modalManageSessions.classList.contains("hidden") ||
    !modalShortcuts.classList.contains("hidden") ||
    !modalAbout.classList.contains("hidden") ||
    !modalSettings.classList.contains("hidden") ||
    state.isLocked;

  if (!anyModalOpen && state.activePartition) {
    try {
      await invoke("show_active_session");
    } catch (_) {}
  }
}

// === Helpers ===
function getDomain(urlStr) {
  try {
    return new URL(urlStr).hostname.replace(/^www\./, "");
  } catch {
    return urlStr || "Welcome";
  }
}

function domainSlug(urlStr) {
  try {
    return new URL(urlStr).hostname.replace(/\W+/g, "_");
  } catch {
    return "site";
  }
}

function nextPartition(domain) {
  let max = 0;
  const rx = new RegExp(`^persist:${domain}(\\d+)?$`);
  for (const b of state.bookmarks) {
    const m = b.partition && b.partition.match(rx);
    if (m) {
      const n = m[1] ? parseInt(m[1], 10) : 1;
      if (!isNaN(n) && n > max) max = n;
    }
  }
  return `persist:${domain}${max >= 1 ? max + 1 : ""}`;
}

function autoBadge(urlStr) {
  const d = getDomain(urlStr);
  return d.substring(0, 4).toUpperCase();
}

function normalizeUrl(rawUrl) {
  let url = rawUrl.trim();
  if (!url) return "";

  // Direct login for Gmail / Google to bypass Workspace marketing redirects
  if (
    /^(https?:\/\/)?(www\.)?(gmail\.com|mail\.google\.com)\/?$/i.test(url) ||
    url.toLowerCase() === "gmail"
  ) {
    return "https://accounts.google.com/ServiceLogin?service=mail&continue=https://mail.google.com/mail/";
  }

  if (!/^https?:\/\//i.test(url)) {
    if (url.includes(".") && !url.includes(" ")) {
      url = "https://" + url;
    } else {
      url = `https://www.google.com/search?q=${encodeURIComponent(url)}`;
    }
  }
  return url;
}

// === Render Tabs with Drag & Drop Reordering ===
function renderTabs() {
  tabsBar.innerHTML = "";

  state.bookmarks.forEach((bm, idx) => {
    const tab = document.createElement("div");
    tab.className = `tab-item ${bm.partition === state.activePartition ? "active" : ""}`;
    tab.draggable = false;
    tab.dataset.partition = bm.partition;
    tab.dataset.index = idx;
    tab.title = `${bm.title || bm.url} (Cmd+${idx + 1})`;

    // Favicon or Domain Initial
    const domain = getDomain(bm.url);
    const favicon = document.createElement("img");
    favicon.className = "tab-favicon";
    favicon.src = `https://www.google.com/s2/favicons?sz=32&domain=${domain}`;
    favicon.onerror = () => {
      favicon.style.display = "none";
    };
    tab.appendChild(favicon);

    // Title / Domain
    const titleSpan = document.createElement("span");
    titleSpan.textContent = bm.title || domain;
    tab.appendChild(titleSpan);

    // Badge if set
    if (bm.badge) {
      const badgeSpan = document.createElement("span");
      badgeSpan.className = "tab-badge";
      badgeSpan.textContent = bm.badge;
      if (bm.color) {
        badgeSpan.style.background = bm.color + "22";
        badgeSpan.style.color = bm.color;
      }
      tab.appendChild(badgeSpan);
    }

    // Close Button (Immediate removal on click without drag conflict)
    const closeBtn = document.createElement("button");
    closeBtn.className = "tab-close-btn";
    closeBtn.innerHTML = "&times;";
    closeBtn.title = "Close Session (Cmd+W)";
    closeBtn.addEventListener("mousedown", (e) => e.stopPropagation());
    closeBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      e.preventDefault();
      removeSession(bm.partition);
    });
    tab.appendChild(closeBtn);

    // Right-Click Context Menu on Tab
    tab.addEventListener("contextmenu", (e) => {
      e.preventDefault();
      e.stopPropagation();
      showTabContextMenu(e.clientX, e.clientY, bm.partition, idx);
    });

    // --- Rock-Solid Pointer Drag & Drop (Window-level tracking) ---
    let startX = 0;
    let isDraggingTab = false;

    tab.addEventListener("pointerdown", (e) => {
      if (e.target.closest(".tab-close-btn") || e.button !== 0) return;
      startX = e.clientX;
      isDraggingTab = false;

      function onPointerMove(moveEvent) {
        const dx = moveEvent.clientX - startX;
        if (!isDraggingTab && Math.abs(dx) > 6) {
          isDraggingTab = true;
          tab.classList.add("dragging");
        }

        if (isDraggingTab) {
          const otherTabs = Array.from(tabsBar.querySelectorAll(".tab-item:not(.dragging)"));
          for (const other of otherTabs) {
            const rect = other.getBoundingClientRect();
            if (moveEvent.clientX >= rect.left && moveEvent.clientX <= rect.right) {
              const isAfter = moveEvent.clientX > rect.left + rect.width / 2;
              tabsBar.insertBefore(tab, isAfter ? other.nextSibling : other);
              break;
            }
          }
        }
      }

      async function onPointerUp() {
        window.removeEventListener("pointermove", onPointerMove);
        window.removeEventListener("pointerup", onPointerUp);

        if (isDraggingTab) {
          tab.classList.remove("dragging");
          const tabElements = Array.from(tabsBar.querySelectorAll(".tab-item"));
          const newOrder = tabElements
            .map((el) => state.bookmarks.find((b) => b.partition === el.dataset.partition))
            .filter(Boolean);

          state.bookmarks = newOrder;
          try {
            await invoke("reorder_bookmarks", { newOrder });
          } catch (err) {
            console.error("Error saving tab order:", err);
          }

          renderTabs();
          if (!modalManageSessions.classList.contains("hidden")) {
            renderManageSessionsList();
          }
        } else {
          // Normal click without drag -> Activate session
          activateSession(bm.partition, bm.url);
        }
      }

      window.addEventListener("pointermove", onPointerMove);
      window.addEventListener("pointerup", onPointerUp);
    });

    tabsBar.appendChild(tab);
  });
}

// === Render Manage Sessions List with Up/Down and Drag & Drop ===
function renderManageSessionsList() {
  manageSessionsList.innerHTML = "";
  if (state.bookmarks.length === 0) {
    manageSessionsList.innerHTML = `<div style="text-align:center; padding: 28px; color: var(--text-muted); font-size: 13px;">No active sessions saved yet.<br><br>Click "Add New Session" below to create one.</div>`;
    return;
  }

  state.bookmarks.forEach((bm, idx) => {
    const row = document.createElement("div");
    row.className = "session-manage-row";
    row.draggable = true;
    row.dataset.index = idx;
    row.dataset.partition = bm.partition;

    const domain = getDomain(bm.url);

    row.innerHTML = `
      <img class="session-manage-favicon" src="https://www.google.com/s2/favicons?sz=32&domain=${domain}" onerror="this.style.display='none'" />
      <div class="session-manage-info">
        <span class="session-manage-title">${bm.title || domain}</span>
        <span class="session-manage-url">${bm.url}</span>
      </div>
      <span class="session-manage-badge" style="background: ${bm.color ? bm.color + '22' : 'rgba(99, 102, 241, 0.15)'}; color: ${bm.color || '#6366f1'};">${bm.badge || 'WEB'}</span>
      <div class="session-manage-actions">
        <button class="sm-btn btn-move-up" title="Move Up" ${idx === 0 ? "disabled" : ""}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="m18 15-6-6-6 6"/></svg>
        </button>
        <button class="sm-btn btn-move-down" title="Move Down" ${idx === state.bookmarks.length - 1 ? "disabled" : ""}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="m6 9 6 6 6-6"/></svg>
        </button>
        <button class="sm-btn btn-open-session" title="Open Session">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><polygon points="5 3 19 12 5 21 5 3"/></svg>
        </button>
        <button class="sm-btn btn-edit-session" title="Edit Name & Badge">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/></svg>
        </button>
        <button class="sm-btn danger btn-del-session" title="Delete Session">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
        </button>
      </div>
    `;

    // Move Up Button (Direct 1-Click Reorder)
    const upBtn = row.querySelector(".btn-move-up");
    if (upBtn) {
      upBtn.addEventListener("click", async (e) => {
        e.stopPropagation();
        e.preventDefault();
        if (idx <= 0) return;
        const item = state.bookmarks.splice(idx, 1)[0];
        state.bookmarks.splice(idx - 1, 0, item);
        const updated = await invoke("reorder_bookmarks", { newOrder: state.bookmarks });
        state.bookmarks = updated;
        renderTabs();
        renderManageSessionsList();
      });
    }

    // Move Down Button (Direct 1-Click Reorder)
    const downBtn = row.querySelector(".btn-move-down");
    if (downBtn) {
      downBtn.addEventListener("click", async (e) => {
        e.stopPropagation();
        e.preventDefault();
        if (idx >= state.bookmarks.length - 1) return;
        const item = state.bookmarks.splice(idx, 1)[0];
        state.bookmarks.splice(idx + 1, 0, item);
        const updated = await invoke("reorder_bookmarks", { newOrder: state.bookmarks });
        state.bookmarks = updated;
        renderTabs();
        renderManageSessionsList();
      });
    }

    // Pointer Drag on Row
    let startY = 0;
    let isDraggingRow = false;

    row.addEventListener("pointerdown", (e) => {
      if (e.target.closest("button, input, a") || e.button !== 0) return;
      startY = e.clientY;
      isDraggingRow = false;

      function onRowPointerMove(moveEvent) {
        const dy = moveEvent.clientY - startY;
        if (!isDraggingRow && Math.abs(dy) > 5) {
          isDraggingRow = true;
          row.classList.add("dragging");
          try {
            row.setPointerCapture(moveEvent.pointerId);
          } catch (_) {}
        }

        if (isDraggingRow) {
          const otherRows = Array.from(manageSessionsList.querySelectorAll(".session-manage-row:not(.dragging)"));
          for (const other of otherRows) {
            const rect = other.getBoundingClientRect();
            if (moveEvent.clientY >= rect.top && moveEvent.clientY <= rect.bottom) {
              const isAfter = moveEvent.clientY > rect.top + rect.height / 2;
              manageSessionsList.insertBefore(row, isAfter ? other.nextSibling : other);
              break;
            }
          }
        }
      }

      async function onRowPointerUp(upEvent) {
        row.removeEventListener("pointermove", onRowPointerMove);
        row.removeEventListener("pointerup", onRowPointerUp);
        try {
          row.releasePointerCapture(upEvent.pointerId);
        } catch (_) {}

        if (isDraggingRow) {
          row.classList.remove("dragging");
          const currentRows = Array.from(manageSessionsList.querySelectorAll(".session-manage-row"));
          const newOrder = currentRows
            .map((el) => state.bookmarks.find((b) => b.partition === el.dataset.partition))
            .filter(Boolean);

          state.bookmarks = newOrder;
          try {
            await invoke("reorder_bookmarks", { newOrder });
          } catch (err) {
            console.error("Error saving session order:", err);
          }
          renderTabs();
          renderManageSessionsList();
        }
      }

      row.addEventListener("pointermove", onRowPointerMove);
      row.addEventListener("pointerup", onRowPointerUp);
    });

    // Open
    row.querySelector(".btn-open-session").addEventListener("click", (e) => {
      e.stopPropagation();
      hideModal(modalManageSessions);
      activateSession(bm.partition, bm.url);
    });

    // Delete (Instant deletion without blocking alerts)
    row.querySelector(".btn-del-session").addEventListener("click", async (e) => {
      e.stopPropagation();
      e.preventDefault();
      await removeSession(bm.partition);
      renderManageSessionsList();
    });

    // Edit Panel Toggle
    const editBtn = row.querySelector(".btn-edit-session");
    editBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      let editPanel = row.nextElementSibling;
      if (editPanel && editPanel.classList.contains("session-edit-panel")) {
        editPanel.remove();
        return;
      }
      editPanel = document.createElement("div");
      editPanel.className = "session-edit-panel";
      editPanel.innerHTML = `
        <div class="edit-row">
          <label>Title</label>
          <input type="text" class="edit-title" value="${bm.title || ''}" placeholder="Session title" />
        </div>
        <div class="edit-row">
          <label>Badge</label>
          <input type="text" class="edit-badge" value="${bm.badge || ''}" maxlength="5" placeholder="e.g. W1, G2, Work" />
        </div>
        <div class="edit-actions">
          <button class="edit-btn-cancel">Cancel</button>
          <button class="edit-btn-save">Save Changes</button>
        </div>
      `;

      editPanel.querySelector(".edit-btn-cancel").addEventListener("click", () => editPanel.remove());
      editPanel.querySelector(".edit-btn-save").addEventListener("click", async () => {
        const newTitle = editPanel.querySelector(".edit-title").value.trim() || bm.title;
        const newBadge = editPanel.querySelector(".edit-badge").value.trim().toUpperCase() || bm.badge;
        const updated = await invoke("update_bookmark_meta", {
          partition: bm.partition,
          title: newTitle,
          badge: newBadge,
          color: bm.color || "#6366f1",
        });
        state.bookmarks = updated;
        renderTabs();
        renderManageSessionsList();
      });

      row.after(editPanel);
    });

    manageSessionsList.appendChild(row);
  });
}

// === Session Actions ===
async function activateSession(partition, url) {
  state.activePartition = partition;
  welcomeView.style.display = "none";

  // Update URL pill without layout shift
  urlDomain.textContent = getDomain(url);
  urlInput.value = url;

  renderTabs();

  try {
    await invoke("activate_session", { partition, url });
  } catch (err) {
    console.error("Error activating session:", err);
  }
}

async function goHome() {
  state.activePartition = null;
  welcomeView.style.display = "flex";
  urlDomain.textContent = "Welcome";
  urlInput.value = "";
  renderTabs();

  try {
    await invoke("deactivate_all");
  } catch (err) {
    console.error("Error going home:", err);
  }
}

async function createSession(urlStr, titleStr, badgeStr, colorStr) {
  const url = normalizeUrl(urlStr);
  if (!url) return;

  const domain = domainSlug(url);
  const partition = nextPartition(domain);
  const badge = (badgeStr || autoBadge(url)).substring(0, 4).toUpperCase();
  const title = titleStr.trim() || getDomain(url);
  const color = colorStr || "#6366f1";

  try {
    const list = await invoke("add_bookmark", {
      title,
      url,
      partition,
      badge,
      color,
      iconSvg: null,
    });
    state.bookmarks = list;
    renderTabs();
    await activateSession(partition, url);
  } catch (err) {
    console.error("Error creating session:", err);
  }
}

async function removeSession(partition) {
  const isCurrent = state.activePartition === partition;
  try {
    const list = await invoke("remove_bookmark", { partition });
    state.bookmarks = list;

    if (isCurrent) {
      if (state.bookmarks.length > 0) {
        const next = state.bookmarks[0];
        activateSession(next.partition, next.url);
      } else {
        goHome();
      }
    } else {
      renderTabs();
    }
  } catch (err) {
    console.error("Error removing session:", err);
  }
}

// === Inactivity & Lock Screen ===
function resetInactivityTimer() {
  if (state.isLocked || !state.settings.lockEnabled) return;
  if (state.inactivityTimer) clearTimeout(state.inactivityTimer);
  state.inactivityTimer = setTimeout(() => {
    lockApp();
  }, state.settings.inactivityMs || 300000);
}

async function lockApp() {
  if (state.isLocked) return;
  state.isLocked = true;
  state.pinBuffer = "";
  updatePinDots();
  pinErrorMsg.classList.add("hidden");
  lockOverlay.classList.remove("hidden");

  try {
    await invoke("hide_active_session");
  } catch (err) {
    console.error("Error hiding webview for lock:", err);
  }

  setTimeout(() => {
    hiddenPinInput.value = "";
    hiddenPinInput.focus();
  }, 100);
}

function updatePinDots() {
  pinDots.forEach((dot, i) => {
    if (i < state.pinBuffer.length) {
      dot.classList.add("filled");
    } else {
      dot.classList.remove("filled");
    }
  });
}

async function handlePinInput(char) {
  if (state.pinBuffer.length < 6) {
    state.pinBuffer += char;
    updatePinDots();
    pinErrorMsg.classList.add("hidden");

    if (state.pinBuffer.length === 6) {
      const pinToVerify = state.pinBuffer;
      const isValid = await invoke("verify_pin", { pin: pinToVerify });
      if (isValid) {
        unlockApp();
      } else {
        triggerPinError();
      }
    }
  }
}

function triggerPinError() {
  lockCard.classList.add("shake");
  pinErrorMsg.classList.remove("hidden");
  setTimeout(() => {
    lockCard.classList.remove("shake");
    state.pinBuffer = "";
    updatePinDots();
    hiddenPinInput.value = "";
  }, 400);
}

async function unlockApp() {
  state.isLocked = false;
  lockOverlay.classList.add("hidden");
  state.pinBuffer = "";
  updatePinDots();
  resetInactivityTimer();

  // Restore active session webview
  if (state.activePartition) {
    try {
      await invoke("show_active_session");
    } catch (err) {}
  } else {
    goHome();
  }
}

// === Navigation Controls ===
btnBack.addEventListener("click", () => {
  if (state.activePartition) invoke("nav_back", { partition: state.activePartition });
});

btnFwd.addEventListener("click", () => {
  if (state.activePartition) invoke("nav_forward", { partition: state.activePartition });
});

btnReload.addEventListener("click", () => {
  if (state.activePartition) invoke("nav_reload", { partition: state.activePartition });
});

btnHome.addEventListener("click", goHome);

// Smart URL Pill (No layout shift)
urlPill.addEventListener("click", () => {
  urlPill.style.display = "none";
  urlInput.style.display = "block";
  urlInput.focus();
  urlInput.select();
});

urlInput.addEventListener("keydown", async (e) => {
  if (e.key === "Enter") {
    const rawVal = urlInput.value.trim();
    if (!rawVal) return;
    const url = normalizeUrl(rawVal);

    if (state.activePartition) {
      await invoke("nav_to", { partition: state.activePartition, url });
      urlDomain.textContent = getDomain(url);
    } else {
      await createSession(url, "", "", "");
    }
    collapseUrlBar();
  } else if (e.key === "Escape") {
    collapseUrlBar();
  }
});

urlInput.addEventListener("blur", () => {
  setTimeout(collapseUrlBar, 150);
});

function collapseUrlBar() {
  urlInput.style.display = "none";
  urlPill.style.display = "flex";
}

// === Presets on Dashboard ===
document.querySelectorAll(".preset-card").forEach((card) => {
  card.addEventListener("click", () => {
    const url = card.dataset.url;
    const title = card.dataset.title;
    const badge = card.dataset.badge;
    const color = card.dataset.color;
    createSession(url, title, badge, color);
  });
});

// Custom launch form on dashboard
formCustomLaunch.addEventListener("submit", (e) => {
  e.preventDefault();
  const url = inputCustomUrl.value.trim();
  if (url) {
    createSession(url, "", "", "");
    inputCustomUrl.value = "";
  }
});

// === Modals Management ===
function openNewSessionModal() {
  showModal(modalNewSession);
  setTimeout(() => document.getElementById("new-session-url").focus(), 60);
}

function openManageSessionsModal() {
  renderManageSessionsList();
  showModal(modalManageSessions);
}

function toggleShortcutsModal() {
  if (modalShortcuts.classList.contains("hidden")) {
    showModal(modalShortcuts);
  } else {
    hideModal(modalShortcuts);
  }
}

// Header Button Listeners
btnNewSession.addEventListener("click", openNewSessionModal);
btnCloseModal.addEventListener("click", () => hideModal(modalNewSession));
btnCancelModal.addEventListener("click", () => hideModal(modalNewSession));

btnManageSessions.addEventListener("click", openManageSessionsModal);
btnCloseManage.addEventListener("click", () => hideModal(modalManageSessions));
btnManageDone.addEventListener("click", () => hideModal(modalManageSessions));
btnManageAdd.addEventListener("click", () => {
  hideModal(modalManageSessions);
  openNewSessionModal();
});

btnShortcuts.addEventListener("click", toggleShortcutsModal);
btnCloseShortcuts.addEventListener("click", () => hideModal(modalShortcuts));



btnAbout.addEventListener("click", () => showModal(modalAbout));
btnCloseAbout.addEventListener("click", () => hideModal(modalAbout));

formNewSession.addEventListener("submit", (e) => {
  e.preventDefault();
  const url = document.getElementById("new-session-url").value;
  const title = document.getElementById("new-session-title").value;
  const badge = document.getElementById("new-session-badge").value;
  const color = document.querySelector('input[name="session-color"]:checked')?.value;

  hideModal(modalNewSession);
  createSession(url, title, badge, color);
  formNewSession.reset();
});

btnSettings.addEventListener("click", () => {
  settingLockToggle.checked = state.settings.lockEnabled;
  settingLockOnLaunch.checked = !!state.settings.lockOnLaunch;
  settingStartMinimized.checked = !!state.settings.startMinimized;
  settingTimeout.value = state.settings.inactivityMs;
  timeoutDisplay.textContent = `${state.settings.inactivityMs / 60000} minutes`;
  showModal(modalSettings);
});

btnCloseSettings.addEventListener("click", () => hideModal(modalSettings));

async function saveUpdatedSettings() {
  try {
    const updated = await invoke("update_settings", {
      lockEnabled: settingLockToggle.checked,
      inactivityMs: parseInt(settingTimeout.value, 10),
      lockOnLaunch: settingLockOnLaunch.checked,
      startMinimized: settingStartMinimized.checked,
    });
    if (updated) state.settings = updated;
    resetInactivityTimer();
  } catch (err) {
    console.error("Error saving settings:", err);
  }
}

settingLockToggle.addEventListener("change", saveUpdatedSettings);
settingLockOnLaunch.addEventListener("change", saveUpdatedSettings);
settingStartMinimized.addEventListener("change", saveUpdatedSettings);

settingTimeout.addEventListener("change", (e) => {
  const ms = parseInt(e.target.value, 10);
  timeoutDisplay.textContent = `${ms / 60000} minutes`;
  saveUpdatedSettings();
});

formChangePin.addEventListener("submit", async (e) => {
  e.preventDefault();
  const pin = inputNewPin.value.trim();
  if (pin.length === 6) {
    await invoke("set_pin", { pin });
    inputNewPin.value = "";
    alert("PIN successfully updated!");
    hideModal(modalSettings);
  }
});

btnLock.addEventListener("click", lockApp);

// Keypad Clicks
keyButtons.forEach((btn) => {
  btn.addEventListener("click", () => handlePinInput(btn.dataset.key));
});

btnPinClear.addEventListener("click", () => {
  state.pinBuffer = "";
  updatePinDots();
});

btnPinDel.addEventListener("click", () => {
  state.pinBuffer = state.pinBuffer.slice(0, -1);
  updatePinDots();
});

// === Centralized Shortcut Dispatcher ===
function executeShortcut(key, { alt = false, shift = false } = {}) {
  resetInactivityTimer();
  if (state.isLocked) return;

  const k = (key || "").toLowerCase();

  // Tab switching: Cmd + 1..9
  if (k >= "1" && k <= "9") {
    const tabIdx = parseInt(k, 10) - 1;
    if (state.bookmarks[tabIdx]) {
      const target = state.bookmarks[tabIdx];
      activateSession(target.partition, target.url);
    }
  }
  // New session: Cmd + T / Cmd + N
  else if (k === "t" || k === "n") {
    openNewSessionModal();
  }
  // Manage sessions: Cmd + M
  else if (k === "m") {
    openManageSessionsModal();
  }
  // Close active session: Cmd + W
  else if (k === "w") {
    if (state.activePartition) {
      removeSession(state.activePartition);
    }
  }
  // Home: Cmd + H
  else if (k === "h") {
    goHome();
  }
  // Reload: Cmd + R
  else if (k === "r") {
    if (state.activePartition) invoke("nav_reload", { partition: state.activePartition });
  }
  // Focus address bar: Cmd + L
  else if (k === "l") {
    if (alt) {
      lockApp();
    } else {
      urlPill.click();
    }
  }
  // Move tab left: Cmd + Alt + Left
  else if (key === "ArrowLeft" && alt) {
    if (state.activePartition) moveTabByPartition(state.activePartition, -1);
  }
  // Move tab right: Cmd + Alt + Right
  else if (key === "ArrowRight" && alt) {
    if (state.activePartition) moveTabByPartition(state.activePartition, 1);
  }
  // Back: Cmd + [
  else if (k === "[") {
    if (state.activePartition) invoke("nav_back", { partition: state.activePartition });
  }
  // Forward: Cmd + ]
  else if (k === "]") {
    if (state.activePartition) invoke("nav_forward", { partition: state.activePartition });
  }
  // Toggle help: Cmd + /
  else if (k === "/") {
    toggleShortcutsModal();
  }
  // Zoom
  else if (k === "=" || k === "+") {
    changeZoom(0.1);
  } else if (k === "-") {
    changeZoom(-0.1);
  } else if (k === "0") {
    resetZoom();
  }
}

// Global Keydown Handler (When main window is focused)
window.addEventListener("keydown", (e) => {
  resetInactivityTimer();

  // Lock overlay active
  if (state.isLocked) {
    if (e.key >= "0" && e.key <= "9") {
      handlePinInput(e.key);
    } else if (e.key === "Backspace") {
      state.pinBuffer = state.pinBuffer.slice(0, -1);
      updatePinDots();
    } else if (e.key === "Escape") {
      state.pinBuffer = "";
      updatePinDots();
    }
    return;
  }

  // Escape closes any open modal
  if (e.key === "Escape") {
    hideModal(modalNewSession);
    hideModal(modalManageSessions);
    hideModal(modalShortcuts);
    hideModal(modalAbout);
    hideModal(modalSettings);
    return;
  }

  const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
  const mod = isMac ? e.metaKey : e.ctrlKey;

  if (mod) {
    e.preventDefault();
    executeShortcut(e.key, { alt: e.altKey, shift: e.shiftKey });
  }
});

// Listen to Native macOS Application Menu Accelerators (Fires from Cocoa on ANY website!)
listen("menu-shortcut", (event) => {
  const id = event.payload;
  if (!id) return;

  if (id.startsWith("tab_")) {
    const idx = parseInt(id.replace("tab_", ""), 10) - 1;
    if (state.bookmarks[idx]) {
      activateSession(state.bookmarks[idx].partition, state.bookmarks[idx].url);
    }
  } else if (id === "new_session") {
    openNewSessionModal();
  } else if (id === "close_session") {
    if (state.activePartition) {
      removeSession(state.activePartition);
    }
  } else if (id === "manage_sessions") {
    openManageSessionsModal();
  } else if (id === "lock_now") {
    lockApp();
  } else if (id === "reload") {
    if (state.activePartition) {
      invoke("nav_reload", { partition: state.activePartition });
    }
  } else if (id === "home") {
    goHome();
  } else if (id === "focus_url") {
    urlPill.click();
  } else if (id === "shortcuts_help") {
    toggleShortcutsModal();
  } else if (id === "move_tab_left") {
    if (state.activePartition) moveTabByPartition(state.activePartition, -1);
  } else if (id === "move_tab_right") {
    if (state.activePartition) moveTabByPartition(state.activePartition, 1);
  }
});

// Global Keydown Handler (Forwarded from child webviews if available)
listen("trigger-shortcut", (event) => {
  if (event.payload) {
    const { key, alt, shift } = event.payload;
    executeShortcut(key, { alt, shift });
  }
});

function changeZoom(delta) {
  state.zoomLevel = Math.max(0.3, Math.min(3.0, state.zoomLevel + delta));
  if (state.activePartition) {
    invoke("set_zoom", { partition: state.activePartition, factor: state.zoomLevel });
  }
}

function resetZoom() {
  state.zoomLevel = 1.0;
  if (state.activePartition) {
    invoke("set_zoom", { partition: state.activePartition, factor: 1.0 });
  }
}

// Activity resets
["mousemove", "mousedown", "touchstart", "scroll"].forEach((evt) => {
  window.addEventListener(evt, resetInactivityTimer, { passive: true });
});

// Close modals on backdrop click
[modalNewSession, modalManageSessions, modalShortcuts, modalAbout, modalSettings].forEach((m) => {
  m.addEventListener("click", (e) => {
    if (e.target === m) hideModal(m);
  });
});

// Listen for navigation events from child webviews
listen("session-navigated", (event) => {
  if (event.payload.partition === state.activePartition) {
    urlDomain.textContent = getDomain(event.payload.url);
    urlInput.value = event.payload.url;
  }
});

// === Initialize ===
async function init() {
  console.log("Initializing MiniBrowser Rust Edition...");
  try {
    const rawState = await invoke("get_state");
    if (rawState) {
      state.settings = rawState.settings || state.settings;
      state.bookmarks = rawState.bookmarks || [];
    }

    renderTabs();

    // Only lock immediately on launch if explicitly configured by the user
    if (state.settings.lockOnLaunch) {
      lockApp();
    } else if (state.bookmarks.length > 0) {
      const first = state.bookmarks[0];
      activateSession(first.partition, first.url);
    } else {
      goHome();
    }

    resetInactivityTimer();
  } catch (err) {
    console.error("Initialization error:", err);
  }
}

init();
