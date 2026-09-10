use crate::store::StoreManager;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Rect, WebviewBuilder, WebviewUrl,
};

pub const HEADER_HEIGHT: f64 = 42.0;

#[cfg(target_os = "macos")]
pub const CHROME_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";

#[cfg(target_os = "macos")]
pub const APPLE_WEBKIT_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.3 Safari/605.1.15";

#[cfg(not(target_os = "macos"))]
pub const CHROME_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";

#[cfg(not(target_os = "macos"))]
pub const APPLE_WEBKIT_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.3 Safari/605.1.15";

pub const USER_AGENT: &str = CHROME_USER_AGENT;

pub fn is_google_service(url: &str, partition: &str) -> bool {
    let url_lower = url.to_lowercase();
    let part_lower = partition.to_lowercase();
    url_lower.contains("google.")
        || url_lower.contains("gmail.com")
        || url_lower.contains("accounts.google")
        || url_lower.contains("mail.google")
        || url_lower.contains("gstatic.com")
        || url_lower.contains("googleusercontent.com")
        || part_lower.contains("google")
        || part_lower.contains("gmail")
}

pub fn get_user_agent_for_session(url: &str, partition: &str) -> &'static str {
    if is_google_service(url, partition) {
        APPLE_WEBKIT_USER_AGENT
    } else {
        CHROME_USER_AGENT
    }
}

pub fn partition_to_uuid_bytes(partition: &str) -> [u8; 16] {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(partition.as_bytes());
    let result = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&result[0..16]);
    // Set UUID version 5 (name-based SHA)
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    // Set variant to RFC 4122 (0b10xxxxxx)
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    bytes
}

pub fn get_unique_download_path(suggested_filename: &str) -> std::path::PathBuf {
    let base_dir = dirs::download_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let clean_name = if suggested_filename.trim().is_empty() {
        "descarga"
    } else {
        suggested_filename.trim()
    };
    let file_name = std::path::Path::new(clean_name)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("descarga");

    let mut dest = base_dir.join(file_name);
    if !dest.exists() {
        return dest;
    }

    let (stem, ext) = match file_name.rfind('.') {
        Some(idx) => (&file_name[..idx], &file_name[idx..]),
        None => (file_name, ""),
    };

    let mut counter = 1;
    while dest.exists() {
        dest = base_dir.join(format!("{} ({}){}", stem, counter, ext));
        counter += 1;
    }
    dest
}

pub fn save_base64_file(filename: &str, data_uri_or_base64: &str) -> Result<std::path::PathBuf, String> {
    use base64::Engine as _;
    let raw_b64 = if let Some(idx) = data_uri_or_base64.find(";base64,") {
        &data_uri_or_base64[idx + 8..]
    } else {
        data_uri_or_base64
    };

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(raw_b64.trim())
        .map_err(|e| format!("Base64 decode error: {}", e))?;

    let dest = get_unique_download_path(filename);
    std::fs::write(&dest, bytes).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(dest)
}

pub fn notify_download_finished(app: &AppHandle, file_path: &std::path::Path) {
    let file_name = file_path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("archivo");
    let path_str = file_path.to_string_lossy().to_string();

    let _ = app.emit(
        "download-finished",
        serde_json::json!({
            "name": file_name,
            "path": path_str
        }),
    );

    use tauri_plugin_notification::NotificationExt;
    let _ = app
        .notification()
        .builder()
        .title("Descarga completada")
        .body(format!("Guardado en Descargas: {}", file_name))
        .show();
}

pub struct SessionManager {
    active_label: Mutex<Option<String>>,
    sessions: Mutex<HashMap<String, String>>, // partition -> webview label
}

impl Default for SessionManager {
    fn default() -> Self {
        Self {
            active_label: Mutex::new(None),
            sessions: Mutex::new(HashMap::new()),
        }
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clean_label(partition: &str) -> String {
        let safe: String = partition
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        format!("wv_{}", safe)
    }

    pub fn partition_to_uuid_bytes(partition: &str) -> [u8; 16] {
        partition_to_uuid_bytes(partition)
    }

    pub fn activate_session(
        &self,
        app: &AppHandle,
        partition: &str,
        raw_url: &str,
        store: &StoreManager,
    ) -> Result<String, String> {
        let window = app
            .get_window("main")
            .ok_or_else(|| "Main window not found".to_string())?;

        let label = Self::clean_label(partition);
        let mut active_lock = self.active_label.lock().unwrap();

        // Hide previous active session if different
        if let Some(ref prev_label) = *active_lock {
            if prev_label != &label {
                if let Some(prev_wv) = app.get_webview(prev_label) {
                    let _ = prev_wv.hide();
                }
            }
        }

        let win_size = window
            .inner_size()
            .map_err(|e| format!("Failed to get window size: {}", e))?;
        let scale = window
            .scale_factor()
            .map_err(|e| format!("Failed to get scale: {}", e))?;
        let logical_w = win_size.width as f64 / scale;
        let logical_h = win_size.height as f64 / scale;

        let content_h = (logical_h - HEADER_HEIGHT).max(100.0);
        let bounds = Rect {
            position: LogicalPosition::new(0.0, HEADER_HEIGHT).into(),
            size: LogicalSize::new(logical_w, content_h).into(),
        };

        // Check if webview already exists
        if let Some(existing_wv) = app.get_webview(&label) {
            let _ = existing_wv.set_bounds(bounds);
            let _ = existing_wv.show();
            let _ = existing_wv.set_focus();
            *active_lock = Some(label.clone());
            return Ok(label);
        }

        // Lazy load: create new child Webview for this session
        let formatted_url = if raw_url.starts_with("http://") || raw_url.starts_with("https://") {
            raw_url.to_string()
        } else {
            format!("https://{}", raw_url)
        };

        let parsed_url: url::Url = formatted_url
            .parse()
            .map_err(|e| format!("Invalid URL: {}", e))?;

        let data_dir = store.session_data_dir(partition);
        let part_clone = partition.to_string();
        let app_handle_clone = app.clone();
        let app_handle_new_win = app.clone();
        let app_handle_dl = app.clone();

        let partition_for_notif = partition.to_string();

        let init_script = r##"
            // === Chrome Fingerprint: defeat embedded webview detection by Google, etc. ===
            (function() {
                // 1. Remove webdriver flag
                try { Object.defineProperty(navigator, 'webdriver', { get: () => undefined }); } catch(e) {}

                // Check if current site is Google/Gmail to keep WebKit environment clean
                var isGoogle = false;
                try {
                    var host = window.location.hostname || '';
                    isGoogle = host.indexOf('google.') !== -1 || host.indexOf('gmail.com') !== -1 || host.indexOf('gstatic.com') !== -1;
                } catch(e) {}

                // 2. Inject window.chrome object (Google checks this first; non-Google sites get Chrome object)
                if (!isGoogle && !window.chrome) {
                    var chrome = {
                        app: {
                            isInstalled: false,
                            InstallState: { DISABLED: 'disabled', INSTALLED: 'installed', NOT_INSTALLED: 'not_installed' },
                            RunningState: { CANNOT_RUN: 'cannot_run', READY_TO_RUN: 'ready_to_run', RUNNING: 'running' }
                        },
                        csi: function() { return { startE: Date.now(), onloadT: Date.now(), pageT: Date.now(), tran: 15 }; },
                        loadTimes: function() {
                            return {
                                commitLoadTime: Date.now() / 1000,
                                connectionInfo: 'h2',
                                finishDocumentLoadTime: Date.now() / 1000,
                                finishLoadTime: Date.now() / 1000,
                                firstPaintAfterLoadTime: 0,
                                firstPaintTime: Date.now() / 1000,
                                navigationType: 'Other',
                                npnNegotiatedProtocol: 'h2',
                                requestTime: Date.now() / 1000,
                                startLoadTime: Date.now() / 1000,
                                wasAlternateProtocolAvailable: false,
                                wasFetchedViaSpdy: true,
                                wasNpnNegotiated: true
                            };
                        },
                        runtime: {
                            OnInstalledReason: { CHROME_UPDATE: 'chrome_update', INSTALL: 'install', SHARED_MODULE_UPDATE: 'shared_module_update', UPDATE: 'update' },
                            OnRestartRequiredReason: { APP_UPDATE: 'app_update', OS_UPDATE: 'os_update', PERIODIC: 'periodic' },
                            PlatformArch: { ARM: 'arm', ARM64: 'arm64', MIPS: 'mips', MIPS64: 'mips64', X86_32: 'x86-32', X86_64: 'x86-64' },
                            PlatformNaclArch: { ARM: 'arm', MIPS: 'mips', MIPS64: 'mips64', X86_32: 'x86-32', X86_64: 'x86-64' },
                            PlatformOs: { ANDROID: 'android', CROS: 'cros', LINUX: 'linux', MAC: 'mac', OPENBSD: 'openbsd', WIN: 'win' },
                            RequestUpdateCheckStatus: { NO_UPDATE: 'no_update', THROTTLED: 'throttled', UPDATE_AVAILABLE: 'update_available' },
                            id: undefined
                        }
                    };
                    try {
                        Object.defineProperty(window, 'chrome', { value: chrome, writable: false, enumerable: true, configurable: false });
                    } catch(e) { window.chrome = chrome; }
                }

                // 3. Realistic plugins list (Chrome has these)
                try {
                    var pluginData = [
                        { name: 'Chrome PDF Plugin', filename: 'internal-pdf-viewer', description: 'Portable Document Format', mimeTypes: [{ type: 'application/x-google-chrome-pdf', suffixes: 'pdf', description: 'Portable Document Format' }] },
                        { name: 'Chrome PDF Viewer', filename: 'mhjfbmdgcfjbbpaeojofohoefgiehjai', description: '', mimeTypes: [{ type: 'application/pdf', suffixes: 'pdf', description: '' }] },
                        { name: 'Native Client', filename: 'internal-nacl-plugin', description: '', mimeTypes: [{ type: 'application/x-nacl', suffixes: '', description: 'Native Client Executable' }, { type: 'application/x-pnacl', suffixes: '', description: 'Portable Native Client Executable' }] }
                    ];
                    var pluginArray = pluginData.map(function(p) {
                        var plugin = Object.create(Plugin.prototype);
                        Object.defineProperty(plugin, 'name', { value: p.name });
                        Object.defineProperty(plugin, 'filename', { value: p.filename });
                        Object.defineProperty(plugin, 'description', { value: p.description });
                        return plugin;
                    });
                    Object.defineProperty(navigator, 'plugins', { get: function() { return pluginArray; }, configurable: true });
                    Object.defineProperty(navigator, 'mimeTypes', { get: function() { return []; }, configurable: true });
                } catch(e) {}

                // 4. Languages - real Chrome sends multiple
                try {
                    Object.defineProperty(navigator, 'languages', { get: function() { return ['es-MX', 'es', 'en-US', 'en']; }, configurable: true });
                } catch(e) {}

                // 5. Realistic hardware concurrency and device memory
                try { Object.defineProperty(navigator, 'hardwareConcurrency', { get: () => 8, configurable: true }); } catch(e) {}
                try { if (!navigator.deviceMemory) Object.defineProperty(navigator, 'deviceMemory', { get: () => 8, configurable: true }); } catch(e) {}

                // 6. Patch toString on key functions to look native
                try {
                    var origToString = Function.prototype.toString;
                    Function.prototype.toString = function() {
                        if (this === Function.prototype.toString) return 'function toString() { [native code] }';
                        return origToString.call(this);
                    };
                } catch(e) {}
            })();


            // === Native Notification Bridge ===
            (function() {
                // Grant permission by default so sites don't keep asking
                var _origNotif = window.Notification;
                var _permission = 'granted';

                function MiniBrowserNotification(title, options) {
                    options = options || {};
                    try {
                        sendHostAction('notify', {
                            title: title || '',
                            body: (options.body || '').toString().substring(0, 300),
                            icon: (options.icon || '').toString().substring(0, 500),
                            tag: (options.tag || '').toString().substring(0, 100)
                        });
                    } catch(e) { console.error('Notification bridge error:', e); }
                    // Return a fake object with event handlers
                    var self = {
                        close: function() {},
                        onclick: null,
                        onclose: null,
                        onerror: null,
                        onshow: null,
                        addEventListener: function() {},
                        removeEventListener: function() {},
                        dispatchEvent: function() { return true; }
                    };
                    return self;
                }

                MiniBrowserNotification.permission = _permission;
                MiniBrowserNotification.requestPermission = function(cb) {
                    var result = Promise.resolve(_permission);
                    if (typeof cb === 'function') cb(_permission);
                    return result;
                };

                try {
                    Object.defineProperty(window, 'Notification', {
                        configurable: true,
                        enumerable: true,
                        get: function() { return MiniBrowserNotification; },
                        set: function() {}
                    });
                } catch(e) {
                    window.Notification = MiniBrowserNotification;
                }

                // Patch navigator.permissions.query so sites see 'granted'
                try {
                    var origQuery = navigator.permissions.query.bind(navigator.permissions);
                    navigator.permissions.query = function(desc) {
                        if (desc && desc.name === 'notifications') {
                            return Promise.resolve({ state: 'granted', onchange: null });
                        }
                        return origQuery(desc);
                    };
                } catch(e) {}
            })();

            function sendHostAction(action, data) {
                try {
                    var query = [];
                    if (data) {
                        for (var k in data) {
                            if (Object.prototype.hasOwnProperty.call(data, k) && data[k] !== undefined && data[k] !== null) {
                                query.push(encodeURIComponent(k) + '=' + encodeURIComponent(data[k]));
                            }
                        }
                    }
                    var actionUrl = 'minibrowser-action://' + action + (query.length ? '?' + query.join('&') : '');
                    window.location.href = actionUrl;
                } catch (err) {
                    console.error('sendHostAction failed:', err);
                }
            }

            window.addEventListener('keydown', function(e) {
                if (e.key === 'F12') {
                    e.preventDefault();
                    e.stopPropagation();
                    sendHostAction('inspect');
                    return;
                }
                var isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
                var mod = isMac ? e.metaKey : e.ctrlKey;
                if (mod) {
                    var k = e.key.toLowerCase();
                    if ((e.altKey || e.shiftKey) && (k === 'i' || k === 'c')) {
                        e.preventDefault();
                        e.stopPropagation();
                        sendHostAction('inspect');
                        return;
                    }
                    if ((e.key >= '1' && e.key <= '9') || ['t', 'n', 'w', 'm', 'h', 'r', 'l', '/', '[', ']', '+', '=', '-'].indexOf(k) !== -1) {
                        e.preventDefault();
                        e.stopPropagation();
                        sendHostAction('shortcut', { key: e.key, alt: e.altKey, shift: e.shiftKey });
                    }
                }
            }, true);

            // Context Menu & Link Actions (Open in Default Browser, Open in New Session, Copy/Paste, Downloads)
            (function() {
                var menuId = 'mini-browser-ctx-menu';

                function removeMenu() {
                    var el = document.getElementById(menuId);
                    if (el) el.remove();
                }

                document.addEventListener('click', function(e) {
                    if (!e.target.closest('#' + menuId)) {
                        removeMenu();
                    }
                }, true);

                window.addEventListener('blur', removeMenu);
                window.addEventListener('scroll', removeMenu, true);
                window.addEventListener('resize', removeMenu);
                window.addEventListener('keydown', function(e) {
                    if (e.key === 'Escape') removeMenu();
                }, true);

                // === Blob & Data download handling (WhatsApp Web, Gmail, web apps) ===
                window.__minibrowser_pending_downloads = window.__minibrowser_pending_downloads || {};

                function extractFilenameFromUrl(url) {
                    try {
                        var parsed = new URL(url);
                        var parts = parsed.pathname.split('/');
                        var last = parts[parts.length - 1];
                        if (last && last.indexOf('.') !== -1) {
                            return decodeURIComponent(last);
                        }
                    } catch(e) {}
                    return '';
                }

                function triggerDownloadPayload(dataUriOrBase64, filename) {
                    var id = 'dl_' + Date.now() + '_' + Math.random().toString(36).substr(2, 6);
                    window.__minibrowser_pending_downloads[id] = {
                        filename: filename || 'descarga',
                        data: dataUriOrBase64
                    };
                    sendHostAction('blob-download-ready', { id: id, filename: filename || 'descarga' });
                }

                function triggerBlobDownload(blobOrUrl, suggestedFilename) {
                    var finalName = suggestedFilename || 'descarga';
                    if (typeof blobOrUrl === 'string') {
                        if (blobOrUrl.startsWith('data:')) {
                            triggerDownloadPayload(blobOrUrl, finalName);
                        } else if (blobOrUrl.startsWith('blob:')) {
                            fetch(blobOrUrl)
                                .then(function(r) { return r.blob(); })
                                .then(function(b) {
                                    var reader = new FileReader();
                                    reader.onloadend = function() {
                                        triggerDownloadPayload(reader.result, finalName);
                                    };
                                    reader.readAsDataURL(b);
                                })
                                .catch(function(err) {
                                    console.error('MiniBrowser blob fetch failed:', err);
                                });
                        } else if (blobOrUrl.startsWith('http://') || blobOrUrl.startsWith('https://')) {
                            fetch(blobOrUrl)
                                .then(function(r) { return r.blob(); })
                                .then(function(b) {
                                    var reader = new FileReader();
                                    reader.onloadend = function() {
                                        triggerDownloadPayload(reader.result, finalName);
                                    };
                                    reader.readAsDataURL(b);
                                })
                                .catch(function() {
                                    sendHostAction('download-file', { url: blobOrUrl, filename: finalName });
                                });
                        }
                    } else if (blobOrUrl instanceof Blob) {
                        var reader = new FileReader();
                        reader.onloadend = function() {
                            triggerDownloadPayload(reader.result, finalName);
                        };
                        reader.readAsDataURL(blobOrUrl);
                    }
                }

                // Intercept programmatic anchor clicks (used by WhatsApp Web, Gmail attachments, etc.)
                try {
                    var origAnchorClick = HTMLAnchorElement.prototype.click;
                    HTMLAnchorElement.prototype.click = function() {
                        var href = this.href || '';
                        var downloadAttr = this.getAttribute('download');
                        if (downloadAttr !== null && downloadAttr !== undefined) {
                            var name = downloadAttr || this.download || extractFilenameFromUrl(href) || 'descarga';
                            if (href.startsWith('blob:') || href.startsWith('data:')) {
                                triggerBlobDownload(href, name);
                                return;
                            }
                        }
                        return origAnchorClick.apply(this, arguments);
                    };
                } catch(e) {}

                // Intercept user clicks on links with download attribute or blob/data href
                document.addEventListener('click', function(e) {
                    var a = e.target.closest('a');
                    if (a) {
                        var href = a.href || '';
                        var downloadAttr = a.getAttribute('download');
                        if (downloadAttr !== null && downloadAttr !== undefined) {
                            var name = downloadAttr || a.download || extractFilenameFromUrl(href) || 'descarga';
                            if (href.startsWith('blob:') || href.startsWith('data:')) {
                                e.preventDefault();
                                e.stopPropagation();
                                triggerBlobDownload(href, name);
                            }
                        }
                    }
                }, true);

                window.addEventListener('contextmenu', function(e) {
                    if (e.shiftKey) return; // Allow native menu if holding Shift

                    removeMenu();

                    var isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
                    var modName = isMac ? 'Cmd' : 'Ctrl';

                    var linkEl = e.target.closest('a[href]');
                    var targetUrl = linkEl ? linkEl.href : null;
                    var selectedText = window.getSelection() ? window.getSelection().toString().trim() : '';
                    var activeEl = document.activeElement;
                    var isEditable = activeEl && (
                        activeEl.isContentEditable ||
                        activeEl.tagName === 'INPUT' ||
                        activeEl.tagName === 'TEXTAREA'
                    );

                    var imgEl = e.target.closest('img');
                    var imgSrc = imgEl ? (imgEl.currentSrc || imgEl.src) : null;
                    if (!imgSrc && e.target.tagName === 'CANVAS') {
                        try { imgSrc = e.target.toDataURL('image/png'); } catch(e) {}
                    }

                    var items = [];

                    // Image actions
                    if (imgSrc) {
                        var imgName = extractFilenameFromUrl(imgSrc) || ('imagen_' + Date.now() + '.png');
                        items.push({
                            icon: '💾',
                            label: 'Guardar imagen en PC',
                            action: function() {
                                triggerBlobDownload(imgSrc, imgName);
                            }
                        });
                        items.push({
                            icon: '📋',
                            label: 'Copiar enlace de imagen',
                            action: function() {
                                if (navigator.clipboard && navigator.clipboard.writeText) {
                                    navigator.clipboard.writeText(imgSrc);
                                }
                            }
                        });
                        items.push({ separator: true });
                    }

                    if (targetUrl && (targetUrl.startsWith('http://') || targetUrl.startsWith('https://') || targetUrl.startsWith('mailto:'))) {
                        items.push({
                            icon: '🌐',
                            label: 'Open in Default Browser',
                            action: function() {
                                sendHostAction('open-default-browser', { url: targetUrl });
                            }
                        });
                        items.push({
                            icon: '🔗',
                            label: 'Open in New Session Tab',
                            action: function() {
                                sendHostAction('open-new-session', { url: targetUrl });
                            }
                        });
                        items.push({
                            icon: '💾',
                            label: 'Descargar archivo en PC',
                            action: function() {
                                var linkName = extractFilenameFromUrl(targetUrl) || 'descarga';
                                triggerBlobDownload(targetUrl, linkName);
                            }
                        });
                        items.push({
                            icon: '📋',
                            label: 'Copy Link Address',
                            action: function() {
                                if (navigator.clipboard && navigator.clipboard.writeText) {
                                    navigator.clipboard.writeText(targetUrl);
                                }
                            }
                        });
                        items.push({ separator: true });
                    }

                    if (selectedText) {
                        items.push({
                            icon: '📄',
                            label: 'Copy',
                            shortcut: modName + '+C',
                            action: function() {
                                document.execCommand('copy');
                            }
                        });
                        if (isEditable) {
                            items.push({
                                icon: '✂️',
                                label: 'Cut',
                                shortcut: modName + '+X',
                                action: function() {
                                    document.execCommand('cut');
                                }
                            });
                        }
                    }

                    if (isEditable) {
                        items.push({
                            icon: '📥',
                            label: 'Paste',
                            shortcut: modName + '+V',
                            action: function() {
                                if (navigator.clipboard && navigator.clipboard.readText) {
                                    navigator.clipboard.readText().then(function(text) {
                                        if (text) {
                                            document.execCommand('insertText', false, text);
                                        }
                                    }).catch(function() {
                                        document.execCommand('paste');
                                    });
                                } else {
                                    document.execCommand('paste');
                                }
                            }
                        });
                        items.push({
                            icon: '🔤',
                            label: 'Select All',
                            shortcut: modName + '+A',
                            action: function() {
                                document.execCommand('selectAll');
                            }
                        });
                        items.push({ separator: true });
                    }

                    items.push({
                        icon: '🔙',
                        label: 'Back',
                        action: function() { window.history.back(); }
                    });
                    items.push({
                        icon: '🔜',
                        label: 'Forward',
                        action: function() { window.history.forward(); }
                    });
                    items.push({
                        icon: '🔄',
                        label: 'Reload',
                        shortcut: modName + '+R',
                        action: function() { window.location.reload(); }
                    });

                    if (!targetUrl) {
                        items.push({ separator: true });
                        items.push({
                            icon: '🌐',
                            label: 'Open Page in Default Browser',
                            action: function() {
                                var curUrl = window.location.href;
                                sendHostAction('open-default-browser', { url: curUrl });
                            }
                        });
                    }

                    items.push({ separator: true });
                    items.push({
                        icon: '📁',
                        label: 'Abrir carpeta Descargas',
                        action: function() {
                            sendHostAction('open-downloads-folder');
                        }
                    });
                    items.push({
                        icon: '🔍',
                        label: 'Inspect Element',
                        shortcut: isMac ? 'Cmd+Opt+I' : 'F12',
                        action: function() {
                            sendHostAction('inspect');
                        }
                    });

                    e.preventDefault();

                    var menu = document.createElement('div');
                    menu.id = menuId;
                    menu.style.cssText = 'position:fixed; z-index:2147483647; background:rgba(24,24,27,0.96); backdrop-filter:blur(20px); -webkit-backdrop-filter:blur(20px); border:1px solid rgba(255,255,255,0.14); border-radius:10px; box-shadow:0 12px 36px rgba(0,0,0,0.65), 0 0 0 1px rgba(0,0,0,0.2); padding:5px; min-width:210px; max-width:280px; font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,Helvetica,sans-serif; font-size:13px; color:#e4e4e7; user-select:none;';

                    items.forEach(function(it) {
                        if (it.separator) {
                            var sep = document.createElement('div');
                            sep.style.cssText = 'height:1px; background:rgba(255,255,255,0.1); margin:4px 6px;';
                            menu.appendChild(sep);
                            return;
                        }
                        var row = document.createElement('div');
                        row.style.cssText = 'display:flex; align-items:center; justify-content:space-between; padding:6px 10px; border-radius:6px; cursor:pointer; gap:8px; font-weight:450; transition:background 0.1s ease;';
                        row.innerHTML = '<span style="display:flex; align-items:center; gap:8px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;"><span style="font-size:13px; opacity:0.9;">' + it.icon + '</span><span>' + it.label + '</span></span>' + (it.shortcut ? '<span style="font-size:11px; opacity:0.4; font-family:monospace;">' + it.shortcut + '</span>' : '');
                        
                        row.addEventListener('mouseenter', function() {
                            row.style.background = 'rgba(99,102,241,0.85)';
                            row.style.color = '#ffffff';
                        });
                        row.addEventListener('mouseleave', function() {
                            row.style.background = 'transparent';
                            row.style.color = '#e4e4e7';
                        });
                        row.addEventListener('click', function(ev) {
                            ev.stopPropagation();
                            removeMenu();
                            try { it.action(); } catch(err) { console.error(err); }
                        });
                        menu.appendChild(row);
                    });

                    document.documentElement.appendChild(menu);

                    var x = e.clientX;
                    var y = e.clientY;
                    var rect = menu.getBoundingClientRect();
                    if (x + rect.width > window.innerWidth) {
                        x = Math.max(10, window.innerWidth - rect.width - 10);
                    }
                    if (y + rect.height > window.innerHeight) {
                        y = Math.max(10, window.innerHeight - rect.height - 10);
                    }
                    menu.style.left = x + 'px';
                    menu.style.top = y + 'px';
                }, true);
            })();
        "##;

        let label_nav = label.clone();
        let label_new_win = label.clone();

        let selected_user_agent = get_user_agent_for_session(&raw_url, partition);
        let uuid_bytes = Self::partition_to_uuid_bytes(partition);

        let builder = WebviewBuilder::new(&label, WebviewUrl::External(parsed_url))
            .devtools(true)
            .data_directory(data_dir)
            .data_store_identifier(uuid_bytes)
            .user_agent(selected_user_agent)
            .initialization_script(init_script)
            .on_download(move |_webview, event| {
                match event {
                    tauri::webview::DownloadEvent::Requested { url, destination } => {
                        let filename = destination
                            .file_name()
                            .and_then(|f| f.to_str())
                            .unwrap_or_else(|| {
                                url.path_segments()
                                    .and_then(|mut s| s.next_back())
                                    .unwrap_or("descarga")
                            })
                            .to_string();
                        let final_path = get_unique_download_path(&filename);
                        *destination = final_path.clone();
                        let _ = app_handle_dl.emit(
                            "download-started",
                            serde_json::json!({
                                "url": url.to_string(),
                                "name": filename
                            }),
                        );
                        true
                    }
                    tauri::webview::DownloadEvent::Finished { url: _, path, success } => {
                        if success {
                            if let Some(ref p) = path {
                                notify_download_finished(&app_handle_dl, p);
                            } else {
                                let download_dir = dirs::download_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
                                notify_download_finished(&app_handle_dl, &download_dir.join("archivo"));
                            }
                        }
                        true
                    }
                    _ => true,
                }
            })
            .on_navigation(move |url| {
                if url.scheme() == "minibrowser-action" {
                    let action = url.host_str().unwrap_or("");
                    let mut params = std::collections::HashMap::new();
                    for (k, v) in url.query_pairs() {
                        params.insert(k.to_string(), v.to_string());
                    }
                    match action {
                        "open-default-browser" => {
                            if let Some(target_url) = params.get("url") {
                                use tauri_plugin_opener::OpenerExt;
                                let _ = app_handle_clone.opener().open_url(target_url, None::<&str>);
                            }
                        }
                        "open-new-session" => {
                            if let Some(target_url) = params.get("url") {
                                let _ = app_handle_clone.emit("open-new-session-url", target_url);
                            }
                        }
                        "open-downloads-folder" => {
                            let download_dir = dirs::download_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
                            use tauri_plugin_opener::OpenerExt;
                            let _ = app_handle_clone.opener().open_path(download_dir.to_string_lossy().to_string(), None::<&str>);
                        }
                        "blob-download-ready" => {
                            let id = params.get("id").cloned().unwrap_or_default();
                            let fallback_filename = params.get("filename").cloned().unwrap_or_else(|| "descarga".to_string());
                            if let Some(wv) = app_handle_clone.get_webview(&label_nav) {
                                let app_handle_save = app_handle_clone.clone();
                                let eval_code = format!(
                                    r#"(function() {{
                                        var map = window.__minibrowser_pending_downloads;
                                        if (map && map['{}']) {{
                                            var item = map['{}'];
                                            delete map['{}'];
                                            return JSON.stringify(item);
                                        }}
                                        return null;
                                    }})()"#,
                                    id, id, id
                                );
                                let _ = wv.eval_with_callback(eval_code, move |res| {
                                    // Parse result: either stringified JSON or JSON value
                                    let parsed_val: Option<serde_json::Value> = if let Ok(raw_json) = serde_json::from_str::<String>(&res) {
                                        serde_json::from_str::<serde_json::Value>(&raw_json).ok()
                                    } else {
                                        serde_json::from_str::<serde_json::Value>(&res).ok()
                                    };

                                    if let Some(val) = parsed_val {
                                        let fname = val["filename"].as_str().unwrap_or(&fallback_filename);
                                        let data_uri = val["data"].as_str().unwrap_or("");
                                        if !data_uri.is_empty() {
                                            match save_base64_file(fname, data_uri) {
                                                Ok(saved_path) => {
                                                    notify_download_finished(&app_handle_save, &saved_path);
                                                }
                                                Err(e) => {
                                                    eprintln!("[DOWNLOAD] Error saving blob file: {}", e);
                                                }
                                            }
                                        }
                                    }
                                });
                            }
                        }
                        "download-file" => {
                            if let Some(target_url) = params.get("url") {
                                let fallback_name = params.get("filename").cloned().unwrap_or_else(|| "descarga".to_string());
                                let app_handle_save = app_handle_clone.clone();
                                let target_url = target_url.clone();
                                std::thread::spawn(move || {
                                    let filename = if fallback_name.trim().is_empty() || fallback_name == "descarga" {
                                        target_url.split('/').last().and_then(|s| s.split('?').next()).unwrap_or("descarga")
                                    } else {
                                        &fallback_name
                                    };
                                    let dest = get_unique_download_path(filename);
                                    #[cfg(not(target_os = "windows"))]
                                    {
                                        let status = std::process::Command::new("curl")
                                            .arg("-L")
                                            .arg("-s")
                                            .arg("-o")
                                            .arg(&dest)
                                            .arg(&target_url)
                                            .status();
                                        if let Ok(st) = status {
                                            if st.success() && dest.exists() {
                                                notify_download_finished(&app_handle_save, &dest);
                                            }
                                        }
                                    }
                                    #[cfg(target_os = "windows")]
                                    {
                                        let status = std::process::Command::new("powershell")
                                            .arg("-Command")
                                            .arg(format!("Invoke-WebRequest -Uri '{}' -OutFile '{}'", target_url, dest.to_string_lossy()))
                                            .status();
                                        if let Ok(st) = status {
                                            if st.success() && dest.exists() {
                                                notify_download_finished(&app_handle_save, &dest);
                                            }
                                        }
                                    }
                                });
                            }
                        }
                        "shortcut" => {
                            let key = params.get("key").cloned().unwrap_or_default();
                            let alt = params.get("alt").map(|v| v == "true").unwrap_or(false);
                            let shift = params.get("shift").map(|v| v == "true").unwrap_or(false);
                            let _ = app_handle_clone.emit(
                                "trigger-shortcut",
                                serde_json::json!({
                                    "key": key,
                                    "alt": alt,
                                    "shift": shift,
                                }),
                            );
                        }
                        "inspect" => {
                            if let Some(wv) = app_handle_clone.get_webview(&label_nav) {
                                #[cfg(any(debug_assertions, feature = "devtools"))]
                                {
                                    if wv.is_devtools_open() {
                                        wv.close_devtools();
                                    } else {
                                        wv.open_devtools();
                                    }
                                }
                            }
                        }
                        "notify" => {
                            let title = params.get("title").cloned().unwrap_or_else(|| partition_for_notif.clone());
                            let body = params.get("body").cloned().unwrap_or_default();

                            let notif_payload = serde_json::json!({
                                "partition": partition_for_notif,
                                "title": &title,
                                "body": &body,
                            });

                            // Guardar partition pendiente en estado global para que el frontend
                            // pueda navegar al tab correcto cuando el usuario haga clic en la notif.
                            {
                                let state: tauri::State<crate::AppStateWrapper> = app_handle_clone.state();
                                let mut pending = state.pending_notification.lock().unwrap();
                                *pending = Some(notif_payload.clone());
                            }

                            // Notificar al frontend inmediatamente para resaltar el tab con actividad.
                            let _ = app_handle_clone.emit("notification-received", &notif_payload);

                            use tauri_plugin_notification::NotificationExt;
                            let _ = app_handle_clone
                                .notification()
                                .builder()
                                .title(&title)
                                .body(&body)
                                .show();
                        }
                        _ => {}
                    }
                    return false;
                }

                let _ = app_handle_clone.emit(
                    "session-navigated",
                    serde_json::json!({
                        "partition": part_clone,
                        "url": url.to_string()
                    }),
                );
                true
            })
            .on_new_window(move |url, _features| {
                if url.scheme() == "minibrowser-action" {
                    let action = url.host_str().unwrap_or("");
                    let mut params = std::collections::HashMap::new();
                    for (k, v) in url.query_pairs() {
                        params.insert(k.to_string(), v.to_string());
                    }
                    match action {
                        "open-default-browser" => {
                            if let Some(target_url) = params.get("url") {
                                use tauri_plugin_opener::OpenerExt;
                                let _ = app_handle_new_win.opener().open_url(target_url, None::<&str>);
                            }
                        }
                        "open-new-session" => {
                            if let Some(target_url) = params.get("url") {
                                let _ = app_handle_new_win.emit("open-new-session-url", target_url);
                            }
                        }
                        "open-downloads-folder" => {
                            let download_dir = dirs::download_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
                            use tauri_plugin_opener::OpenerExt;
                            let _ = app_handle_new_win.opener().open_path(download_dir.to_string_lossy().to_string(), None::<&str>);
                        }
                        "inspect" => {
                            if let Some(wv) = app_handle_new_win.get_webview(&label_new_win) {
                                #[cfg(any(debug_assertions, feature = "devtools"))]
                                {
                                    if wv.is_devtools_open() {
                                        wv.close_devtools();
                                    } else {
                                        wv.open_devtools();
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    return tauri::webview::NewWindowResponse::Deny;
                }

                // Normal target="_blank" links open in a new session tab
                let _ = app_handle_new_win.emit("open-new-session-url", url.to_string());
                tauri::webview::NewWindowResponse::Deny
            });

        let webview = window
            .add_child(
                builder,
                LogicalPosition::new(0.0, HEADER_HEIGHT),
                LogicalSize::new(logical_w, content_h),
            )
            .map_err(|e| format!("Failed to add child webview: {}", e))?;

        let _ = webview.show();
        let _ = webview.set_focus();

        self.sessions
            .lock()
            .unwrap()
            .insert(partition.to_string(), label.clone());
        *active_lock = Some(label.clone());

        Ok(label)
    }

    pub fn deactivate_all(&self, app: &AppHandle) {
        let mut active_lock = self.active_label.lock().unwrap();
        if let Some(ref label) = *active_lock {
            if let Some(wv) = app.get_webview(label) {
                let _ = wv.hide();
            }
        }
        *active_lock = None;
    }

    pub fn hide_active(&self, app: &AppHandle) {
        let active_lock = self.active_label.lock().unwrap();
        if let Some(ref label) = *active_lock {
            if let Some(wv) = app.get_webview(label) {
                let _ = wv.hide();
            }
        }
    }

    pub fn show_active(&self, app: &AppHandle) {
        let active_lock = self.active_label.lock().unwrap();
        if let Some(ref label) = *active_lock {
            if let Some(wv) = app.get_webview(label) {
                let _ = wv.show();
                let _ = wv.set_focus();
            }
        }
    }

    pub fn sync_bounds(&self, app: &AppHandle, width: f64, height: f64) {
        let active_lock = self.active_label.lock().unwrap();
        if let Some(ref label) = *active_lock {
            if let Some(wv) = app.get_webview(label) {
                let content_h = (height - HEADER_HEIGHT).max(100.0);
                let _ = wv.set_bounds(Rect {
                    position: LogicalPosition::new(0.0, HEADER_HEIGHT).into(),
                    size: LogicalSize::new(width, content_h).into(),
                });
            }
        }
    }

    pub fn close_session(&self, app: &AppHandle, partition: &str) -> Result<(), String> {
        let label = Self::clean_label(partition);
        if let Some(wv) = app.get_webview(&label) {
            let _ = wv.close();
        }

        let mut active_lock = self.active_label.lock().unwrap();
        if let Some(ref active) = *active_lock {
            if active == &label {
                *active_lock = None;
            }
        }
        self.sessions.lock().unwrap().remove(partition);
        Ok(())
    }

    pub fn go_back(&self, app: &AppHandle, partition: &str) {
        let label = Self::clean_label(partition);
        if let Some(wv) = app.get_webview(&label) {
            let _ = wv.eval("window.history.back()");
        }
    }

    pub fn go_forward(&self, app: &AppHandle, partition: &str) {
        let label = Self::clean_label(partition);
        if let Some(wv) = app.get_webview(&label) {
            let _ = wv.eval("window.history.forward()");
        }
    }

    pub fn reload(&self, app: &AppHandle, partition: &str) {
        let label = Self::clean_label(partition);
        if let Some(wv) = app.get_webview(&label) {
            let _ = wv.eval("window.location.reload()");
        }
    }

    pub fn navigate(&self, app: &AppHandle, partition: &str, url_str: &str) -> Result<(), String> {
        let label = Self::clean_label(partition);
        if let Some(wv) = app.get_webview(&label) {
            let formatted = if url_str.starts_with("http://") || url_str.starts_with("https://") {
                url_str.to_string()
            } else {
                format!("https://{}", url_str)
            };
            if let Ok(parsed) = formatted.parse() {
                let _ = wv.navigate(parsed);
            }
        }
        Ok(())
    }

    pub fn set_zoom(&self, app: &AppHandle, partition: &str, factor: f64) {
        let label = Self::clean_label(partition);
        if let Some(wv) = app.get_webview(&label) {
            let _ = wv.set_zoom(factor);
        }
    }

    pub fn get_active_label(&self) -> Option<String> {
        self.active_label.lock().unwrap().clone()
    }
}
