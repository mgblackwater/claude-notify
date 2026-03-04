mod focus;
mod server;
mod settings;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
    WebviewUrl,
    webview::WebviewWindowBuilder,
};

fn urlencoding(s: &str) -> String {
    let mut result = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(b as char);
            }
            b' ' => result.push_str("%20"),
            _ => result.push_str(&format!("%{:02X}", b)),
        }
    }
    result
}

use settings::{get_settings, reset_settings, update_settings, Settings, SettingsState};

static TOAST_COUNTER: AtomicU32 = AtomicU32::new(0);

#[tauri::command]
fn test_notification(app: tauri::AppHandle) {
    show_toast_window(&app, "Claude Notify", "Test", "Everything is working!");
}

#[tauri::command]
fn activate_terminal() {
    focus::activate_terminal_window();
}

pub fn show_toast_window(app: &tauri::AppHandle, title: &str, project: &str, message: &str) {
    let id = TOAST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let label = format!("toast-{}", id);

    // Encode data as URL query params so no timing issues
    let params = format!(
        "toast.html?title={}&project={}&message={}",
        urlencoding(title),
        urlencoding(project),
        urlencoding(message),
    );

    let builder = WebviewWindowBuilder::new(
        app,
        &label,
        WebviewUrl::App(params.into()),
    )
    .title("")
    .inner_size(400.0, 160.0)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .focused(false)
    .visible(false);  // Start hidden, show after positioning

    match builder.build() {
        Ok(window) => {
            // Position bottom-right, then show
            if let Ok(Some(monitor)) = window.primary_monitor() {
                let size = monitor.size();
                let scale = monitor.scale_factor();
                let x = (size.width as f64 / scale) - 420.0;
                let y = (size.height as f64 / scale) - 200.0;
                let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
            }
            let _ = window.show();
        }
        Err(e) => {
            log::error!("Failed to create toast window: {}", e);
        }
    }
}

pub fn run() {
    env_logger::init();

    let settings = Settings::load();
    let settings_state: SettingsState = Mutex::new(settings.clone());
    let settings_arc = Arc::new(Mutex::new(settings));

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(settings_state)
        .invoke_handler(tauri::generate_handler![
            get_settings,
            update_settings,
            reset_settings,
            test_notification,
            activate_terminal,
        ])
        .setup(move |app| {
            // Build tray menu
            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let test_item = MenuItem::with_id(app, "test", "Test Notification", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&settings_item, &test_item, &quit_item])?;

            // Build tray icon
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => {
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move {
                            open_settings_window(&app);
                        });
                    }
                    "test" => {
                        show_toast_window(app, "Claude Notify", "Test", "Everything is working!");
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle().clone();
                        tauri::async_runtime::spawn(async move {
                            open_settings_window(&app);
                        });
                    }
                })
                .build(app)?;

            // Start HTTP server in background
            let app_handle = app.handle().clone();
            let server_settings = settings_arc.clone();
            tauri::async_runtime::spawn(async move {
                server::start_server(app_handle, server_settings).await;
            });

            log::info!("Claude Notify started");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Claude Notify");
}

fn open_settings_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.set_focus();
        return;
    }

    let _ = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("Claude Notify — Settings")
        .inner_size(480.0, 580.0)
        .resizable(false)
        .center()
        .build();
}
