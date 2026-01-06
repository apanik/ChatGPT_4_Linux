#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod settings;

use anyhow::{Context, Result};
use settings::{reset_companion_position, AppSettings, SettingsManager};
use tauri::{
    api::shell::open,
    AppHandle, CustomMenuItem, LogicalSize, Manager, Menu, MenuItem, Size, Submenu, SystemTray,
    SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, WebviewWindow, WindowBuilder, WindowEvent,
    WindowUrl,
};
use url::Url;
use std::{fs, path::PathBuf};

const CHATGPT_URL: &str = "https://chatgpt.com";
const COMPANION_LABEL: &str = "companion";
const ABOUT_DISCLAIMER: &str = "Unofficial Linux client. Not affiliated with OpenAI.";

#[tauri::command]
fn get_settings(state: tauri::State<SettingsManager>) -> AppSettings {
    state.get()
}

#[tauri::command]
fn save_settings(
    state: tauri::State<SettingsManager>,
    settings: AppSettings,
    app: tauri::AppHandle,
) -> Result<AppSettings, String> {
    let updated = state
        .update(|s| {
            *s = settings.clone();
        })
        .map_err(|e| e.to_string())?;

    apply_hotkey(&app, &updated).map_err(|e| e.to_string())?;
    configure_autostart(&app, &updated).map_err(|e| e.to_string())?;
    Ok(updated)
}

#[tauri::command]
fn clear_local_data(state: tauri::State<SettingsManager>, app: tauri::AppHandle) -> Result<(), String> {
    state.clear().map_err(|e| e.to_string())?;
    for window in app.webview_windows().values() {
        let _ = window.clear_all_cookies();
        let _ = window.clear_all_browsing_data();
    }
    Ok(())
}

#[tauri::command]
fn reset_companion(state: tauri::State<SettingsManager>, app: tauri::AppHandle) -> Result<AppSettings, String> {
    let updated = reset_companion_position(&state).map_err(|e| e.to_string())?;
    place_companion(&app, &updated)?;
    Ok(updated)
}

fn configure_autostart(app: &tauri::AppHandle, settings: &AppSettings) -> Result<()> {
    let autostart_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("autostart");
    fs::create_dir_all(&autostart_dir).context("creating autostart dir")?;
    let desktop_file = autostart_dir.join("codex-chatgpt-client.desktop");
    if settings.preferences.launch_at_login {
        let exec_path = app
            .path_resolver()
            .app_executable_path()
            .unwrap_or_else(|_| PathBuf::from("codex-chatgpt-client"));
        let contents = format!(
            "[Desktop Entry]\nType=Application\nName=Codex ChatGPT Client\nExec={} --hidden\nX-GNOME-Autostart-enabled=true\n",
            exec_path.display()
        );
        fs::write(&desktop_file, contents).context("writing autostart entry")?;
    } else if desktop_file.exists() {
        let _ = fs::remove_file(&desktop_file);
    }
    Ok(())
}

fn apply_hotkey(app: &tauri::AppHandle, settings: &AppSettings) -> Result<()> {
    let mut manager = app.global_shortcut_manager();
    manager.unregister_all()?;
    let accelerator = settings.hotkey.accelerator.clone();
    let handle = app.clone();
    manager.register(accelerator.clone(), move || {
        let _ = focus_or_create_companion(&handle);
    })?;
    Ok(())
}

fn build_menu() -> Menu {
    let new_chat = CustomMenuItem::new("new_chat", "New Chat");
    let find = CustomMenuItem::new("find", "Find in conversation (Ctrl+F)");
    let toggle_links = CustomMenuItem::new("toggle_links", "Toggle external links");
    let about = CustomMenuItem::new("about", "About");
    let settings = CustomMenuItem::new("settings", "Settings");
    Menu::new().add_submenu(Submenu::new(
        "ChatGPT",
        Menu::new()
            .add_item(new_chat)
            .add_item(find)
            .add_item(toggle_links)
            .add_item(settings)
            .add_native_item(MenuItem::Separator)
            .add_item(about),
    ))
}

fn build_tray() -> SystemTray {
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("open_companion", "Open Companion"))
        .add_item(CustomMenuItem::new("open_main", "Open Main"))
        .add_item(CustomMenuItem::new("tray_new_chat", "New Chat"))
        .add_item(CustomMenuItem::new("tray_settings", "Settings"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit", "Quit"));
    SystemTray::new().with_menu(tray_menu)
}

fn setup_windows(app: &tauri::AppHandle, settings: &SettingsManager) -> Result<()> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| anyhow::anyhow!("main window missing"))?;

    main.on_window_event(|event| {
        if let WindowEvent::Focused(false) = event {
            // placeholder for focus tracking
        }
    });

    if app.get_webview_window(COMPANION_LABEL).is_none() {
        create_companion(app, &settings.get())?;
    }

    Ok(())
}

fn create_companion(app: &tauri::AppHandle, settings: &AppSettings) -> Result<()> {
    let mut builder = WindowBuilder::new(
        app,
        COMPANION_LABEL,
        WindowUrl::External(Url::parse(CHATGPT_URL)?),
    )
    .title("ChatGPT Companion")
    .resizable(true)
    .always_on_top(settings.companion.window.always_on_top)
    .inner_size(
        settings.companion.window.width,
        settings.companion.window.height,
    )
    .visible(false);

    if settings.companion.window.opacity < 1.0 {
        builder = builder.opacity(settings.companion.window.opacity);
    }

    let window = builder.build()?;
    place_companion(app, settings)?;
    window.on_window_event(|event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
        }
    });
    window.show()?;
    Ok(())
}

fn place_companion(app: &tauri::AppHandle, settings: &AppSettings) -> Result<()> {
    if let Some(window) = app.get_webview_window(COMPANION_LABEL) {
        window.set_always_on_top(settings.companion.window.always_on_top)?;
        window.set_size(Size::Logical(LogicalSize::new(
            settings.companion.window.width,
            settings.companion.window.height,
        )))?;
        if let Some(pos) = &settings.companion.window.position {
            window.set_position(tauri::LogicalPosition {
                x: pos.x,
                y: pos.y,
            })?;
        } else if let Ok(monitor) = window.current_monitor() {
            if let Some(monitor) = monitor {
                let size = monitor.size();
                let x = (size.width as f64 / 2.0) - (settings.companion.window.width / 2.0);
                let y = size.height as f64 - settings.companion.window.height - 40.0;
                window.set_position(tauri::LogicalPosition { x, y })?;
            }
        }
    }
    Ok(())
}

fn focus_or_create_companion(app: &tauri::AppHandle) -> Result<()> {
    if let Some(window) = app.get_webview_window(COMPANION_LABEL) {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }
    let settings = app.state::<SettingsManager>().get();
    create_companion(app, &settings)?;
    if let Some(window) = app.get_webview_window(COMPANION_LABEL) {
        window.set_focus()?;
    }
    Ok(())
}

fn open_settings_window(app: &tauri::AppHandle) -> Result<()> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }
    WindowBuilder::new(
        app,
        "settings",
        WindowUrl::App("settings.html".into()),
    )
    .title("Settings")
    .resizable(true)
    .inner_size(720.0, 640.0)
    .build()?;
    Ok(())
}

fn handle_menu_event(id: &str, app: &tauri::AppHandle, window: Option<&WebviewWindow>) {
    match id {
        "new_chat" | "tray_new_chat" => {
            if let Some(main) = app.get_webview_window("main") {
                let _ = main.eval(&format!("window.location.href='{}'", CHATGPT_URL));
            }
        }
        "find" => {
            if let Some(main) = app.get_webview_window("main") {
                let _ = main.eval(
                    "window.dispatchEvent(new KeyboardEvent('keydown', {key:'f', ctrlKey:true}));",
                );
            }
        }
        "settings" | "tray_settings" => {
            let _ = open_settings_window(app);
        }
        "toggle_links" => {
            let settings_manager = app.state::<SettingsManager>();
            let _ = settings_manager.update(|s| {
                s.preferences.open_links_in_browser = !s.preferences.open_links_in_browser;
            });
        }
        "about" => {
            let _ = tauri::api::dialog::message(
                window.cloned(),
                "About",
                format!(
                    "Codex ChatGPT Client\n{}\nLog in via the official ChatGPT site. Login credentials are never intercepted.",
                    ABOUT_DISCLAIMER
                ),
            );
        }
        "open_companion" => {
            let _ = focus_or_create_companion(app);
        }
        "open_main" => {
            if let Some(main) = app.get_webview_window("main") {
                let _ = main.show();
                let _ = main.set_focus();
            }
        }
        "quit" => {
            std::process::exit(0);
        }
        _ => {}
    }
}

fn configure_external_navigation(app: &AppHandle, allow_browser: bool) -> Result<()> {
    if !allow_browser {
        return Ok(());
    }
    for window in app.webview_windows().values() {
        window.with_webview(|webview| {
            webview
                .navigation_handler(|url| {
                    if url.as_str().starts_with(CHATGPT_URL) {
                        true
                    } else {
                        let _ = open(&webview, url.as_str().to_string(), None);
                        false
                    }
                })
                .unwrap();
        })?;
    }
    Ok(())
}

fn persist_companion_move(event: &tauri::WindowEvent) {
    if let tauri::WindowEvent::Moved(position) = event {
        if let Some(manager) = event.window().app_handle().try_state::<SettingsManager>() {
            let _ = manager.update(|s| {
                s.companion.window.position = Some(settings::WindowPosition {
                    x: position.x as f64,
                    y: position.y as f64,
                });
            });
        }
    }
}

fn main() {
    tauri::Builder::default()
        .menu(build_menu())
        .system_tray(build_tray())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            clear_local_data,
            reset_companion
        ])
        .setup(|app| {
            let settings_manager = SettingsManager::new(app)?;
            let settings_clone = settings_manager.get();
            app.manage(settings_manager);
            apply_hotkey(app, &settings_clone)?;
            configure_autostart(app, &settings_clone)?;
            setup_windows(app, app.state())?;
            configure_external_navigation(app, settings_clone.preferences.open_links_in_browser)?;
            Ok(())
        })
        .on_menu_event(|event| {
            let id = event.menu_item_id().to_string();
            let app_handle = event.window().app_handle();
            handle_menu_event(&id, &app_handle, Some(&event.window()));
        })
        .on_system_tray_event(|app, event| {
            if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                handle_menu_event(&id, app, None);
            }
        })
        .on_window_event(|event| {
            if event.window().label() == COMPANION_LABEL {
                persist_companion_move(event.event());
            }
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, _event| {});
}
