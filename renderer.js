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

// encuentra el siguiente partition libre: persist:dominio, persist:dominio2, ...
function nextPartitionForDomain(domain) {
  // persist:domain, persist:domain2, persist:domain3...
  let max = 0;
  const rx = new RegExp(`^persist:${domain}(\\d+)?$`);
  for (const b of state.bookmarks) {
    const m = b.partition && b.partition.match(rx);
    if (m) {
      const n = m[1] ? parseInt(m[1], 10) : 1; // sin sufijo = 1
      if (!isNaN(n) && n > max) max = n;
    }
  }
  // si max==0 => ninguno, devolvemos persist:domain
  // si max>=1 => devolvemos persist:domain{max+1}
  return `persist:${domain}${max >= 1 ? max + 1 : ""}`;
}

// === Core ===
function activateFavorite(partition) {
  activeFavPartition = partition;
  document.querySelectorAll("webview").forEach((w) => {
    if (w.getAttribute("partition") === partition) {
      w.classList.add("active");
      urlEl.value = w.getURL(); // mostrar URL del favorito
      urlEl.setAttribute("readonly", "true"); // bloquear edición
      urlEl.style.display = "block"; // visible pero de solo lectura
      document.title = w.getTitle() || "Mini Browser";
    } else {
      w.classList.remove("active");
    }
  });
  renderBookmarks();
}

async function createWebview(fav) {
  // 🔹 garantizar sesión del partition (evita pantalla negra)
  await api.ensureSession(fav.partition);

  const w = document.createElement("webview");
  w.setAttribute("partition", fav.partition);
  w.setAttribute("allowpopups", "true");
  w.setAttribute("preload", "");
  w.src = fav.url;
  w.className = fav.partition === activeFavPartition ? "active" : "";

  w.addEventListener("page-title-updated", () => {
    if (fav.partition === activeFavPartition) {
      document.title = w.getTitle() || "Mini Browser";
    }
  });
  w.addEventListener("did-navigate", () => {
    if (fav.partition === activeFavPartition) urlEl.value = w.getURL();
    api.signalActivity();
  });
  w.addEventListener("did-navigate-in-page", () => api.signalActivity());
  w.addEventListener("dom-ready", () => api.signalActivity());
  w.addEventListener("ipc-message", () => api.signalActivity());

  webviewsEl.appendChild(w);
  fav.webview = w;
}

function renderBookmarks() {
  bookmarksEl.innerHTML = "";
  (state.bookmarks || []).forEach((b) => {
    const wrapper = document.createElement("div");

    const btn = document.createElement("button");
    btn.textContent = b.title || b.url;
    btn.title = b.partition; // tooltip con persist:xxx
    if (b.partition === activeFavPartition) {
      btn.style.outline = "2px solid #2a7fde";
    }
    btn.addEventListener("click", () => activateFavorite(b.partition));

    const del = document.createElement("button");
    del.textContent = "×";
    del.title = "Eliminar favorito";
    del.addEventListener("click", async (e) => {
      e.stopPropagation();

      // 🔹 borrar por partition (no por URL) para no afectar duplicados del mismo dominio
      const updated = await api.removeBookmark({
        partition: b.partition,
        url: b.url,
      });
      state.bookmarks = updated;
      renderBookmarks();

      // cerrar su webview
      const w = document.querySelector(`webview[partition="${b.partition}"]`);
      if (w) w.remove();

      if (activeFavPartition === b.partition) {
        const next = state.bookmarks[0];
        if (next) activateFavorite(next.partition);
        else {
          activeFavPartition = null;
          document.title = "Mini Browser";
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

// Crear favorito desde URL (solo cuando aparece el input)
async function addBookmarkFromUrl(rawUrl) {
  let url = rawUrl.trim();
  if (!url) return;
  if (!/^https?:\/\//i.test(url)) url = "https://" + url;

  const domain = domainSlugFromUrl(url);
  const partition = nextPartitionForDomain(domain);

  const list = await api.addBookmark({
    title: url, // puedes cambiar a un nombre más friendly si quieres
    url,
    partition,
  });
  state.bookmarks = list;
  renderBookmarks();

  // crear webview y activar
  await createWebview({ url, partition });
  activateFavorite(partition);
}

// Controles de navegación (del webview activo)
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

// Init
async function init() {
  state = await api.getState();

  // montar webviews existentes
  for (const b of state.bookmarks || []) {
    await createWebview(b);
  }

  if (state.bookmarks.length > 0) {
    activateFavorite(state.bookmarks[0].partition);
  } else {
    // sin favoritos: ocultar URL (se mostrará solo al crear)
    urlEl.value = "";
    urlEl.style.display = "none";
  }

  renderBookmarks();
}

// Eventos UI
backEl.addEventListener("click", goBack);
fwdEl.addEventListener("click", goFwd);
reloadEl.addEventListener("click", reload);

// Mostrar input para crear nuevo favorito
newFavEl.addEventListener("click", () => {
  urlEl.style.display = "block";
  urlEl.removeAttribute("readonly"); // permitir escribir SOLO en este modo
  urlEl.value = "";
  urlEl.focus();
});

// Crear favorito al presionar Enter (solo cuando NO está readonly)
urlEl.addEventListener("keydown", async (e) => {
  if (e.key === "Enter" && !urlEl.hasAttribute("readonly")) {
    await addBookmarkFromUrl(urlEl.value);
    urlEl.value = "";
    urlEl.setAttribute("readonly", "true");
    urlEl.style.display = "none";
  }
});

init();
