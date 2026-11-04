use std::time::Duration;

use serde::Serialize;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_positioner::{Position as PlacerPosition, WindowExt};

use crate::models::{AppConfig, SidebarEdge, UiMode};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LayoutEvent {
    ui_mode: UiMode,
    floating_expanded: bool,
    sidebar_edge: SidebarEdge,
}

pub fn apply_mode(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;

    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());
    let scale = monitor
        .as_ref()
        .map(|value| value.scale_factor())
        .unwrap_or(1.0);
    let monitor_position = monitor
        .as_ref()
        .map(|value| *value.position())
        .unwrap_or(PhysicalPosition::new(0, 0));
    let monitor_size = monitor
        .as_ref()
        .map(|value| *value.size())
        .unwrap_or_else(|| tauri::PhysicalSize::new(1920, 1080));
    let monitor_width = monitor_size.width as f64 / scale;
    let monitor_height = monitor_size.height as f64 / scale;

    let (width, height) = match config.ui_mode {
        UiMode::Floating => (
            404.0,
            if config.floating_expanded {
                (monitor_height - 120.0).clamp(420.0, 780.0)
            } else {
                470.0
            },
        ),
        UiMode::Sidebar => (372.0, (monitor_height - 96.0).max(420.0)),
        UiMode::Tray => (420.0, (monitor_height - 140.0).clamp(420.0, 680.0)),
    };

    window
        .set_size(LogicalSize::new(width, height))
        .map_err(|error| format!("无法调整窗口尺寸: {error}"))?;
    window
        .set_resizable(false)
        .map_err(|error| format!("无法设置窗口尺寸: {error}"))?;
    window
        .set_always_on_top(config.always_on_top || config.ui_mode == UiMode::Tray)
        .map_err(|error| format!("无法设置窗口置顶: {error}"))?;
    window
        .set_skip_taskbar(true)
        .map_err(|error| format!("无法设置任务栏显示: {error}"))?;

    match config.ui_mode {
        UiMode::Floating => {
            let saved = config
                .floating_x
                .zip(config.floating_y)
                .filter(|(x, y)| position_is_visible(app, *x, *y, width * scale, height * scale));
            let position = saved.unwrap_or_else(|| {
                let x = monitor_position.x as f64 + (monitor_width - width - 20.0) * scale;
                let y = monitor_position.y as f64 + (monitor_height - height - 64.0) * scale;
                (x.round() as i32, y.round() as i32)
            });
            window
                .set_position(PhysicalPosition::new(position.0, position.1))
                .map_err(|error| format!("无法定位悬浮窗: {error}"))?;
        }
        UiMode::Sidebar => {
            let x = if config.sidebar_edge == SidebarEdge::Right {
                monitor_position.x as f64 + (monitor_width - width - 12.0) * scale
            } else {
                monitor_position.x as f64 + 12.0 * scale
            };
            let y = monitor_position.y as f64 + 48.0 * scale;
            window
                .set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32))
                .map_err(|error| format!("无法定位侧栏: {error}"))?;
        }
        UiMode::Tray => {
            if window
                .move_window(PlacerPosition::TrayBottomCenter)
                .is_err()
            {
                let x = monitor_position.x as f64 + (monitor_width - width - 16.0) * scale;
                let y = monitor_position.y as f64 + (monitor_height - height - 64.0) * scale;
                window
                    .set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32))
                    .map_err(|error| format!("无法定位托盘面板: {error}"))?;
            }
        }
    }

    let _ = app.emit(
        "ui://layout",
        LayoutEvent {
            ui_mode: config.ui_mode,
            floating_expanded: config.floating_expanded,
            sidebar_edge: config.sidebar_edge,
        },
    );

    Ok(())
}

pub fn show_main(app: &AppHandle) -> Result<(), String> {
    let config = app.state::<AppState>().config();
    apply_mode(app, &config)?;
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    window
        .show()
        .map_err(|error| format!("无法显示窗口: {error}"))?;
    window
        .set_focus()
        .map_err(|error| format!("无法聚焦窗口: {error}"))?;
    Ok(())
}

pub fn hide_main(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    window
        .hide()
        .map_err(|error| format!("无法隐藏窗口: {error}"))
}

pub fn toggle_main(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    match window.is_visible() {
        Ok(true) => hide_main(app),
        _ => show_main(app),
    }
}

pub fn hide_on_blur(app: &AppHandle) {
    let config = app.state::<AppState>().config();
    if config.ui_mode != UiMode::Tray {
        return;
    }
    if let Some(window) = app.get_webview_window("main") {
        if window.is_focused().unwrap_or(false) {
            return;
        }
        let _ = window.hide();
    }
}

pub fn open_settings(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window
            .show()
            .map_err(|error| format!("无法显示设置窗口: {error}"))?;
        window
            .set_focus()
            .map_err(|error| format!("无法聚焦设置窗口: {error}"))?;
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("AI Usage Dock · Settings")
        .inner_size(600.0, 780.0)
        .min_inner_size(520.0, 600.0)
        .resizable(true)
        .center()
        .visible(true)
        .build()
        .map_err(|error| format!("无法创建设置窗口: {error}"))?;

    window
        .set_focus()
        .map_err(|error| format!("无法聚焦设置窗口: {error}"))?;
    Ok(())
}

pub fn remember_floating_position(app: &AppHandle) {
    let state = app.state::<AppState>();
    let config = state.config();
    if config.ui_mode != UiMode::Floating {
        return;
    }
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let Ok(position) = window.outer_position() else {
        return;
    };

    {
        let mut last_save = state
            .last_position_save
            .lock()
            .expect("position lock poisoned");
        if let Some(last) = *last_save {
            if last.elapsed() < Duration::from_secs(1) {
                return;
            }
        }
        *last_save = Some(std::time::Instant::now());
    }

    let mut config = config;
    config.floating_x = Some(position.x);
    config.floating_y = Some(position.y);
    state.replace_config(config.clone());
    let _ = crate::config::save(app, &config);
}

fn position_is_visible(app: &AppHandle, x: i32, y: i32, width: f64, height: f64) -> bool {
    let Ok(monitors) = app.available_monitors() else {
        return false;
    };
    monitors.iter().any(|monitor| {
        let position = monitor.position();
        let size = monitor.size();
        let left = position.x as f64;
        let top = position.y as f64;
        let right = left + size.width as f64;
        let bottom = top + size.height as f64;
        let window_left = x as f64;
        let window_top = y as f64;
        let window_right = window_left + width;
        let window_bottom = window_top + height;
        window_left >= left - 8.0
            && window_top >= top - 8.0
            && window_right <= right + 8.0
            && window_bottom <= bottom + 8.0
    })
}
