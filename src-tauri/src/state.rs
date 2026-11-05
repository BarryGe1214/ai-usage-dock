use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Mutex as AsyncMutex, Notify};

use crate::models::{
    AccountConfig, AppConfig, ProviderKind, SnapshotCache, SnapshotStatus, UsageSnapshot,
};
use crate::providers;
use crate::{credentials, tray};

const CACHE_VERSION: u32 = 1;
const CACHE_FILE: &str = "snapshots.json";

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub snapshots: Mutex<HashMap<String, UsageSnapshot>>,
    pub client: reqwest::Client,
    pub refresh_lock: AsyncMutex<()>,
    pub refresh_notify: Arc<Notify>,
    pub last_position_save: Mutex<Option<Instant>>,
    pub settings_open: AtomicBool,
}

impl AppState {
    pub fn new(config: AppConfig, snapshots: Vec<UsageSnapshot>, client: reqwest::Client) -> Self {
        let mut snapshot_map = snapshots
            .into_iter()
            .map(|snapshot| (snapshot.account_id.clone(), snapshot))
            .collect::<HashMap<_, _>>();
        for account in &config.accounts {
            snapshot_map
                .entry(account.id.clone())
                .or_insert_with(|| UsageSnapshot::loading(account));
        }

        Self {
            config: Mutex::new(config),
            snapshots: Mutex::new(snapshot_map),
            client,
            refresh_lock: AsyncMutex::new(()),
            refresh_notify: Arc::new(Notify::new()),
            last_position_save: Mutex::new(None),
            settings_open: AtomicBool::new(false),
        }
    }

    pub fn config(&self) -> AppConfig {
        self.config.lock().expect("config lock poisoned").clone()
    }

    pub fn replace_config(&self, config: AppConfig) {
        *self.config.lock().expect("config lock poisoned") = config;
    }

    pub fn refresh_interval_secs(&self) -> u64 {
        self.config
            .lock()
            .expect("config lock poisoned")
            .refresh_interval_secs
            .clamp(30, 3600)
    }

    pub fn snapshot_list(&self) -> Vec<UsageSnapshot> {
        let config = self.config();
        let snapshots = self.snapshots.lock().expect("snapshot lock poisoned");
        let mut result = Vec::with_capacity(config.accounts.len());
        for account in &config.accounts {
            if let Some(snapshot) = snapshots.get(&account.id) {
                result.push(snapshot.clone());
            }
        }
        result
    }
}

pub fn cache_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_data_dir()
        .ok()
        .map(|dir| dir.join(CACHE_FILE))
}

pub fn load_cache(app: &AppHandle) -> Vec<UsageSnapshot> {
    let Some(path) = cache_path(app) else {
        return Vec::new();
    };
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };
    match serde_json::from_str::<SnapshotCache>(&raw) {
        Ok(cache) if cache.version == CACHE_VERSION => cache.snapshots,
        _ => Vec::new(),
    }
}

fn save_cache(app: &AppHandle, snapshots: &[UsageSnapshot]) {
    let Some(path) = cache_path(app) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let cache = SnapshotCache {
        version: CACHE_VERSION,
        saved_at: chrono::Utc::now().to_rfc3339(),
        snapshots: snapshots.to_vec(),
    };
    if let Ok(raw) = serde_json::to_string_pretty(&cache) {
        let _ = fs::write(path, format!("{raw}\n"));
    }
}

pub fn spawn_scheduler(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let _ = refresh(app.clone(), None).await;

        loop {
            let interval = {
                let state = app.state::<AppState>();
                state.refresh_interval_secs()
            };
            let notify = {
                let state = app.state::<AppState>();
                state.refresh_notify.clone()
            };

            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(interval)) => {}
                _ = notify.notified() => {
                    continue;
                }
            }

            let _ = refresh(app.clone(), None).await;
        }
    });
}

pub async fn refresh_all(app: AppHandle) -> Vec<UsageSnapshot> {
    refresh(app, None).await
}

pub async fn refresh_one(app: AppHandle, account_id: String) -> Vec<UsageSnapshot> {
    refresh(app, Some(account_id)).await
}

async fn refresh(app: AppHandle, only_account: Option<String>) -> Vec<UsageSnapshot> {
    let state = app.state::<AppState>();
    let _guard = state.refresh_lock.lock().await;

    let config = state.config();
    let targets = config
        .accounts
        .iter()
        .filter(|account| {
            only_account
                .as_ref()
                .map(|id| id == &account.id)
                .unwrap_or(true)
        })
        .cloned()
        .collect::<Vec<_>>();

    if targets.is_empty() {
        return state.snapshot_list();
    }

    {
        let mut snapshots = state.snapshots.lock().expect("snapshot lock poisoned");
        for account in &targets {
            if account.enabled {
                let entry = snapshots
                    .entry(account.id.clone())
                    .or_insert_with(|| UsageSnapshot::loading(account));
                entry.status = SnapshotStatus::Loading;
                entry.label = account.label.clone();
            }
        }
    }

    let client = state.client.clone();
    let base_url = config.funcloud_base_url.clone();
    let previous = {
        let snapshots = state.snapshots.lock().expect("snapshot lock poisoned");
        targets
            .iter()
            .filter_map(|account| {
                snapshots
                    .get(&account.id)
                    .map(|snapshot| (account.id.clone(), snapshot.clone()))
            })
            .collect::<HashMap<_, _>>()
    };

    let futures = targets.into_iter().map(|account| {
        let client = client.clone();
        let base_url = base_url.clone();
        let previous = previous.get(&account.id).cloned();
        async move { fetch_account(account, client, base_url, previous).await }
    });

    let results = futures_util::future::join_all(futures).await;

    {
        let mut snapshots = state.snapshots.lock().expect("snapshot lock poisoned");
        for snapshot in &results {
            snapshots.insert(snapshot.account_id.clone(), snapshot.clone());
        }
        for account in &config.accounts {
            if !snapshots.contains_key(&account.id) {
                snapshots.insert(
                    account.id.clone(),
                    UsageSnapshot {
                        account_id: account.id.clone(),
                        provider: account.provider,
                        label: account.label.clone(),
                        status: SnapshotStatus::Disabled,
                        fetched_at: chrono::Utc::now().to_rfc3339(),
                        error: None,
                        windows: Vec::new(),
                        summary: None,
                        daily: Vec::new(),
                        models: Vec::new(),
                        codex: None,
                    },
                );
            }
        }
    }

    let list = state.snapshot_list();
    save_cache(&app, &list);
    crate::widget_feed::write(&app, &list);
    tray::update_tooltip(&app, &list);
    let _ = app.emit("usage://updated", &list);
    list
}

async fn fetch_account(
    account: AccountConfig,
    client: reqwest::Client,
    base_url: String,
    previous: Option<UsageSnapshot>,
) -> UsageSnapshot {
    if !account.enabled {
        let mut snapshot = previous.unwrap_or_else(|| UsageSnapshot {
            account_id: account.id.clone(),
            provider: account.provider,
            label: account.label.clone(),
            status: SnapshotStatus::Disabled,
            fetched_at: chrono::Utc::now().to_rfc3339(),
            error: None,
            windows: Vec::new(),
            summary: None,
            daily: Vec::new(),
            models: Vec::new(),
            codex: None,
        });
        snapshot.status = SnapshotStatus::Disabled;
        snapshot.label = account.label.clone();
        snapshot.fetched_at = chrono::Utc::now().to_rfc3339();
        snapshot.error = None;
        return snapshot;
    }

    let result = match account.provider {
        ProviderKind::Codex => providers::codex::fetch().await.map(|data| UsageSnapshot {
            account_id: account.id.clone(),
            provider: account.provider,
            label: account.label.clone(),
            status: SnapshotStatus::Ok,
            fetched_at: chrono::Utc::now().to_rfc3339(),
            error: None,
            windows: data.windows,
            summary: None,
            daily: Vec::new(),
            models: Vec::new(),
            codex: Some(data.info),
        }),
        ProviderKind::Funcloud => {
            let account_id = account.id.clone();
            let secret = tokio::task::spawn_blocking(move || credentials::get_secret(&account_id))
                .await
                .map_err(|error| format!("读取凭据任务失败: {error}"))
                .and_then(|result| result);

            match secret {
                Ok(api_key) => providers::funcloud::fetch(&client, &base_url, &api_key)
                    .await
                    .map(|data| UsageSnapshot {
                        account_id: account.id.clone(),
                        provider: account.provider,
                        label: account.label.clone(),
                        status: SnapshotStatus::Ok,
                        fetched_at: chrono::Utc::now().to_rfc3339(),
                        error: None,
                        windows: data.windows,
                        summary: Some(data.summary),
                        daily: data.daily,
                        models: data.models,
                        codex: None,
                    }),
                Err(error) => Err(error),
            }
        }
    };

    match result {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let mut snapshot = previous.unwrap_or_else(|| UsageSnapshot {
                account_id: account.id.clone(),
                provider: account.provider,
                label: account.label.clone(),
                status: SnapshotStatus::Error,
                fetched_at: chrono::Utc::now().to_rfc3339(),
                error: None,
                windows: Vec::new(),
                summary: None,
                daily: Vec::new(),
                models: Vec::new(),
                codex: None,
            });
            snapshot.status = SnapshotStatus::Error;
            snapshot.label = account.label.clone();
            snapshot.fetched_at = chrono::Utc::now().to_rfc3339();
            snapshot.error = Some(error);
            snapshot
        }
    }
}
