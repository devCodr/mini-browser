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

        let init_script = r#"
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
        "#;

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
