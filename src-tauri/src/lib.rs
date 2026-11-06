mod commands;
mod config;
mod credentials;
mod models;
mod providers;
mod state;
mod tray;
mod widget_feed;
mod widget_install;
mod widget_install_status;
mod window;

use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = window::show_main(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let config = config::load(app.handle());
            let snapshots = state::load_cache(app.handle());
            let client = reqwest::Client::builder()
                .user_agent(concat!("ai-usage-dock/", env!("CARGO_PKG_VERSION")))
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|error| format!("无法初始化 HTTP 客户端: {error}"))?;

            app.manage(state::AppState::new(config.clone(), snapshots, client));
            app.manage(widget_install::WidgetInstaller::new());
            let initial_snapshots = app.state::<state::AppState>().snapshot_list();
            widget_feed::write(app.handle(), &initial_snapshots);
            tray::setup(app.handle())?;
            if let Err(error) = window::apply_mode(app.handle(), &config) {
                eprintln!("apply_mode failed: {error}");
            }

            let launched_at_login = std::env::args().any(|argument| argument == "--autostart");
            if !launched_at_login {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }

            #[cfg(not(debug_assertions))]
            {
                use tauri_plugin_autostart::ManagerExt;
                let manager = app.autolaunch();
                if config.start_at_login {
                    let _ = manager.enable();
                } else {
                    let _ = manager.disable();
                }
            }

            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            state::spawn_scheduler(app.handle().clone());
            widget_install::spawn_auto_install(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                if window.label() == "main" {
                    api.prevent_close();
                    let settings_open = window
                        .app_handle()
                        .state::<state::AppState>()
                        .settings_open
                        .load(std::sync::atomic::Ordering::SeqCst);
                    if settings_open {
                        let _ = window::close_settings(window.app_handle());
                    } else {
                        let _ = window.hide();
                    }
                }
            }
            WindowEvent::Moved(_) => {
                if window.label() == "main" {
                    window::remember_floating_position(&window.app_handle());
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::update_preferences,
            commands::add_funcloud_account,
            commands::update_funcloud_account,
            commands::remove_account,
            commands::set_account_enabled,
            commands::refresh_all,
            commands::refresh_account,
            commands::set_ui_mode,
            commands::set_floating_expanded,
            commands::hide_main,
            commands::show_main,
            commands::open_settings,
            commands::close_settings,
            commands::quit_app,
            commands::check_codex,
            commands::check_update,
            commands::install_update,
            commands::restart_app,
            commands::open_url,
            widget_install::widget_status,
            widget_install::install_widget,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AI Usage Dock");
}
