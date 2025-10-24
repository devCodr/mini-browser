# MiniBrowser (Electron)

Pequeño navegador personalizado construido con **Electron** que permite
abrir **sesiones independientes por favorito**, reordenar favoritos con
**drag & drop** y bloquear la ventana con un **PIN/overlay** luego de
inactividad.

## ✨ Características

- Sesiones dedicadas por favorito (`persist:...`).
- Reordenar favoritos arrastrando (drag & drop).
- Bloqueo por inactividad con PIN (default `123456`).
- Persistencia de datos en disco (settings + bookmarks).
- Menú contextual en webview: **Open in default browser**.

## ▶️ Uso

- **➕ Crear favorito**: escribe URL y presiona Enter.
- **Arrastrar favoritos** para cambiar orden.
- Navegación con ⟵ ⟶ ⟳ solo afecta a la pestaña activa.
- Bloqueo manual: Menú → _Lock now_.

## 📦 Instalación

```bash
npm i
npm start
```

Build distribución:

```bash
npm run build
```

Build para mac solamente

```bash
npm run package:mac
```

## 🗂️ Archivos clave

- `main.js`: proceso principal, menús, overlay PIN.
- `renderer.js`: UI, favoritos y drag & drop.
- `overlay.html`: pantalla de bloqueo.
- `welcome.html`: bienvenida inicial.

## 🔐 PIN

- Inactividad: 5 min (editable en `settings.json`).
- PIN por defecto: `123456`.
- Cambiable desde el overlay.

## 💾 Datos

Guardados en `app.getPath("userData")`: - `settings.json`: preferencias
y PIN hash.

- `bookmarks.json`: lista de favoritos.

## 📄 Licencia

MIT © 2025 Chris Larico
