# MiniBrowser (Electron)

🚀 **MiniBrowser** es un navegador ligero y personalizado construido con **Electron** diseñado para privacidad y productividad. Permite abrir **sesiones independientes por favorito**, reorganizar favoritos con **drag & drop** y bloquear la ventana con un **PIN seguro** tras períodos de inactividad.

[![Release](https://img.shields.io/github/release/devCodr/mini-browser.svg)](https://github.com/devCodr/mini-browser/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Electron](https://img.shields.io/badge/Electron-41.3.0-blue.svg)](https://electronjs.org/)

## ✨ Características Principales

- 🔐 **Sesiones aisladas** por favorito con persistencia separada (`persist:...`)
- 🎯 **Drag & Drop** para reorganizar favoritos intuitivamente
- 🔒 **Bloqueo automático** por inactividad con PIN personalizable
- 💾 **Persistencia de datos** local (configuración y favoritos)
- 🌐 **Integración nativa** - abrir enlaces en navegador predeterminado
- 🎨 **Interfaz minimalista** y enfocada en la productividad

## 🚀 Descargas

### 📦 Releases Automáticos

Descarga la versión más reciente para tu plataforma:

👉 **[Ver todos los releases](https://github.com/devCodr/mini-browser/releases)** - Selecciona el archivo adecuado para tu sistema:

- **macOS**: `MiniBrowser-*.dmg` (Apple Silicon M1/M2/M3+)
- **Linux**: `mini-browser_*.deb` (Debian/Ubuntu) o `MiniBrowser-*.AppImage`
- **Windows**: `MiniBrowser Setup *.exe`

> 📋 **Nota**: Los releases se generan automáticamente con cada nuevo tag en el repositorio. Siempre descarga la última versión disponible.

## ▶️ Uso Rápido

### Para Usuarios Finales

1. Descarga el instalador adecuado para tu sistema desde [releases](https://github.com/devCodr/mini-browser/releases)
2. Instala y ejecuta MiniBrowser
3. **➕ Añadir favorito**: Escribe una URL y presiona Enter
4. **🔄 Reorganizar**: Arrastra los favoritos para cambiar su orden
5. **🔐 Bloqueo manual**: Menú → _Lock now_

### Para Desarrolladores

```bash
# Clonar repositorio
git clone https://github.com/devCodr/mini-browser.git
cd mini-browser

# Instalar dependencias
pnpm install

# Ejecutar en modo desarrollo
pnpm start
```

## 🛠️ Build y Distribución

### Build Completo (Multi-plataforma)

```bash
pnpm run build
```

### Build Específico por Plataforma

```bash
# macOS (Apple Silicon optimizado)
pnpm run package:mac

# Linux
pnpm run package:linux

# Windows
pnpm run package:windows
```

### Generar Iconos

```bash
pnpm run make:icon
```

## 🗂️ Arquitectura del Proyecto

```
mini-browser/
├── main.js              # Proceso principal Electron, menús, overlay PIN
├── renderer.js          # UI principal, gestión de favoritos, drag & drop
├── preload.js           # Preload script para seguridad
├── overlay.html         # Interfaz de bloqueo PIN
├── welcome.html         # Pantalla de bienvenida inicial
├── webview-preload.js   # Preload para webviews
├── overlay-preload.js   # Preload para overlay
├── package.json         # Configuración del proyecto y scripts
└── build/               # Recursos de build (iconos, etc.)
```

## 🔐 Configuración de Seguridad

### PIN y Bloqueo

- **Tiempo de inactividad**: 5 minutos (configurable en `settings.json`)
- **PIN por defecto**: `123456` (cambiable desde el overlay)
- **Almacenamiento seguro**: Hash del PIN almacenado localmente

### Datos Persistentes

Los datos se guardan en el directorio de usuario de la aplicación:

- `settings.json`: Preferencias, configuración de PIN y tiempo de inactividad
- `bookmarks.json`: Lista completa de favoritos con metadatos

## 🔄 Flujo de Trabajo de Releases

El proyecto utiliza **GitHub Actions** para generar releases automáticos optimizados:

1. **Trigger**: Creación de un nuevo tag (ej: `v1.0.1`)
2. **Build**: Compilación multi-plataforma con `electron-builder`
   - **macOS**: Apple Silicon (arm64) optimizado para rendimiento nativo
   - **Linux**: x64 compatible con distribuciones modernas
   - **Windows**: x64 con instalador NSIS
3. **Upload**: Generación de assets optimizados para cada plataforma
4. **Release**: Creación automática del GitHub Release

### Crear Nuevo Release

```bash
# Crear nuevo tag
git tag v1.0.1
git push origin v1.0.1

# El workflow se ejecutará automáticamente
```

## 🤝 Contribución

¡Las contribuciones son bienvenidas! Por favor:

1. Fork el repositorio
2. Crea una rama para tu feature (`git checkout -b feature/amazing-feature`)
3. Commit tus cambios (`git commit -m 'Add amazing feature'`)
4. Push a la rama (`git push origin feature/amazing-feature`)
5. Abre un Pull Request

## 📄 Licencia

Este proyecto está bajo la **Licencia MIT** - ver el archivo [LICENSE](LICENSE) para detalles.

© 2025 [Chris Larico](https://larico.dev) - [GitHub](https://github.com/devCodr)
