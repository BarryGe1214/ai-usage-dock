use tauri::menu::{Menu, MenuBuilder, MenuItem, SubmenuBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

use crate::config;
use crate::models::{ConfigView, Language, ProviderKind, SnapshotStatus, UiMode, UsageSnapshot};
use crate::state::{self, AppState};
use crate::window;

const TRAY_ID: &str = "main";

pub fn setup(app: &AppHandle) -> Result<(), String> {
    let menu = build_menu(app)?;
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("AI Usage Dock")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            match id {
                "show" => {
                    let _ = window::toggle_main(app);
                }
                "refresh" => {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = state::refresh_all(app).await;
                    });
                }
                "mode_floating" => set_mode(app, UiMode::Floating),
                "mode_sidebar" => set_mode(app, UiMode::Sidebar),
                "mode_tray" => set_mode(app, UiMode::Tray),
                "settings" => {
                    let _ = window::open_settings(app);
                }
                "quit" => crate::commands::exit_app(app),
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            let app = tray.app_handle();
            tauri_plugin_positioner::on_tray_event(app, &event);
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = window::toggle_main(app);
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder
        .build(app)
        .map_err(|error| format!("无法创建系统托盘: {error}"))?;
    Ok(())
}

pub fn update_menu(app: &AppHandle) -> Result<(), String> {
    let menu = build_menu(app)?;
    let tray = app
        .tray_by_id(TRAY_ID)
        .ok_or_else(|| "系统托盘不存在".to_string())?;
    tray.set_menu(Some(menu))
        .map_err(|error| format!("无法更新托盘菜单: {error}"))
}

fn build_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, String> {
    let config = app.state::<AppState>().config();
    let zh = config.language == Language::Zh;

    let show = MenuItem::with_id(
        app,
        "show",
        if zh { "显示 / 隐藏" } else { "Show / Hide" },
        true,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    let refresh = MenuItem::with_id(
        app,
        "refresh",
        if zh { "立即刷新" } else { "Refresh now" },
        true,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    let mode_floating = MenuItem::with_id(
        app,
        "mode_floating",
        if zh { "悬浮窗" } else { "Floating window" },
        true,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    let mode_sidebar = MenuItem::with_id(
        app,
        "mode_sidebar",
        if zh { "贴边侧栏" } else { "Docked sidebar" },
        true,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    let mode_tray = MenuItem::with_id(
        app,
        "mode_tray",
        if zh { "任务栏组件" } else { "Taskbar widget" },
        true,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    let settings = MenuItem::with_id(
        app,
        "settings",
        if zh { "设置..." } else { "Settings..." },
        true,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    let quit = MenuItem::with_id(
        app,
        "quit",
        if zh { "退出" } else { "Quit" },
        true,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;

    let mode_menu = SubmenuBuilder::new(app, if zh { "显示形态" } else { "Display mode" })
        .item(&mode_floating)
        .item(&mode_sidebar)
        .item(&mode_tray)
        .build()
        .map_err(|error| error.to_string())?;

    MenuBuilder::new(app)
        .item(&show)
        .item(&refresh)
        .separator()
        .item(&mode_menu)
        .separator()
        .item(&settings)
        .item(&quit)
        .build()
        .map_err(|error| error.to_string())
}

fn set_mode(app: &AppHandle, mode: UiMode) {
    let state = app.state::<AppState>();
    let mut config = state.config();
    config.ui_mode = mode;
    state.replace_config(config.clone());
    let _ = config::save(app, &config);
    let _ = window::apply_mode(app, &config);
    let _ = app.emit("config://updated", ConfigView::from(&config));
    let _ = window::show_main(app);
}

pub fn update_tooltip(app: &AppHandle, snapshots: &[UsageSnapshot]) {
    let zh = app.state::<AppState>().config().language == Language::Zh;
    let mut parts = Vec::new();

    let codex_remaining = snapshots
        .iter()
        .filter(|snapshot| {
            snapshot.provider == ProviderKind::Codex && snapshot.status == SnapshotStatus::Ok
        })
        .flat_map(|snapshot| snapshot.windows.iter().map(|window| 100.0 - window.percent))
        .reduce(f64::min);
    if let Some(remaining) = codex_remaining {
        parts.push(format!("Codex {:.0}%", remaining.clamp(0.0, 100.0)));
    }

    let today_cost = snapshots
        .iter()
        .filter(|snapshot| {
            snapshot.provider == ProviderKind::Funcloud && snapshot.status == SnapshotStatus::Ok
        })
        .filter_map(|snapshot| snapshot.summary.as_ref().map(|summary| summary.today_cost))
        .sum::<f64>();
    let funcloud_count = snapshots
        .iter()
        .filter(|snapshot| {
            snapshot.provider == ProviderKind::Funcloud && snapshot.status == SnapshotStatus::Ok
        })
        .count();
    if funcloud_count > 0 {
        parts.push(format!("Funcloud ${today_cost:.2}"));
    }

    let errors = snapshots
        .iter()
        .filter(|snapshot| snapshot.status == SnapshotStatus::Error)
        .count();
    if errors > 0 {
        parts.push(if zh {
            format!("{errors} 个账号异常")
        } else {
            format!("{errors} error(s)")
        });
    }

    let tooltip = if parts.is_empty() {
        "AI Usage Dock".to_string()
    } else {
        format!("AI Usage Dock · {}", parts.join(" · "))
    };

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(tooltip));
    }
}
