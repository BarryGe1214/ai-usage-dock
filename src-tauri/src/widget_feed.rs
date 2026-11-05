use std::fs;
use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::models::{Language, ProviderKind, SnapshotStatus, UsageSnapshot};

const FEED_VERSION: u32 = 1;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WidgetFeed {
    version: u32,
    language: Language,
    updated_at: String,
    codex_remaining_percent: Option<f64>,
    funcloud_today_cost: f64,
    funcloud_total_cost: f64,
    error_count: usize,
    account_count: usize,
    accounts: Vec<WidgetAccount>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WidgetAccount {
    label: String,
    provider: &'static str,
    status: &'static str,
    remaining_percent: Option<f64>,
    today_cost: Option<f64>,
    windows: Vec<WidgetWindow>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WidgetWindow {
    window_sec: i64,
    remaining_percent: f64,
    resets_at: Option<String>,
}

/// Writes a compact snapshot that the Windows widget provider reads.
/// Path: %LOCALAPPDATA%\com.barryge1214.aiusagedock\widget.json
pub fn write(app: &AppHandle, snapshots: &[UsageSnapshot]) {
    let Some(path) = feed_path(app) else {
        return;
    };
    let language = app
        .try_state::<crate::state::AppState>()
        .map(|state| state.config().language)
        .unwrap_or_default();
    let feed = build(snapshots, language);
    let Ok(raw) = serde_json::to_string_pretty(&feed) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    // Write to a temporary file first so the widget provider never reads a
    // partially written JSON document.
    let temp = path.with_extension("json.tmp");
    if fs::write(&temp, format!("{raw}\n")).is_ok() {
        let _ = fs::rename(&temp, &path);
    }
}

fn feed_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_local_data_dir()
        .ok()
        .map(|dir| dir.join("widget.json"))
}

fn build(snapshots: &[UsageSnapshot], language: Language) -> WidgetFeed {
    let codex_remaining = snapshots
        .iter()
        .filter(|snapshot| {
            snapshot.provider == ProviderKind::Codex && snapshot.status == SnapshotStatus::Ok
        })
        .flat_map(|snapshot| snapshot.windows.iter())
        .filter(|window| window.enabled)
        .map(remaining_percent)
        .reduce(f64::min);

    let funcloud = snapshots.iter().filter(|snapshot| {
        snapshot.provider == ProviderKind::Funcloud && snapshot.status == SnapshotStatus::Ok
    });
    let (funcloud_today_cost, funcloud_total_cost) =
        funcloud.fold((0.0, 0.0), |(today, total), snapshot| {
            match snapshot.summary.as_ref() {
                Some(summary) => (today + summary.today_cost, total + summary.total_cost),
                None => (today, total),
            }
        });

    let accounts = snapshots
        .iter()
        .map(|snapshot| {
            let provider = match snapshot.provider {
                ProviderKind::Codex => "codex",
                ProviderKind::Funcloud => "funcloud",
            };
            let status = match snapshot.status {
                SnapshotStatus::Ok => "ok",
                SnapshotStatus::Error => "error",
                SnapshotStatus::Loading => "loading",
                SnapshotStatus::Disabled => "disabled",
            };
            let remaining = if snapshot.provider == ProviderKind::Codex
                && snapshot.status == SnapshotStatus::Ok
            {
                snapshot
                    .windows
                    .iter()
                    .map(remaining_percent)
                    .reduce(f64::min)
            } else {
                None
            };
            WidgetAccount {
                label: snapshot.label.clone(),
                provider,
                status,
                remaining_percent: remaining,
                today_cost: snapshot.summary.as_ref().map(|summary| summary.today_cost),
                windows: snapshot
                    .windows
                    .iter()
                    .filter(|window| window.enabled)
                    .map(|window| WidgetWindow {
                        window_sec: window.window_sec,
                        remaining_percent: remaining_percent(window),
                        resets_at: window.resets_at.clone(),
                    })
                    .collect(),
            }
        })
        .collect::<Vec<_>>();

    WidgetFeed {
        version: FEED_VERSION,
        language,
        updated_at: chrono::Utc::now().to_rfc3339(),
        codex_remaining_percent: codex_remaining,
        funcloud_today_cost,
        funcloud_total_cost,
        error_count: snapshots
            .iter()
            .filter(|snapshot| snapshot.status == SnapshotStatus::Error)
            .count(),
        account_count: snapshots.len(),
        accounts,
    }
}

fn remaining_percent(window: &crate::models::QuotaWindow) -> f64 {
    if window.limit > 0.0 {
        (window.remaining / window.limit * 100.0).clamp(0.0, 100.0)
    } else {
        (100.0 - window.percent).clamp(0.0, 100.0)
    }
}
