use crate::store::StoreManager;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Rect, WebviewBuilder, WebviewUrl,
};

pub const HEADER_HEIGHT: f64 = 42.0;

#[cfg(target_os = "macos")]
pub const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.3 Safari/605.1.15";

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

        let init_script = r##"
            try {
                Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
            } catch(e) {}

            window.addEventListener('keydown', function(e) {
                var isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
                var mod = isMac ? e.metaKey : e.ctrlKey;
                if (mod) {
                    var k = e.key.toLowerCase();
                    if ((e.key >= '1' && e.key <= '9') || ['t', 'n', 'w', 'm', 'h', 'r', 'l', '/', '[', ']', '+', '=', '-'].indexOf(k) !== -1) {
                        e.preventDefault();
                        e.stopPropagation();
                        if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
                            window.__TAURI_INTERNALS__.invoke('handle_child_shortcut', { key: e.key, alt: e.altKey, shift: e.shiftKey });
                        }
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
                                if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
                                    window.__TAURI_INTERNALS__.invoke('open_in_default_browser', { url: targetUrl });
                                } else {
                                    window.open(targetUrl, '_blank');
                                }
                            }
                        });
                        items.push({
                            icon: '🔗',
                            label: 'Open in New Session Tab',
                            action: function() {
                                if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
                                    window.__TAURI_INTERNALS__.invoke('open_in_new_session', { url: targetUrl });
                                }
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
                                if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
                                    window.__TAURI_INTERNALS__.invoke('open_in_default_browser', { url: curUrl });
                                }
                            }
                        });
                    }

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

        let builder = WebviewBuilder::new(&label, WebviewUrl::External(parsed_url))
            .data_directory(data_dir)
            .user_agent(USER_AGENT)
            .initialization_script(init_script)
            .on_navigation(move |url| {
                let _ = app_handle_clone.emit(
                    "session-navigated",
                    serde_json::json!({
                        "partition": part_clone,
                        "url": url.to_string()
                    }),
                );
                true
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
}
