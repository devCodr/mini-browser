pub mod session;
pub mod store;

use session::SessionManager;
use std::sync::Mutex;
use store::{Bookmark, StoreManager};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};

pub struct AppStateWrapper {
    pub store: Mutex<StoreManager>,
    pub sessions: SessionManager,
}

pub fn rebuild_menu(app: &AppHandle, bookmarks: &[Bookmark]) {
    let Ok(file_menu) = tauri::menu::SubmenuBuilder::new(app, "File")
        .item(&tauri::menu::MenuItemBuilder::with_id("new_session", "New Session").accelerator("CmdOrCtrl+T").build(app).unwrap())
        .item(&tauri::menu::MenuItemBuilder::with_id("close_session", "Close Active Session").accelerator("CmdOrCtrl+W").build(app).unwrap())
        .item(&tauri::menu::MenuItemBuilder::with_id("manage_sessions", "Manage Sessions...").accelerator("CmdOrCtrl+M").build(app).unwrap())
        .separator()
        .item(&tauri::menu::MenuItemBuilder::with_id("lock_now", "Lock Browser").accelerator("CmdOrCtrl+Alt+L").build(app).unwrap())
        .build() else { return; };

    let mut tabs_builder = tauri::menu::SubmenuBuilder::new(app, "Tabs");

    // Dynamic list: show ONLY actual tabs with their real names and numbers!
    if bookmarks.is_empty() {
        if let Ok(item) = tauri::menu::MenuItemBuilder::with_id("no_tabs", "No Open Tabs").enabled(false).build(app) {
            tabs_builder = tabs_builder.item(&item);
        }
    } else {
        for (idx, bm) in bookmarks.iter().take(9).enumerate() {
            let num = idx + 1;
            let display_title = if bm.title.trim().is_empty() {
                bm.url.trim_start_matches("https://").trim_start_matches("http://")
            } else {
                &bm.title
            };
            let label = format!("{}. {}", num, display_title);
            let accel = format!("CmdOrCtrl+{}", num);
            let id = format!("tab_{}", num);
            if let Ok(item) = tauri::menu::MenuItemBuilder::with_id(id, label).accelerator(accel).build(app) {
                tabs_builder = tabs_builder.item(&item);
            }
        }
    }

    tabs_builder = tabs_builder.separator();
    if let Ok(item) = tauri::menu::MenuItemBuilder::with_id("move_tab_left", "Move Active Tab Left").accelerator("CmdOrCtrl+Alt+Left").build(app) {
        tabs_builder = tabs_builder.item(&item);
    }
    if let Ok(item) = tauri::menu::MenuItemBuilder::with_id("move_tab_right", "Move Active Tab Right").accelerator("CmdOrCtrl+Alt+Right").build(app) {
        tabs_builder = tabs_builder.item(&item);
    }

    let Ok(tabs_menu) = tabs_builder.build() else { return; };

    let Ok(edit_menu) = tauri::menu::SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .separator()
        .select_all()
        .build() else { return; };

    let Ok(view_menu) = tauri::menu::SubmenuBuilder::new(app, "View")
        .item(&tauri::menu::MenuItemBuilder::with_id("reload", "Reload Active Tab").accelerator("CmdOrCtrl+R").build(app).unwrap())
        .item(&tauri::menu::MenuItemBuilder::with_id("home", "Home / Dashboard").accelerator("CmdOrCtrl+H").build(app).unwrap())
        .item(&tauri::menu::MenuItemBuilder::with_id("focus_url", "Focus Address Bar").accelerator("CmdOrCtrl+L").build(app).unwrap())
        .item(&tauri::menu::MenuItemBuilder::with_id("shortcuts_help", "Keyboard Shortcuts & Help").accelerator("CmdOrCtrl+/").build(app).unwrap())
        .build() else { return; };

    if let Ok(menu) = tauri::menu::MenuBuilder::new(app)
        .items(&[&file_menu, &edit_menu, &tabs_menu, &view_menu])
        .build() {
        let _ = app.set_menu(menu);
    }
}

// === Tauri Commands ===

#[tauri::command]
fn get_state(state: State<'_, AppStateWrapper>) -> Result<serde_json::Value, String> {
    let store = state.store.lock().unwrap();
    let settings = store.load_settings();
    let bookmarks = store.load_bookmarks();
    Ok(serde_json::json!({
        "settings": settings,
        "bookmarks": bookmarks
    }))
}

#[tauri::command]
fn activate_session(
    app: AppHandle,
    state: State<'_, AppStateWrapper>,
    partition: String,
    url: String,
) -> Result<String, String> {
    let store = state.store.lock().unwrap();
    state.sessions.activate_session(&app, &partition, &url, &store)
}

#[tauri::command]
fn deactivate_all(app: AppHandle, state: State<'_, AppStateWrapper>) -> Result<(), String> {
    state.sessions.deactivate_all(&app);
    Ok(())
}

#[tauri::command]
fn hide_active_session(app: AppHandle, state: State<'_, AppStateWrapper>) -> Result<(), String> {
    state.sessions.hide_active(&app);
    Ok(())
}

#[tauri::command]
fn show_active_session(app: AppHandle, state: State<'_, AppStateWrapper>) -> Result<(), String> {
    state.sessions.show_active(&app);
    Ok(())
}

#[tauri::command]
fn open_in_default_browser(app: AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener().open_url(&url, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_in_new_session(app: AppHandle, url: String) -> Result<(), String> {
    app.emit("open-new-session-url", &url).map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_devtools(app: AppHandle, state: State<'_, AppStateWrapper>) -> Result<(), String> {
    #[cfg(any(debug_assertions, feature = "devtools"))]
    {
        if let Some(label) = state.sessions.get_active_label() {
            if let Some(wv) = app.get_webview(&label) {
                if wv.is_devtools_open() {
                    wv.close_devtools();
                } else {
                    wv.open_devtools();
                }
                return Ok(());
            }
        }
        if let Some(main_wv) = app.get_webview("main") {
            if main_wv.is_devtools_open() {
                main_wv.close_devtools();
            } else {
                main_wv.open_devtools();
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn start_dragging(window: tauri::Window) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
fn handle_child_shortcut(
    app: AppHandle,
    key: String,
    alt: bool,
    shift: bool,
) -> Result<(), String> {
    app.emit(
        "trigger-shortcut",
        serde_json::json!({
            "key": key,
            "alt": alt,
            "shift": shift
        }),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn add_bookmark(
    app: AppHandle,
    state: State<'_, AppStateWrapper>,
    title: String,
    url: String,
    partition: String,
    badge: Option<String>,
    color: Option<String>,
    icon_svg: Option<String>,
) -> Result<Vec<Bookmark>, String> {
    let store = state.store.lock().unwrap();
    let mut bookmarks = store.load_bookmarks();

    if !bookmarks.iter().any(|b| b.partition == partition) {
        let id = format!("bm_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis());

        bookmarks.push(Bookmark {
            id,
            title,
            url,
            partition,
            badge,
            color,
            icon_svg,
        });
        store.save_bookmarks(&bookmarks);
        rebuild_menu(&app, &bookmarks);
    }
    Ok(bookmarks)
}

#[tauri::command]
fn remove_bookmark(
    app: AppHandle,
    state: State<'_, AppStateWrapper>,
    partition: String,
) -> Result<Vec<Bookmark>, String> {
    let _ = state.sessions.close_session(&app, &partition);
    let store = state.store.lock().unwrap();
    let mut bookmarks = store.load_bookmarks();
    bookmarks.retain(|b| b.partition != partition);
    store.save_bookmarks(&bookmarks);
    rebuild_menu(&app, &bookmarks);
    Ok(bookmarks)
}

#[tauri::command]
fn reorder_bookmarks(
    app: AppHandle,
    state: State<'_, AppStateWrapper>,
    new_order: Vec<Bookmark>,
) -> Result<Vec<Bookmark>, String> {
    let store = state.store.lock().unwrap();
    store.save_bookmarks(&new_order);
    rebuild_menu(&app, &new_order);
    Ok(new_order)
}

#[tauri::command]
fn update_bookmark_icon(
    state: State<'_, AppStateWrapper>,
    partition: String,
    icon_svg: Option<String>,
) -> Result<Vec<Bookmark>, String> {
    let store = state.store.lock().unwrap();
    let mut bookmarks = store.load_bookmarks();
    if let Some(b) = bookmarks.iter_mut().find(|x| x.partition == partition) {
        b.icon_svg = icon_svg;
        store.save_bookmarks(&bookmarks);
    }
    Ok(bookmarks)
}

#[tauri::command]
fn update_bookmark_meta(
    app: AppHandle,
    state: State<'_, AppStateWrapper>,
    partition: String,
    title: Option<String>,
    badge: Option<String>,
    color: Option<String>,
    icon_svg: Option<String>,
) -> Result<Vec<Bookmark>, String> {
    let store = state.store.lock().unwrap();
    let mut bookmarks = store.load_bookmarks();
    if let Some(b) = bookmarks.iter_mut().find(|x| x.partition == partition) {
        if let Some(t) = title { b.title = t; }
        b.badge = badge;
        b.color = color;
        if let Some(icon) = icon_svg {
            let trimmed = icon.trim().to_string();
            b.icon_svg = if trimmed.is_empty() { None } else { Some(trimmed) };
        }
        store.save_bookmarks(&bookmarks);
        rebuild_menu(&app, &bookmarks);
    }
    Ok(bookmarks)
}

#[tauri::command]
fn nav_back(app: AppHandle, state: State<'_, AppStateWrapper>, partition: String) {
    state.sessions.go_back(&app, &partition);
}

#[tauri::command]
fn nav_forward(app: AppHandle, state: State<'_, AppStateWrapper>, partition: String) {
    state.sessions.go_forward(&app, &partition);
}

#[tauri::command]
fn nav_reload(app: AppHandle, state: State<'_, AppStateWrapper>, partition: String) {
    state.sessions.reload(&app, &partition);
}

#[tauri::command]
fn nav_to(
    app: AppHandle,
    state: State<'_, AppStateWrapper>,
    partition: String,
    url: String,
) -> Result<(), String> {
    state.sessions.navigate(&app, &partition, &url)
}

#[tauri::command]
fn set_zoom(app: AppHandle, state: State<'_, AppStateWrapper>, partition: String, factor: f64) {
    state.sessions.set_zoom(&app, &partition, factor);
}

#[tauri::command]
fn verify_pin(state: State<'_, AppStateWrapper>, pin: String) -> bool {
    let store = state.store.lock().unwrap();
    let settings = store.load_settings();
    settings.verify_pin(&pin)
}

#[tauri::command]
fn set_pin(state: State<'_, AppStateWrapper>, pin: String) -> Result<bool, String> {
    let store = state.store.lock().unwrap();
    let mut settings = store.load_settings();
    settings.set_pin(&pin);
    store.save_settings(&settings);
    Ok(true)
}

#[tauri::command]
fn toggle_lock(state: State<'_, AppStateWrapper>) -> Result<bool, String> {
    let store = state.store.lock().unwrap();
    let mut settings = store.load_settings();
    settings.lock_enabled = !settings.lock_enabled;
    store.save_settings(&settings);
    Ok(settings.lock_enabled)
}

#[tauri::command]
fn set_inactivity_ms(state: State<'_, AppStateWrapper>, ms: u64) -> Result<u64, String> {
    let store = state.store.lock().unwrap();
    let mut settings = store.load_settings();
    settings.inactivity_ms = ms.max(60000);
    store.save_settings(&settings);
    Ok(settings.inactivity_ms)
}

#[tauri::command]
fn lock_now(app: AppHandle, state: State<'_, AppStateWrapper>) -> Result<(), String> {
    state.sessions.deactivate_all(&app);
    Ok(())
}

#[tauri::command]
fn update_settings(
    state: State<'_, AppStateWrapper>,
    lock_enabled: bool,
    inactivity_ms: u64,
    lock_on_launch: bool,
    start_minimized: bool,
) -> Result<store::Settings, String> {
    let store = state.store.lock().unwrap();
    let mut settings = store.load_settings();
    settings.lock_enabled = lock_enabled;
    settings.inactivity_ms = inactivity_ms.max(60000);
    settings.lock_on_launch = lock_on_launch;
    settings.start_minimized = start_minimized;
    store.save_settings(&settings);
    Ok(settings)
}

#[tauri::command]
fn get_platform() -> String {
    #[cfg(target_os = "macos")]
    return "macos".to_string();
    #[cfg(target_os = "windows")]
    return "windows".to_string();
    #[cfg(target_os = "linux")]
    return "linux".to_string();
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    return "unknown".to_string();
}

#[tauri::command]
fn minimize_window(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_window("main") {
        let _ = w.minimize();
    }
    Ok(())
}

#[tauri::command]
fn toggle_maximize_window(app: AppHandle) -> Result<bool, String> {
    if let Some(w) = app.get_window("main") {
        let is_max = w.is_maximized().unwrap_or(false);
        if is_max {
            let _ = w.unmaximize();
            Ok(false)
        } else {
            let _ = w.maximize();
            Ok(true)
        }
    } else {
        Ok(false)
    }
}

#[tauri::command]
fn close_window(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_window("main") {
        let _ = w.close();
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::current_dir().unwrap().join(".data"));

            let store_manager = StoreManager::new(data_dir);
            let session_manager = SessionManager::new();

            let initial_settings = store_manager.load_settings();
            let initial_bookmarks = store_manager.load_bookmarks();

            app.manage(AppStateWrapper {
                store: Mutex::new(store_manager),
                sessions: session_manager,
            });

            rebuild_menu(&app.handle(), &initial_bookmarks);

            let app_handle_menu = app.handle().clone();
            app.on_menu_event(move |_app, event| {
                let id = event.id().as_ref();
                let _ = app_handle_menu.emit("menu-shortcut", id);
            });

            if let Some(main_window) = app.get_window("main") {
                #[cfg(not(target_os = "macos"))]
                {
                    let _ = main_window.set_decorations(false);
                }

                if initial_settings.start_minimized {
                    let _ = main_window.minimize();
                }

                #[cfg(target_os = "macos")]
                {
                    apply_traffic_lights_inset(&main_window, 16.0, 14.0);
                    let app_h = app.handle().clone();
                    std::thread::spawn(move || {
                        for delay in [50, 150, 300, 600] {
                            std::thread::sleep(std::time::Duration::from_millis(delay));
                            let h = app_h.clone();
                            let _ = app_h.run_on_main_thread(move || {
                                if let Some(w) = h.get_window("main") {
                                    apply_traffic_lights_inset(&w, 16.0, 14.0);
                                }
                            });
                        }
                    });
                }

                let app_handle = app.handle().clone();
                main_window.on_window_event(move |event| {
                    match event {
                        WindowEvent::Resized(size) => {
                            if let Some(w) = app_handle.get_window("main") {
                                #[cfg(target_os = "macos")]
                                apply_traffic_lights_inset(&w, 16.0, 14.0);

                                if let Ok(scale) = w.scale_factor() {
                                    let logical_w = size.width as f64 / scale;
                                    let logical_h = size.height as f64 / scale;
                                    let state: State<AppStateWrapper> = app_handle.state();
                                    state.sessions.sync_bounds(&app_handle, logical_w, logical_h);
                                }
                            }
                        }
                        #[cfg(target_os = "macos")]
                        WindowEvent::Focused(_) => {
                            if let Some(w) = app_handle.get_window("main") {
                                apply_traffic_lights_inset(&w, 16.0, 14.0);
                            }
                        }
                        _ => {}
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            get_platform,
            minimize_window,
            toggle_maximize_window,
            close_window,
            activate_session,
            deactivate_all,
            hide_active_session,
            show_active_session,
            start_dragging,
            handle_child_shortcut,
            add_bookmark,
            remove_bookmark,
            reorder_bookmarks,
            update_bookmark_icon,
            update_bookmark_meta,
            nav_back,
            nav_forward,
            nav_reload,
            nav_to,
            set_zoom,
            verify_pin,
            set_pin,
            toggle_lock,
            set_inactivity_ms,
            open_in_default_browser,
            open_in_new_session,
            toggle_devtools,
            update_settings,
            lock_now
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(target_os = "macos")]
pub fn apply_traffic_lights_inset(window: &tauri::Window, x: f64, y: f64) {
    use objc2_app_kit::{NSView, NSWindow, NSWindowButton};

    eprintln!("[TRAFFIC] apply_traffic_lights_inset called with x={}, y={}", x, y);
    let Ok(ns_window_ptr) = window.ns_window() else {
        eprintln!("[TRAFFIC] window.ns_window() returned Err");
        return;
    };
    if ns_window_ptr.is_null() {
        eprintln!("[TRAFFIC] ns_window_ptr is null");
        return;
    }

    unsafe {
        let ns_window: &NSWindow = &*ns_window_ptr.cast();
        let Some(close) = ns_window.standardWindowButton(NSWindowButton::CloseButton) else {
            eprintln!("[TRAFFIC] CloseButton not found");
            return;
        };
        let Some(miniaturize) = ns_window.standardWindowButton(NSWindowButton::MiniaturizeButton) else {
            eprintln!("[TRAFFIC] MiniaturizeButton not found");
            return;
        };
        let zoom = ns_window.standardWindowButton(NSWindowButton::ZoomButton);

        let Some(superview1) = close.superview() else {
            eprintln!("[TRAFFIC] superview1 not found");
            return;
        };
        let Some(title_bar_container_view) = superview1.superview() else {
            eprintln!("[TRAFFIC] title_bar_container_view not found");
            return;
        };

        let close_rect = NSView::frame(&close);
        let header_h = 42.0f64;

        let mut title_bar_rect = NSView::frame(&title_bar_container_view);
        title_bar_rect.size.height = header_h;
        title_bar_rect.origin.y = ns_window.frame().size.height - header_h;
        title_bar_container_view.setFrame(title_bar_rect);

        let mut superview1_rect = NSView::frame(&superview1);
        superview1_rect.size.height = header_h;
        superview1_rect.origin.y = 0.0;
        superview1.setFrame(superview1_rect);

        let space_between = if NSView::frame(&miniaturize).origin.x > close_rect.origin.x {
            NSView::frame(&miniaturize).origin.x - close_rect.origin.x
        } else {
            20.0
        };

        let mut window_buttons = vec![close, miniaturize];
        if let Some(zoom) = zoom {
            window_buttons.push(zoom);
        }

        let button_y = (header_h - close_rect.size.height) / 2.0;

        for (i, button) in window_buttons.into_iter().enumerate() {
            let mut rect = NSView::frame(&button);
            rect.origin.x = x + (i as f64 * space_between);
            rect.origin.y = button_y;
            button.setFrameOrigin(rect.origin);
            eprintln!("[TRAFFIC] button {} placed at x={}, y={}", i, rect.origin.x, rect.origin.y);
        }
    }
}

