use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::models::AppConfig;

const CONFIG_FILE: &str = "config.json";

pub fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法获取配置目录: {error}"))?;
    Ok(dir.join(CONFIG_FILE))
}

pub fn load(app: &AppHandle) -> AppConfig {
    let Ok(path) = config_path(app) else {
        return AppConfig::default();
    };

    let mut config = match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str::<AppConfig>(&raw).unwrap_or_else(|error| {
            eprintln!("failed to parse config at {}: {error}", path.display());
            AppConfig::default()
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => AppConfig::default(),
        Err(error) => {
            eprintln!("failed to read config at {}: {error}", path.display());
            AppConfig::default()
        }
    };

    config.normalize();
    config
}

pub fn save(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = config_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("无法创建配置目录: {error}"))?;
    }
    let raw =
        serde_json::to_string_pretty(config).map_err(|error| format!("无法序列化配置: {error}"))?;
    fs::write(&path, format!("{raw}\n")).map_err(|error| format!("无法写入配置: {error}"))
}
