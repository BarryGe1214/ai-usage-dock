use tauri::{AppHandle, Emitter, Manager};

use crate::models::{
    AccountConfig, AccountView, AppConfig, Bootstrap, CodexCheck, ConfigView, PreferencesUpdate,
    ProviderKind, UiMode,
};
use crate::state::{self, AppState};
use crate::window;
use crate::{config, credentials, providers, tray};

#[tauri::command]
pub fn bootstrap(app: AppHandle) -> Result<Bootstrap, String> {
    let state = app.state::<AppState>();
    let config = state.config();
    Ok(Bootstrap {
        config: ConfigView::from(&config),
        accounts: accounts_view(&config),
        snapshots: state.snapshot_list(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        codex_path: providers::codex::locate_codex().map(|path| path.display().to_string()),
    })
}

#[tauri::command]
pub async fn update_preferences(
    app: AppHandle,
    preferences: PreferencesUpdate,
) -> Result<ConfigView, String> {
    let state = app.state::<AppState>();
    let previous = state.config();
    let mut config = previous.clone();

    config.language = preferences.language;
    config.theme = preferences.theme;
    config.ui_mode = preferences.ui_mode;
    config.sidebar_edge = preferences.sidebar_edge;
    config.refresh_interval_secs = preferences.refresh_interval_secs;
    config.always_on_top = preferences.always_on_top;
    config.start_at_login = preferences.start_at_login;
    config.floating_expanded = preferences.floating_expanded;
    config.funcloud_base_url = preferences.funcloud_base_url;
    config.normalize();

    state.replace_config(config.clone());
    config::save(&app, &config)?;

    if previous.language != config.language {
        let _ = tray::update_menu(&app);
    }
    if previous.ui_mode != config.ui_mode
        || previous.sidebar_edge != config.sidebar_edge
        || previous.floating_expanded != config.floating_expanded
        || previous.always_on_top != config.always_on_top
    {
        window::apply_mode(&app, &config)?;
    }
    if previous.funcloud_base_url != config.funcloud_base_url {
        let app_handle = app.clone();
        tauri::async_runtime::spawn(async move {
            let _ = state::refresh_all(app_handle).await;
        });
    }
    if previous.refresh_interval_secs != config.refresh_interval_secs {
        state.refresh_notify.notify_one();
    }

    apply_autostart(&app, config.start_at_login);

    let view = ConfigView::from(&config);
    let _ = app.emit("config://updated", &view);
    Ok(view)
}

#[tauri::command]
pub async fn add_funcloud_account(
    app: AppHandle,
    label: String,
    api_key: String,
) -> Result<AccountView, String> {
    let label = label.trim();
    if label.is_empty() {
        return Err("请输入账号名称".to_string());
    }
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err("请输入 Funcloud API Key".to_string());
    }

    let id = format!("funcloud-{}", uuid::Uuid::new_v4());
    let key = api_key.to_string();
    let secret_id = id.clone();
    tokio::task::spawn_blocking(move || credentials::set_secret(&secret_id, &key))
        .await
        .map_err(|error| format!("保存凭据任务失败: {error}"))??;

    let state = app.state::<AppState>();
    let mut config = state.config();
    let account = AccountConfig {
        id: id.clone(),
        provider: ProviderKind::Funcloud,
        label: label.to_string(),
        enabled: true,
    };
    config.accounts.push(account.clone());
    config.normalize();
    state.replace_config(config.clone());
    state
        .snapshots
        .lock()
        .expect("snapshot lock poisoned")
        .insert(account.id.clone(), crate::models::UsageSnapshot::loading(&account));
    config::save(&app, &config)?;

    emit_accounts(&app, &config);
    let snapshots = state.snapshot_list();
    let _ = app.emit("usage://updated", &snapshots);
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = state::refresh_one(app_handle, id).await;
    });

    Ok(AccountView {
        id: account.id,
        provider: account.provider,
        label: account.label,
        enabled: account.enabled,
        has_secret: true,
    })
}

#[tauri::command]
pub async fn update_funcloud_account(
    app: AppHandle,
    id: String,
    label: String,
    api_key: Option<String>,
) -> Result<AccountView, String> {
    let label = label.trim();
    if label.is_empty() {
        return Err("请输入账号名称".to_string());
    }

    if let Some(api_key) = api_key.filter(|value| !value.trim().is_empty()) {
        let key = api_key.trim().to_string();
        let secret_id = id.clone();
        tokio::task::spawn_blocking(move || credentials::set_secret(&secret_id, &key))
            .await
            .map_err(|error| format!("保存凭据任务失败: {error}"))??;
    }

    let state = app.state::<AppState>();
    let mut config = state.config();
    let account = config
        .accounts
        .iter_mut()
        .find(|account| account.id == id)
        .ok_or_else(|| "账号不存在".to_string())?;
    if account.provider != ProviderKind::Funcloud {
        return Err("Codex 账号名称由本机登录管理".to_string());
    }
    account.label = label.to_string();
    let view = AccountView {
        id: account.id.clone(),
        provider: account.provider,
        label: account.label.clone(),
        enabled: account.enabled,
        has_secret: credentials::has_secret(&id),
    };

    state.replace_config(config.clone());
    config::save(&app, &config)?;
    emit_accounts(&app, &config);

    let app_handle = app.clone();
    let refresh_id = id.clone();
    tauri::async_runtime::spawn(async move {
        let _ = state::refresh_one(app_handle, refresh_id).await;
    });

    Ok(view)
}

#[tauri::command]
pub async fn remove_account(app: AppHandle, id: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut config = state.config();
    let account = config
        .accounts
        .iter()
        .find(|account| account.id == id)
        .cloned()
        .ok_or_else(|| "账号不存在".to_string())?;

    if account.provider == ProviderKind::Codex {
        return Err("Codex 账号由本机登录管理，不能删除，只能禁用".to_string());
    }

    let secret_id = id.clone();
    tokio::task::spawn_blocking(move || credentials::delete_secret(&secret_id))
        .await
        .map_err(|error| format!("删除凭据任务失败: {error}"))??;

    config.accounts.retain(|account| account.id != id);
    state.replace_config(config.clone());
    state
        .snapshots
        .lock()
        .expect("snapshot lock poisoned")
        .remove(&id);
    config::save(&app, &config)?;

    emit_accounts(&app, &config);
    let snapshots = state.snapshot_list();
    let _ = app.emit("usage://updated", &snapshots);
    tray::update_tooltip(&app, &snapshots);
    Ok(())
}

#[tauri::command]
pub async fn set_account_enabled(
    app: AppHandle,
    id: String,
    enabled: bool,
) -> Result<ConfigView, String> {
    let state = app.state::<AppState>();
    let mut config = state.config();
    let account = config
        .accounts
        .iter_mut()
        .find(|account| account.id == id)
        .ok_or_else(|| "账号不存在".to_string())?;
    account.enabled = enabled;
    state.replace_config(config.clone());
    config::save(&app, &config)?;

    emit_accounts(&app, &config);
    let view = ConfigView::from(&config);
    let _ = app.emit("config://updated", &view);

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = state::refresh_one(app_handle, id).await;
    });

    Ok(view)
}

#[tauri::command]
pub async fn refresh_all(app: AppHandle) -> Result<Vec<crate::models::UsageSnapshot>, String> {
    Ok(state::refresh_all(app).await)
}

#[tauri::command]
pub async fn refresh_account(
    app: AppHandle,
    id: String,
) -> Result<Vec<crate::models::UsageSnapshot>, String> {
    Ok(state::refresh_one(app, id).await)
}

#[tauri::command]
pub fn set_ui_mode(app: AppHandle, mode: UiMode) -> Result<ConfigView, String> {
    let state = app.state::<AppState>();
    let mut config = state.config();
    config.ui_mode = mode;
    state.replace_config(config.clone());
    config::save(&app, &config)?;
    window::apply_mode(&app, &config)?;
    let _ = window::show_main(&app);
    let view = ConfigView::from(&config);
    let _ = app.emit("config://updated", &view);
    Ok(view)
}

#[tauri::command]
pub fn set_floating_expanded(app: AppHandle, expanded: bool) -> Result<ConfigView, String> {
    let state = app.state::<AppState>();
    let mut config = state.config();
    config.floating_expanded = expanded;
    state.replace_config(config.clone());
    config::save(&app, &config)?;
    if config.ui_mode == UiMode::Floating {
        window::apply_mode(&app, &config)?;
    }
    let view = ConfigView::from(&config);
    let _ = app.emit("config://updated", &view);
    Ok(view)
}

#[tauri::command]
pub fn hide_main(app: AppHandle) -> Result<(), String> {
    window::hide_main(&app)
}

#[tauri::command]
pub fn show_main(app: AppHandle) -> Result<(), String> {
    window::show_main(&app)
}

#[tauri::command]
pub fn open_settings(app: AppHandle) -> Result<(), String> {
    window::open_settings(&app)
}

#[tauri::command]
pub fn close_settings(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window
            .close()
            .map_err(|error| format!("无法关闭设置窗口: {error}"))?;
    }
    Ok(())
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    exit_app(&app);
}

pub fn exit_app(app: &AppHandle) {
    app.exit(0);
    // app.exit 依赖事件循环收尾；若被阻塞，兜底强制结束进程。
    let _ = std::thread::Builder::new().spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(1200));
        std::process::exit(0);
    });
}

#[tauri::command]
pub async fn check_codex() -> Result<CodexCheck, String> {
    Ok(providers::codex::check().await)
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("只允许打开 http/https 链接".to_string());
    }
    open::that(url).map_err(|error| format!("无法打开链接: {error}"))
}

fn emit_accounts(app: &AppHandle, config: &AppConfig) {
    let _ = app.emit("accounts://updated", accounts_view(config));
}

fn accounts_view(config: &AppConfig) -> Vec<AccountView> {
    config
        .accounts
        .iter()
        .map(|account| AccountView {
            id: account.id.clone(),
            provider: account.provider,
            label: account.label.clone(),
            enabled: account.enabled,
            has_secret: if account.provider == ProviderKind::Funcloud {
                credentials::has_secret(&account.id)
            } else {
                true
            },
        })
        .collect()
}

fn apply_autostart(app: &AppHandle, enabled: bool) {
    #[cfg(not(debug_assertions))]
    {
        use tauri_plugin_autostart::ManagerExt;
        let manager = app.autolaunch();
        if enabled {
            let _ = manager.enable();
        } else {
            let _ = manager.disable();
        }
    }
    #[cfg(debug_assertions)]
    {
        let _ = (app, enabled);
    }
}
