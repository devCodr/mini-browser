use crate::store::StoreManager;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Rect, WebviewBuilder, WebviewUrl,
};

pub const HEADER_HEIGHT: f64 = 42.0;

#[cfg(target_os = "macos")]
pub const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";

#[cfg(not(target_os = "macos"))]
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";

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

        let partition_for_notif = partition.to_string();

        let init_script = r##"
            // === Chrome Fingerprint: defeat embedded webview detection by Google, etc. ===
            (function() {
                // 1. Remove webdriver flag
                try { Object.defineProperty(navigator, 'webdriver', { get: () => undefined }); } catch(e) {}

                // 2. Inject window.chrome object (Google checks this first)
                if (!window.chrome) {
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

            // Context Menu & Link Actions (Open in Default Browser, Open in New Session, Copy/Paste)
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

                    var items = [];

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

        let builder = WebviewBuilder::new(&label, WebviewUrl::External(parsed_url))
            .devtools(true)
            .data_directory(data_dir)
            .user_agent(USER_AGENT)
            .initialization_script(init_script)
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
