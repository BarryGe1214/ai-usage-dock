export type Language = "zh" | "en";
export type Theme = "system" | "dark" | "light";
export type UiMode = "floating" | "sidebar" | "tray";
export type SidebarEdge = "left" | "right";
export type ProviderKind = "funcloud" | "codex";
export type SnapshotStatus = "ok" | "error" | "loading" | "disabled";

export interface AccountView {
  id: string;
  provider: ProviderKind;
  label: string;
  enabled: boolean;
  hasSecret: boolean;
}

export interface QuotaWindow {
  id: string;
  windowSec: number;
  enabled: boolean;
  limit: number;
  used: number;
  remaining: number;
  percent: number;
  resetsAt?: string | null;
}

export interface FuncloudSummary {
  todayCost: number;
  rpm: number;
  tpm: number;
  totalReqCnt: number;
  totalInputs: number;
  totalOutputs: number;
  totalTokens: number;
  totalCacheWrites: number;
  totalCacheReads: number;
  totalCost: number;
  avgLatencyMs: number;
}

export interface DailyUsage {
  date: string;
  reqCnt: number;
  inputs: number;
  outputs: number;
  cacheReads: number;
  cacheWrites: number;
  cost: number;
}

export interface ModelUsage {
  model: string;
  reqCnt: number;
  inputs: number;
  outputs: number;
  totalTokens: number;
  cost: number;
}

export interface CodexInfo {
  planType?: string | null;
  limitName?: string | null;
  reachedType?: string | null;
  credits?: unknown;
}

export interface UsageSnapshot {
  accountId: string;
  provider: ProviderKind;
  label: string;
  status: SnapshotStatus;
  fetchedAt: string;
  error?: string | null;
  windows: QuotaWindow[];
  summary?: FuncloudSummary | null;
  daily: DailyUsage[];
  models: ModelUsage[];
  codex?: CodexInfo | null;
}

export interface AppConfig {
  language: Language;
  theme: Theme;
  uiMode: UiMode;
  sidebarEdge: SidebarEdge;
  refreshIntervalSecs: number;
  alwaysOnTop: boolean;
  startAtLogin: boolean;
  floatingExpanded: boolean;
  funcloudBaseUrl: string;
}

export interface Bootstrap {
  config: AppConfig;
  accounts: AccountView[];
  snapshots: UsageSnapshot[];
  version: string;
  platform: string;
  codexPath?: string | null;
}

export interface CodexCheck {
  installed: boolean;
  path?: string | null;
  loggedIn: boolean;
  planType?: string | null;
  error?: string | null;
}

export interface PreferencesUpdate extends AppConfig {}

export interface LayoutEvent {
  uiMode: UiMode;
  floatingExpanded: boolean;
  sidebarEdge: SidebarEdge;
}

export interface ViewEvent {
  view: "dock" | "settings";
}

export interface UpdateInfo {
  version: string;
  currentVersion: string;
  notes?: string | null;
}

export type UpdatePhase =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "ready"
  | "uptodate"
  | "error";

export interface UpdateState {
  info: UpdateInfo | null;
  phase: UpdatePhase;
  progress: number;
  error?: string | null;
}

export interface UpdateProgress {
  downloaded: number;
  total?: number | null;
}

export type WidgetInstallPhase =
  | "idle"
  | "checking"
  | "downloading"
  | "installing"
  | "done"
  | "error"
  | "unsupported";

export interface WidgetStatus {
  supported: boolean;
  installed: boolean;
  installedVersion?: string | null;
  targetVersion: string;
  phase: WidgetInstallPhase;
  downloaded: number;
  total?: number | null;
  error?: string | null;
}
