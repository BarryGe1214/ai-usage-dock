use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Zh,
    En,
}

impl Default for Language {
    fn default() -> Self {
        Self::Zh
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Dark,
    Light,
}

impl Default for Theme {
    fn default() -> Self {
        Self::System
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiMode {
    Floating,
    Sidebar,
    Tray,
}

impl Default for UiMode {
    fn default() -> Self {
        Self::Floating
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SidebarEdge {
    Left,
    Right,
}

impl Default for SidebarEdge {
    fn default() -> Self {
        Self::Right
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Funcloud,
    Codex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountConfig {
    pub id: String,
    pub provider: ProviderKind,
    pub label: String,
    pub enabled: bool,
}

impl AccountConfig {
    pub fn codex_default() -> Self {
        Self {
            id: "codex-default".to_string(),
            provider: ProviderKind::Codex,
            label: "OpenAI Codex".to_string(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    pub language: Language,
    pub theme: Theme,
    pub ui_mode: UiMode,
    pub sidebar_edge: SidebarEdge,
    pub refresh_interval_secs: u64,
    pub always_on_top: bool,
    pub start_at_login: bool,
    pub floating_expanded: bool,
    pub floating_x: Option<i32>,
    pub floating_y: Option<i32>,
    pub funcloud_base_url: String,
    pub accounts: Vec<AccountConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: Language::default(),
            theme: Theme::default(),
            ui_mode: UiMode::default(),
            sidebar_edge: SidebarEdge::default(),
            refresh_interval_secs: 120,
            always_on_top: true,
            start_at_login: true,
            floating_expanded: false,
            floating_x: None,
            floating_y: None,
            funcloud_base_url: "https://api.funcloud.ai".to_string(),
            accounts: vec![AccountConfig::codex_default()],
        }
    }
}

impl AppConfig {
    pub fn normalize(&mut self) {
        self.refresh_interval_secs = self.refresh_interval_secs.clamp(30, 3600);
        if self.funcloud_base_url.trim().is_empty() {
            self.funcloud_base_url = "https://api.funcloud.ai".to_string();
        }
        self.funcloud_base_url = self
            .funcloud_base_url
            .trim()
            .trim_end_matches('/')
            .to_string();
        if !self
            .accounts
            .iter()
            .any(|a| a.provider == ProviderKind::Codex)
        {
            self.accounts.insert(0, AccountConfig::codex_default());
        }
        for account in &mut self.accounts {
            if account.label.trim().is_empty() {
                account.label = match account.provider {
                    ProviderKind::Codex => "OpenAI Codex".to_string(),
                    ProviderKind::Funcloud => "Funcloud".to_string(),
                };
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigView {
    pub language: Language,
    pub theme: Theme,
    pub ui_mode: UiMode,
    pub sidebar_edge: SidebarEdge,
    pub refresh_interval_secs: u64,
    pub always_on_top: bool,
    pub start_at_login: bool,
    pub floating_expanded: bool,
    pub funcloud_base_url: String,
}

impl From<&AppConfig> for ConfigView {
    fn from(value: &AppConfig) -> Self {
        Self {
            language: value.language,
            theme: value.theme,
            ui_mode: value.ui_mode,
            sidebar_edge: value.sidebar_edge,
            refresh_interval_secs: value.refresh_interval_secs,
            always_on_top: value.always_on_top,
            start_at_login: value.start_at_login,
            floating_expanded: value.floating_expanded,
            funcloud_base_url: value.funcloud_base_url.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountView {
    pub id: String,
    pub provider: ProviderKind,
    pub label: String,
    pub enabled: bool,
    pub has_secret: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotStatus {
    Ok,
    Error,
    Loading,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QuotaWindow {
    pub id: String,
    pub window_sec: i64,
    pub enabled: bool,
    pub limit: f64,
    pub used: f64,
    pub remaining: f64,
    pub percent: f64,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FuncloudSummary {
    pub today_cost: f64,
    pub rpm: f64,
    pub tpm: f64,
    pub total_req_cnt: f64,
    pub total_inputs: f64,
    pub total_outputs: f64,
    pub total_tokens: f64,
    pub total_cache_writes: f64,
    pub total_cache_reads: f64,
    pub total_cost: f64,
    pub avg_latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DailyUsage {
    pub date: String,
    pub req_cnt: f64,
    pub inputs: f64,
    pub outputs: f64,
    pub cache_reads: f64,
    pub cache_writes: f64,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsage {
    pub model: String,
    pub req_cnt: f64,
    pub inputs: f64,
    pub outputs: f64,
    pub cache_reads: f64,
    pub total_tokens: f64,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CodexInfo {
    pub plan_type: Option<String>,
    pub limit_name: Option<String>,
    pub reached_type: Option<String>,
    pub credits: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSnapshot {
    pub account_id: String,
    pub provider: ProviderKind,
    pub label: String,
    pub status: SnapshotStatus,
    pub fetched_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default)]
    pub windows: Vec<QuotaWindow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<FuncloudSummary>,
    #[serde(default)]
    pub daily: Vec<DailyUsage>,
    #[serde(default)]
    pub models: Vec<ModelUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex: Option<CodexInfo>,
}

impl UsageSnapshot {
    pub fn loading(account: &AccountConfig) -> Self {
        Self {
            account_id: account.id.clone(),
            provider: account.provider,
            label: account.label.clone(),
            status: SnapshotStatus::Loading,
            fetched_at: chrono::Utc::now().to_rfc3339(),
            error: None,
            windows: Vec::new(),
            summary: None,
            daily: Vec::new(),
            models: Vec::new(),
            codex: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub config: ConfigView,
    pub accounts: Vec<AccountView>,
    pub snapshots: Vec<UsageSnapshot>,
    pub version: String,
    pub platform: String,
    pub codex_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexCheck {
    pub installed: bool,
    pub path: Option<String>,
    pub logged_in: bool,
    pub plan_type: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreferencesUpdate {
    pub language: Language,
    pub theme: Theme,
    pub ui_mode: UiMode,
    pub sidebar_edge: SidebarEdge,
    pub refresh_interval_secs: u64,
    pub always_on_top: bool,
    pub start_at_login: bool,
    pub floating_expanded: bool,
    pub funcloud_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotCache {
    pub version: u32,
    pub saved_at: String,
    pub snapshots: Vec<UsageSnapshot>,
}
