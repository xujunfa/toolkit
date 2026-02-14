mod commands;
mod db;
mod tray_text;

use std::sync::Arc;
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use commands::zenmux::ZenmuxPollingState;

#[tauri::command]
fn update_tray_title(app: tauri::AppHandle, title: String) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_title(Some(&title));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:toolkit.db", db::migrations())
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin({
            let toggle_main = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyT);
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    if shortcut == &toggle_main {
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                })
                .build()
        })
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).ok();
            let db_path = app_data_dir.join("toolkit.db");
            let db_path_str = db_path.to_string_lossy().to_string();

            let pool = tauri::async_runtime::block_on(db::create_pool(&db_path_str))
                .expect("failed to create database pool");
            app.manage(pool.clone());

            app.manage(ZenmuxPollingState {
                handle: Arc::new(tokio::sync::Mutex::new(None)),
            });

            // Auto-start ZenMux polling if config exists
            let app_handle = app.handle().clone();
            let pool_clone = pool.clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(config) =
                    commands::zenmux::get_zenmux_config_by_pool(&pool_clone).await
                {
                    if !config.ctoken.is_empty() {
                        let handle =
                            commands::zenmux::spawn_polling_loop(app_handle.clone(), config);
                        let state = app_handle.state::<ZenmuxPollingState>();
                        let mut guard = state.handle.lock().await;
                        *guard = Some(handle);
                    }
                }
            });

            let tray_icon = app.default_window_icon().cloned().unwrap();
            TrayIconBuilder::with_id("main-tray")
                .icon(tray_icon)
                .icon_as_template(true)
                .tooltip("Toolkit")
                .title("Toolkit")
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            app.global_shortcut()
                .register(Shortcut::new(
                    Some(Modifiers::CONTROL | Modifiers::ALT),
                    Code::KeyT,
                ))
                .expect("failed to register Ctrl+Option+T");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::ping,
            commands::app::get_app_info,
            commands::settings::get_settings,
            commands::settings::set_settings,
            commands::profiles::list_profiles,
            commands::profiles::create_profile,
            commands::profiles::update_profile,
            commands::profiles::delete_profile,
            commands::profiles::sync_profiles_to_zshrc,
            commands::zenmux::get_zenmux_config,
            commands::zenmux::set_zenmux_config,
            commands::zenmux::get_zenmux_usage,
            commands::zenmux::start_zenmux_polling,
            commands::zenmux::stop_zenmux_polling,
            update_tray_title,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct WindowConfig {
        label: String,
        visible: Option<bool>,
    }

    #[derive(Debug, Deserialize)]
    struct AppConfig {
        windows: Vec<WindowConfig>,
    }

    #[derive(Debug, Deserialize)]
    struct TauriConfig {
        app: AppConfig,
    }

    #[test]
    fn startup_opens_main_and_keeps_timer_hidden() {
        let config: TauriConfig =
            serde_json::from_str(include_str!("../tauri.conf.json")).expect("parse tauri.conf");
        let main = config
            .app
            .windows
            .iter()
            .find(|w| w.label == "main")
            .expect("main window config exists");
        let timer = config
            .app
            .windows
            .iter()
            .find(|w| w.label == "timer")
            .expect("timer window config exists");
        assert_eq!(main.visible, Some(true));
        assert_eq!(timer.visible, Some(false));
    }

    #[test]
    fn setup_does_not_hide_main_window_on_boot() {
        let source = include_str!("lib.rs");
        assert!(!source.contains("if let Some(main_window) = app.get_webview_window(\"main\")"));
    }

}
