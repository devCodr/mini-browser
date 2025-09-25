# MiniBrowser (Electron)

Pequeño navegador personalizado construido con **Electron** que permite abrir **sesiones independientes por favorito** (aisladas por `partition`) y bloquear la ventana con un **PIN/overlay** luego de un período de inactividad.

> **Estado del proyecto:** mínimamente funcional para navegación y favoritos persistentes. En los siguientes pasos iremos puliendo menú, webview (menú contextual y abrir enlaces en el navegador del sistema), etiquetas abreviadas de favoritos y el cuadro **About**.

---

## ✨ Características

- **Favoritos con sesión dedicada**: cada favorito se abre en su propio `webview` y `session` persistente (`persist:...`), ideal para logins separados.
- **Bloqueo por inactividad**: overlay que solicita **PIN** (por defecto `123456`) pasado un tiempo. Se puede forzar el bloqueo desde el menú.
- **Persistencia de datos**: ajustes y favoritos se guardan bajo el directorio de usuario de la app.
- **UI simple**: barra con navegación (atrás/adelante/recargar), botón **➕** para crear nuevo favorito y botones de favoritos a la derecha.

> Próximas mejoras (se implementarán en los puntos 2–5 de tu lista):
>
> - Renombrar el título/menú de la ventana a **“MiniBrowser”**.
> - Menú contextual del `webview` con **Open in default browser** para enlaces.
> - Etiquetas de favoritos de **dos letras** (`fb`, `yt`, `fb2`, …).
> - Ubicación y edición del popup **About**.

---

## 📦 Requisitos

- **Node.js 18+** recomendado
- **npm** (o pnpm/yarn si prefieres, ajustando comandos)
- SO soportados por Electron

---

## 🛠️ Instalación

1. Clona o copia el proyecto en una carpeta vacía.
2. Instala dependencias:
   ```bash
   npm i
   ```

## ▶️ Modo desarrollo

```bash
npm start
```

Esto lanza Electron y abre la ventana principal.

## 🚀 Build (distribución)

```bash
npm run build
```

Genera artefactos con **electron-builder** (ver configuración en `package.json`).

---

## 🗺️ Estructura principal

- `main.js`: proceso principal de Electron (ventana, menús, overlay de PIN, IPC y persistencia).
- `preload.js`: expone API segura al renderer (bookmarks, sesiones, lock, etc.).
- `index.html`: UI principal (toolbar, contenedor de `webview`s).
- `renderer.js`: lógica de UI en la ventana (crear `webview`, navegación, favoritos).
- `overlay.html` + `overlay-preload.js`: overlay de bloqueo por PIN.
- `package.json`: metadatos, scripts y configuración de build.
- `icon.png`: icono (Linux).

---

## 🧭 Uso

### Crear un favorito

1. Pulsa **➕**.
2. En el input, pega o escribe la URL (se acepta sin `https://`).
3. Presiona **Enter** para crear el favorito.
4. El favorito se abre en un `webview` con su **partition** dedicada (por dominio).

> Actualmente el botón del favorito muestra la URL completa. En el **punto 4** cambiaremos a etiquetas de **dos letras** auto‑generadas (p. ej., `fb`, `yt`, `gh`, y si se repite dominio: `fb2`, `fb3`, …).

### Navegación

- **⟵** Atrás, **⟶** Adelante, **⟳** Recargar afectan solo al `webview` activo.
- El input de URL está **solo lectura** mientras navegas; se usa únicamente para **crear** un favorito nuevo.

---

## 🔐 Bloqueo por PIN

- **Tiempo de inactividad** por defecto: **5 minutos**.
- **PIN por defecto**: `123456`. Al primer bloqueo puedes establecer uno nuevo.
- Para bloquear manualmente: Menú de la app → **Lock now**.
- Para habilitar/deshabilitar el lock: Menú de la app → **Enable/Disable Lock**.

> Los hashes y la configuración se guardan en disco (ver sección de persistencia).

---

## 💾 Persistencia y rutas

Archivos almacenados en `app.getPath("userData")` (varía según SO):

- `settings.json`: preferencias, `pinSalt`/`pinHash`, `inactivityMs`, etc.
- `bookmarks.json`: lista de favoritos `{ title, url, partition }`.
- `pinned.json`: pestañas/favoritos fijados (si se usa).

> Si quieres “resetear” el PIN, puedes borrar `settings.json` con la app cerrada (perderás ese ajuste).

---

## 🧩 Personalización / Desarrollo

- **Menú de la app**: se construye en `createMainMenu()` dentro de `main.js`.
- **Overlay**: contenido en `overlay.html` (estilos y texto), APIs en `overlay-preload.js`.
- **Etiquetas de favoritos**: generación y renderizado en `renderer.js` (funciones `domainSlugFromUrl`, `nextPartitionForDomain` y `renderBookmarks`).

---

## 🐞 Troubleshooting

- Si ves un `webview` en negro, asegúrate de **crear la sesión** antes de insertarlo (ya se hace con `ensureSession(partition)`).
- CSP: `index.html` incluye una CSP simple; evita inyectar scripts externos sin ajustar la política.
- DevTools: Menú → View → **Toggle Developer Tools**.

---

## ❓ FAQ

**¿Dónde cambio el tiempo de inactividad?**  
En tiempo de ejecución vía IPC `settings:setInactivity` o editando `settings.json` con la app cerrada.

**¿Dónde modificaré el “About”?**  
En el **punto 5** te indico exactamente dónde y cómo (menú/role `about` y alternativas).

---

## 📄 Licencia

MIT © 2025 Chris Larico
