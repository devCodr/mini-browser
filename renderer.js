let state = { settings: null, bookmarks: [] };
let activeFavPartition = null;

const zoomByPartition = {};
const webviewsEl = document.getElementById("webviews");
const urlEl = document.getElementById("url");
const backEl = document.getElementById("back");
const homeEl = document.getElementById("home");
const fwdEl = document.getElementById("fwd");
const reloadEl = document.getElementById("reload");
const newFavEl = document.getElementById("newFav"); // botón ➕
const bookmarksEl = document.getElementById("bookmarks");

// === Helpers ===
function domainSlugFromUrl(url) {
  try {
    return new URL(url).hostname.replace(/\W+/g, "_");
  } catch {
    return "site";
  }
}

function updateUrlInput(url) {
  urlEl.value = url;
}
function safeGetURL(w) {
  try {
    return w.getURL();
  } catch {
    return w?.src || "";
  }
}

function nextPartitionForDomain(domain) {
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

// 🔹 Generar etiqueta corta de 2 letras + numeración
function shortLabelFromUrl(url, existingLabels) {
  try {
    const hostname = new URL(url).hostname.replace(/^www\./, "");
    const parts = hostname.split(".");
    let base = parts[0].substring(0, 2).toLowerCase();

    let label = base;
    let counter = 2;
    while (existingLabels.includes(label)) {
      label = base + counter;
      counter++;
    }
    return label;
  } catch {
    return "??";
  }
}

// === Core ===
function activateFavorite(partition) {
  activeFavPartition = partition;
  let activeWebview = null;
  document.querySelectorAll("webview").forEach((w) => {
    if (w.getAttribute("partition") === partition) {
      w.classList.add("active");
      activeWebview = w;
      urlEl.value = safeGetURL(w);
      // urlEl.setAttribute("readonly", "true");
      urlEl.style.display = "block";
      try {
        document.title = w.getTitle() || "MiniBrowser";
      } catch {
        document.title = "MiniBrowser";
      }

      const z = zoomByPartition[partition] ?? 1;
      try {
        w.setZoomFactor(z);
      } catch {}
    } else {
      w.classList.remove("active");
    }
  });

  // focus the active webview so keyboard shortcuts (Cmd+C/Cmd+A) work
  if (activeWebview) {
    try {
      activeWebview.setAttribute("tabindex", "0");
      activeWebview.focus();
    } catch (e) {
      // ignore
    }
  }

  renderBookmarks();
}

async function createWebview(fav) {
  await electronAPI.ensureSession(fav.partition);

  const w = document.createElement("webview");
  w.setAttribute("partition", fav.partition);
  w.setAttribute("allowpopups", "true");
  w.setAttribute("tabindex", "0");
  w.src = fav.url;
  w.className = fav.partition === activeFavPartition ? "active" : "";

  w.addEventListener("page-title-updated", () => {
    if (fav.partition === activeFavPartition) updateUrlInput(safeGetURL(w));
  });

  w.addEventListener("did-navigate", () => {
    if (fav.partition === activeFavPartition) updateUrlInput(safeGetURL(w));
  });

  w.addEventListener("dom-ready", () => {
    if (fav.partition === activeFavPartition) updateUrlInput(safeGetURL(w));
    const z = zoomByPartition[fav.partition] ?? 1;
    try {
      w.setZoomFactor(z);
    } catch {}
  });

  webviewsEl.appendChild(w);
  fav.webview = w;
}
function getDomainFromUrl(url) {
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return url;
  }
}

function renderBookmarks() {
  bookmarksEl.innerHTML = "";

  (state.bookmarks || []).forEach((b, idx) => {
    const wrapper = document.createElement("div");
    wrapper.className = "bookmark";
    wrapper.draggable = true; // 🔹 habilitar arrastrar
    wrapper.dataset.index = idx;

    // drag start
    wrapper.addEventListener("dragstart", (e) => {
      e.dataTransfer.setData("text/plain", idx);
      wrapper.classList.add("dragging");
    });
    wrapper.addEventListener("dragend", () => {
      wrapper.classList.remove("dragging");
    });

    // drag over
    wrapper.addEventListener("dragover", (e) => {
      e.preventDefault();
      const draggingEl = bookmarksEl.querySelector(".dragging");
      if (!draggingEl) return;
      const bounding = wrapper.getBoundingClientRect();
      const offset = e.clientX - bounding.left;
      const before = offset < bounding.width / 2;
      if (before) {
        bookmarksEl.insertBefore(draggingEl, wrapper);
      } else {
        bookmarksEl.insertBefore(draggingEl, wrapper.nextSibling);
      }
    });

    // drop
    wrapper.addEventListener("drop", async () => {
      const newOrder = Array.from(bookmarksEl.children).map((el) => {
        const i = el.dataset.index;
        return state.bookmarks[i];
      });
      state.bookmarks = await electronAPI.reorderBookmarks(newOrder);
      renderBookmarks();
    });

    // === Botón principal con favicon ===
    const btn = document.createElement("button");
    const domain = getDomainFromUrl(b.url);
    const faviconUrl = `https://www.google.com/s2/favicons?sz=32&domain=${domain}`;

    if (b.iconSvg) {
      // insert raw SVG (assume trusted since user pasted it)
      const wrap = document.createElement("span");
      wrap.style.display = "inline-block";
      wrap.style.width = "32px";
      wrap.style.height = "32px";
      wrap.style.lineHeight = "0";
      wrap.innerHTML = b.iconSvg;
      // ensure the svg fits
      const sv = wrap.querySelector("svg");
      if (sv) {
        sv.setAttribute("width", "32");
        sv.setAttribute("height", "32");
        sv.style.display = "block";
      }
      btn.appendChild(wrap);
    } else {
      const img = document.createElement("img");
      img.src = faviconUrl;
      img.alt = domain;
      img.width = 32;
      img.height = 32;
      btn.appendChild(img);
    }
    btn.title = `${domain} | ${b.partition}`;
    if (b.partition === activeFavPartition) btn.classList.add("active");
    btn.addEventListener("click", () => activateFavorite(b.partition));

    const del = document.createElement("button");
    del.textContent = "×";
    del.title = "Remove Bookmark";
    del.addEventListener("click", async (e) => {
      e.stopPropagation();
      const updated = await electronAPI.removeBookmark({
        partition: b.partition,
        url: b.url,
      });
      state.bookmarks = updated;
      renderBookmarks();
      const w = document.querySelector(`webview[partition="${b.partition}"]`);
      if (w) w.remove();
      if (activeFavPartition === b.partition) {
        const next = state.bookmarks[0];
        if (next) activateFavorite(next.partition);
        else {
          activeFavPartition = null;
          const partition = "persist:welcome";
          createWebview({ url: "welcome.html", partition }).then(() => {
            activateFavorite(partition);
            urlEl.style.display = "none";
          });
        }
      }
    });

    // change icon button
    const changeIconBtn = document.createElement("button");
    changeIconBtn.textContent = "✎";
    changeIconBtn.title = "Change Icon";
    changeIconBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      openIconModal(b.partition, b.iconSvg || "");
    });

    wrapper.appendChild(btn);
    wrapper.appendChild(changeIconBtn);
    wrapper.appendChild(del);
    bookmarksEl.appendChild(wrapper);
  });
}

// --- Icon modal logic ---
const iconModal = document.getElementById("iconModal");
const iconSvgInput = document.getElementById("iconSvgInput");
const iconCancel = document.getElementById("iconCancel");
const iconSave = document.getElementById("iconSave");
let iconEditingPartition = null;

function openIconModal(partition, existingSvg) {
  iconEditingPartition = partition;
  iconSvgInput.value = existingSvg || "";
  iconModal.style.display = "flex";
  iconSvgInput.style.width = "100%";
  iconSvgInput.style.height = "160px";
  iconSvgInput.focus();
}

function closeIconModal() {
  iconEditingPartition = null;
  iconModal.style.display = "none";
}

iconCancel.addEventListener("click", () => closeIconModal());

iconSave.addEventListener("click", async () => {
  if (!iconEditingPartition) return closeIconModal();
  const raw = iconSvgInput.value.trim();

  // basic check: must contain <svg
  if (raw && !/<svg[\s>]/i.test(raw)) {
    alert("Please paste valid SVG markup starting with <svg>");
    return;
  }

  // Save via electronAPI
  const updated = await electronAPI.updateBookmarkIcon({
    partition: iconEditingPartition,
    iconSvg: raw,
  });

  state.bookmarks = updated;
  renderBookmarks();
  closeIconModal();
});

async function addBookmarkFromUrl(rawUrl) {
  let url = rawUrl.trim();
  if (!url) return;
  if (!/^https?:\/\//i.test(url)) url = "https://" + url;

  const domain = domainSlugFromUrl(url);
  const partition = nextPartitionForDomain(domain);

  const list = await electronAPI.addBookmark({
    title: url,
    url,
    partition,
  });
  state.bookmarks = list;
  renderBookmarks();

  await createWebview({ url, partition });
  activateFavorite(partition);
}

function currentWebview() {
  return document.querySelector(`webview[partition="${activeFavPartition}"]`);
}
function goBack() {
  const w = currentWebview();
  if (w && w.canGoBack()) w.goBack();
}
function goFwd() {
  const w = currentWebview();
  if (w && w.canGoForward()) w.goForward();
}
function reload() {
  const w = currentWebview();
  if (w) w.reload();
}

async function init() {
  state = await electronAPI.getState();

  for (const b of state.bookmarks || []) {
    await createWebview(b);
  }

  if (state.bookmarks.length > 0) {
    activateFavorite(state.bookmarks[0].partition);
  } else {
    // mostrar bienvenida en un webview no listado en bookmarks
    const partition = "persist:welcome";
    await createWebview({ url: "welcome.html", partition });
    activateFavorite(partition);
    urlEl.style.display = "none";
  }

  renderBookmarks();
}

backEl.addEventListener("click", goBack);
fwdEl.addEventListener("click", goFwd);
reloadEl.addEventListener("click", reload);

newFavEl.addEventListener("click", () => {
  urlEl.style.display = "block";
  // urlEl.removeAttribute("readonly");
  urlEl.value = "";
  urlEl.focus();
});

urlEl.addEventListener("keydown", async (e) => {
  if (e.key === "Enter" && !urlEl.hasAttribute("readonly")) {
    await addBookmarkFromUrl(urlEl.value);
    urlEl.value = "";
    // urlEl.setAttribute("readonly", "true");
    urlEl.style.display = "none";
  }
});

init();

// === Home button ===
async function goHome() {
  // look for an existing welcome partition
  const welcomePartition = "persist:welcome";
  const existing = document.querySelector(
    `webview[partition="${welcomePartition}"]`
  );
  if (existing) {
    activateFavorite(welcomePartition);
    urlEl.style.display = "none";
    return;
  }

  // create and activate a welcome webview
  await createWebview({ url: "welcome.html", partition: welcomePartition });
  activateFavorite(welcomePartition);
  urlEl.style.display = "none";
}

homeEl.addEventListener("click", goHome);

// === Zoom desde el menú ===
electronAPI.onZoom((dir) => {
  if (!activeFavPartition) return;

  let z = zoomByPartition[activeFavPartition] ?? 1;
  if (dir === "in") z = Math.min(3, +(z + 0.1).toFixed(2));
  if (dir === "out") z = Math.max(0.25, +(z - 0.1).toFixed(2));
  if (dir === "reset") z = 1;

  zoomByPartition[activeFavPartition] = z;

  const w = document.querySelector(
    `webview[partition="${activeFavPartition}"]`
  );
  if (w) {
    try {
      w.setZoomFactor(z);
    } catch {}
  }
});

// === Reload desde el menú ===
electronAPI.onTabReload(() => {
  reload(); // recarga solo el webview activo; no cambia de favorito
});
