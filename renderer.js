let state = { settings: null, bookmarks: [] };
let activeFavPartition = null;

const webviewsEl = document.getElementById("webviews");
const urlEl = document.getElementById("url");
const backEl = document.getElementById("back");
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

// === Core ===
function activateFavorite(partition) {
  activeFavPartition = partition;
  document.querySelectorAll("webview").forEach((w) => {
    if (w.getAttribute("partition") === partition) {
      w.classList.add("active");
      urlEl.value = w.getURL();
      urlEl.setAttribute("readonly", "true");
      urlEl.style.display = "block";
      document.title = w.getTitle() || "MiniBrowser";
    } else {
      w.classList.remove("active");
    }
  });
  renderBookmarks();
}

async function createWebview(fav) {
  await electronAPI.ensureSession(fav.partition);

  const w = document.createElement("webview");
  w.setAttribute("partition", fav.partition);
  w.setAttribute("allowpopups", "true");
  w.src = fav.url;
  w.className = fav.partition === activeFavPartition ? "active" : "";

  w.addEventListener("page-title-updated", () => {
    if (fav.partition === activeFavPartition) updateUrlInput(w.getURL());
  });

  w.addEventListener("did-navigate", () => {
    if (fav.partition === activeFavPartition) updateUrlInput(w.getURL());
  });

  webviewsEl.appendChild(w);
  fav.webview = w;
}

function renderBookmarks() {
  bookmarksEl.innerHTML = "";
  (state.bookmarks || []).forEach((b) => {
    const wrapper = document.createElement("div");

    const btn = document.createElement("button");
    btn.textContent = b.title || b.url;
    btn.title = b.partition;
    if (b.partition === activeFavPartition) {
      btn.style.outline = "2px solid #2a7fde";
    }
    btn.addEventListener("click", () => activateFavorite(b.partition));

    const del = document.createElement("button");
    del.textContent = "×";
    del.title = "Eliminar favorito";
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
          document.title = "MiniBrowser";
          urlEl.value = "";
          urlEl.style.display = "none";
        }
      }
    });

    wrapper.appendChild(btn);
    wrapper.appendChild(del);
    bookmarksEl.appendChild(wrapper);
  });
}

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
    urlEl.value = "";
    urlEl.style.display = "none";
  }

  renderBookmarks();
}

backEl.addEventListener("click", goBack);
fwdEl.addEventListener("click", goFwd);
reloadEl.addEventListener("click", reload);

newFavEl.addEventListener("click", () => {
  urlEl.style.display = "block";
  urlEl.removeAttribute("readonly");
  urlEl.value = "";
  urlEl.focus();
});

urlEl.addEventListener("keydown", async (e) => {
  if (e.key === "Enter" && !urlEl.hasAttribute("readonly")) {
    await addBookmarkFromUrl(urlEl.value);
    urlEl.value = "";
    urlEl.setAttribute("readonly", "true");
    urlEl.style.display = "none";
  }
});

init();
