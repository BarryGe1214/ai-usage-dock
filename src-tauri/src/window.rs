use std::sync::atomic::Ordering;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition};

use crate::models::{AppConfig, SidebarEdge, UiMode};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LayoutEvent {
    ui_mode: UiMode,
    floating_expanded: bool,
    sidebar_edge: SidebarEdge,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ViewEvent {
    view: &'static str,
}

struct MonitorBounds {
    position: PhysicalPosition<i32>,
    width: f64,
    height: f64,
    scale: f64,
}

fn monitor_bounds(app: &AppHandle) -> MonitorBounds {
    let monitor = app
        .get_webview_window("main")
        .and_then(|window| window.current_monitor().ok().flatten())
        .or_else(|| {
            app.get_webview_window("main")
                .and_then(|window| window.primary_monitor().ok().flatten())
        });

    match monitor {
        Some(monitor) => {
            let scale = monitor.scale_factor();
            let size = *monitor.size();
            MonitorBounds {
                position: *monitor.position(),
                width: size.width as f64 / scale,
                height: size.height as f64 / scale,
                scale,
            }
        }
        None => MonitorBounds {
            position: PhysicalPosition::new(0, 0),
            width: 1920.0,
            height: 1080.0,
            scale: 1.0,
        },
    }
}

pub fn apply_mode(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    {
        let state = app.state::<AppState>();
        if state.settings_open.load(Ordering::SeqCst) {
            return Ok(());
        }
    }

    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;

    let bounds = monitor_bounds(app);

    let (width, height) = match config.ui_mode {
        UiMode::Floating => (
            404.0,
            if config.floating_expanded {
                (bounds.height - 120.0).clamp(420.0, 780.0)
            } else {
                470.0
            },
        ),
        UiMode::Sidebar => (372.0, (bounds.height - 96.0).max(420.0)),
        UiMode::Tray => (340.0, (bounds.height - 260.0).clamp(320.0, 440.0)),
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

    let scale = bounds.scale;
    let monitor_x = bounds.position.x as f64;
    let monitor_y = bounds.position.y as f64;

    match config.ui_mode {
        UiMode::Floating => {
            let saved = config
                .floating_x
                .zip(config.floating_y)
                .filter(|(x, y)| position_is_visible(app, *x, *y, width * scale, height * scale));
            let position = saved.unwrap_or_else(|| {
                let x = monitor_x + (bounds.width - width - 20.0) * scale;
                let y = monitor_y + (bounds.height - height - 64.0) * scale;
                (x.round() as i32, y.round() as i32)
            });
            window
                .set_position(PhysicalPosition::new(position.0, position.1))
                .map_err(|error| format!("无法定位悬浮窗: {error}"))?;
        }
        UiMode::Sidebar => {
            let x = if config.sidebar_edge == SidebarEdge::Right {
                monitor_x + (bounds.width - width - 12.0) * scale
            } else {
                monitor_x + 12.0 * scale
            };
            let y = monitor_y + 48.0 * scale;
            window
                .set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32))
                .map_err(|error| format!("无法定位侧栏: {error}"))?;
        }
        UiMode::Tray => {
            // 任务栏小组件：固定在屏幕右下角、任务栏上方，不拦截失焦隐藏。
            let x = monitor_x + (bounds.width - width - 16.0) * scale;
            let y = monitor_y + (bounds.height - height - 64.0) * scale;
            window
                .set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32))
                .map_err(|error| format!("无法定位任务栏小组件: {error}"))?;
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
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;

    let settings_open = {
        let state = app.state::<AppState>();
        state.settings_open.load(Ordering::SeqCst)
    };

    if !settings_open {
        let config = app.state::<AppState>().config();
        apply_mode(app, &config)?;
    }

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

pub fn open_settings(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    let state = app.state::<AppState>();
    let already_open = state.settings_open.swap(true, Ordering::SeqCst);

    if !already_open {
        let bounds = monitor_bounds(app);
        let width = (bounds.width - 96.0).clamp(520.0, 680.0);
        let height = (bounds.height - 120.0).clamp(560.0, 860.0);
        window
            .set_size(LogicalSize::new(width, height))
            .map_err(|error| format!("无法调整设置界面尺寸: {error}"))?;
        window
            .set_resizable(true)
            .map_err(|error| format!("无法设置设置界面尺寸: {error}"))?;
        window
            .set_always_on_top(true)
            .map_err(|error| format!("无法置顶设置界面: {error}"))?;

        let x = bounds.position.x as f64 + (bounds.width - width) / 2.0 * bounds.scale;
        let y = bounds.position.y as f64 + (bounds.height - height) / 2.0 * bounds.scale;
        window
            .set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32))
            .map_err(|error| format!("无法定位设置界面: {error}"))?;
    }

    window
        .show()
        .map_err(|error| format!("无法显示窗口: {error}"))?;
    window
        .set_focus()
        .map_err(|error| format!("无法聚焦窗口: {error}"))?;
    let _ = app.emit("ui://view", ViewEvent { view: "settings" });
    Ok(())
}

pub fn close_settings(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    if !state.settings_open.swap(false, Ordering::SeqCst) {
        return Ok(());
    }

    let config = state.config();
    apply_mode(app, &config)?;

    if let Some(window) = app.get_webview_window("main") {
        window
            .show()
            .map_err(|error| format!("无法显示窗口: {error}"))?;
        window
            .set_focus()
            .map_err(|error| format!("无法聚焦窗口: {error}"))?;
    }

    let _ = app.emit("ui://view", ViewEvent { view: "dock" });
    Ok(())
}

pub fn remember_floating_position(app: &AppHandle) {
    let state = app.state::<AppState>();
    let config = state.config();
    if config.ui_mode != UiMode::Floating {
        return;
    }
    if state.settings_open.load(Ordering::SeqCst) {
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
