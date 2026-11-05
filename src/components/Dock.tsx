import { useEffect, useState } from "react";
import {
  AlertTriangle,
  AppWindow,
  ChevronDown,
  ChevronUp,
  Cloud,
  PanelRight,
  PanelTop,
  Power,
  RefreshCw,
  Settings,
  Sparkles,
  X,
} from "lucide-react";

import type { Translator } from "../i18n";
import type {
  AccountView,
  AppConfig,
  Language,
  UiMode,
  UsageSnapshot,
} from "../types";
import {
  formatMoney,
  formatRelative,
  remainingPercent,
  severity,
} from "../lib/format";
import { AccountCard } from "./AccountCard";
import { Logo } from "./Logo";

interface DockProps {
  config: AppConfig;
  accounts: AccountView[];
  snapshots: UsageSnapshot[];
  language: Language;
  t: Translator;
  version: string;
  onRefresh: () => Promise<void>;
  onRefreshAccount: (id: string) => Promise<void>;
  onToggleAccount: (id: string, enabled: boolean) => Promise<void>;
  onOpenSettings: () => void;
  onHide: () => void;
  onQuit: () => void;
  onToggleExpand: () => void;
  onSetMode: (mode: UiMode) => void;
}

export function Dock({
  config,
  accounts,
  snapshots,
  language,
  t,
  version,
  onRefresh,
  onRefreshAccount,
  onToggleAccount,
  onOpenSettings,
  onHide,
  onQuit,
  onToggleExpand,
  onSetMode,
}: DockProps) {
  const [refreshing, setRefreshing] = useState(false);
  const [refreshError, setRefreshError] = useState<string | null>(null);
  const [, setTick] = useState(0);

  useEffect(() => {
    const timer = window.setInterval(() => setTick((value) => value + 1), 30_000);
    return () => window.clearInterval(timer);
  }, []);

  const detailed =
    config.uiMode === "sidebar" ||
    (config.uiMode === "floating" && config.floatingExpanded);
  const compact = !detailed;

  const orderedSnapshots = snapshots;

  const codexRemaining = snapshots
    .filter((snapshot) => snapshot.provider === "codex" && snapshot.status === "ok")
    .flatMap((snapshot) => snapshot.windows.map(remainingPercent))
    .reduce<number | null>(
      (minimum, value) => (minimum === null ? value : Math.min(minimum, value)),
      null,
    );
  const funcloudToday = snapshots
    .filter((snapshot) => snapshot.provider === "funcloud" && snapshot.status === "ok")
    .reduce((total, snapshot) => total + (snapshot.summary?.todayCost ?? 0), 0);
  const funcloudCount = snapshots.filter(
    (snapshot) => snapshot.provider === "funcloud" && snapshot.status === "ok",
  ).length;
  const errorCount = snapshots.filter((snapshot) => snapshot.status === "error").length;
  const allLoading =
    snapshots.length > 0 && snapshots.every((snapshot) => snapshot.status === "loading");
  const lastRefresh = snapshots
    .map((snapshot) => snapshot.fetchedAt)
    .sort()
    .at(-1);

  async function handleRefresh() {
    if (refreshing) return;
    setRefreshing(true);
    setRefreshError(null);
    try {
      await onRefresh();
    } catch (error) {
      setRefreshError(error instanceof Error ? error.message : String(error));
    } finally {
      setRefreshing(false);
    }
  }

  async function handleRetry(id: string) {
    try {
      await onRefreshAccount(id);
    } catch (error) {
      setRefreshError(error instanceof Error ? error.message : String(error));
    }
  }

  const modeButtons: Array<{ mode: UiMode; label: string; icon: typeof AppWindow }> = [
    { mode: "floating", label: t("modeFloating"), icon: AppWindow },
    { mode: "sidebar", label: t("modeSidebar"), icon: PanelRight },
    { mode: "tray", label: t("modeTray"), icon: PanelTop },
  ];

  return (
    <div className={`dock mode-${config.uiMode}${detailed ? " is-detailed" : " is-compact"}`}>
      <header className="dock-header">
        <div className="brand" data-tauri-drag-region>
          <Logo size={34} />
          <div className="brand-text" data-tauri-drag-region>
            <div className="brand-title">
              <h1>{t("appName")}</h1>
              <span className={`health-dot${errorCount > 0 ? " has-error" : ""}`} />
            </div>
            <span className="brand-subtitle">{t("tagline")}</span>
          </div>
        </div>
        <div className="header-actions">
          <button
            className={`icon-button${refreshing ? " spinning" : ""}`}
            onClick={handleRefresh}
            title={refreshing ? t("refreshing") : t("refresh")}
            aria-label={t("refresh")}
          >
            <RefreshCw size={15} />
          </button>
          <button
            className="icon-button"
            onClick={onOpenSettings}
            title={t("settings")}
            aria-label={t("settings")}
          >
            <Settings size={15} />
          </button>
          {config.uiMode === "floating" ? (
            <button
              className="icon-button"
              onClick={onToggleExpand}
              title={config.floatingExpanded ? t("collapse") : t("expand")}
              aria-label={config.floatingExpanded ? t("collapse") : t("expand")}
            >
              {config.floatingExpanded ? <ChevronDown size={15} /> : <ChevronUp size={15} />}
            </button>
          ) : null}
          <button
            className="icon-button subtle"
            onClick={onHide}
            title={t("hide")}
            aria-label={t("hide")}
          >
            <X size={15} />
          </button>
          <button
            className="icon-button danger"
            onClick={onQuit}
            title={t("quit")}
            aria-label={t("quit")}
          >
            <Power size={15} />
          </button>
        </div>
      </header>

      <div className="dock-summary">
        {codexRemaining !== null ? (
          <span className={`summary-pill tone-${severity(codexRemaining)}`}>
            <Sparkles size={12} />
            {t("codexRemaining", { value: Math.round(codexRemaining) })}
          </span>
        ) : null}
        {funcloudCount > 0 ? (
          <span className="summary-pill provider-funcloud">
            <Cloud size={12} />
            {t("funcloudToday", { value: formatMoney(funcloudToday) })}
          </span>
        ) : null}
        {errorCount > 0 ? (
          <span className="summary-pill tone-bad">
            <AlertTriangle size={12} />
            {t("issuesCount", { value: errorCount })}
          </span>
        ) : null}
        {allLoading ? (
          <span className="summary-pill muted">
            <RefreshCw size={12} className="spin" />
            {t("loadingShort")}
          </span>
        ) : null}
        {snapshots.length === 0 ? (
          <span className="summary-pill muted">{t("loading")}</span>
        ) : null}
      </div>

      <main className="dock-body">
        {refreshError ? (
          <div className="inline-alert">
            <AlertTriangle size={14} />
            <span>{refreshError}</span>
            <button className="text-button" onClick={handleRefresh}>
              {t("retry")}
            </button>
          </div>
        ) : null}

        {orderedSnapshots.length > 0 ? (
          orderedSnapshots.map((snapshot) => (
            <AccountCard
              key={snapshot.accountId}
              snapshot={snapshot}
              account={accounts.find((account) => account.id === snapshot.accountId)}
              language={language}
              t={t}
              mode={config.uiMode}
              compact={compact}
              detailed={detailed}
              onRetry={() => handleRetry(snapshot.accountId)}
              onToggleEnabled={(enabled) =>
                onToggleAccount(snapshot.accountId, enabled)
              }
            />
          ))
        ) : (
          <div className="empty-state">
            <div className="empty-icon">
              <Sparkles size={22} />
            </div>
            <h2>{t("noAccounts")}</h2>
            <p>{t("noAccountsHint")}</p>
            <button className="primary-button" onClick={onOpenSettings}>
              <Settings size={14} />
              {t("openSettings")}
            </button>
          </div>
        )}
      </main>

      <footer className="dock-footer">
        <span className="updated-at" title={lastRefresh ?? undefined}>
          {lastRefresh ? t("lastUpdated", { value: formatRelative(lastRefresh, t) }) : t("never")}
        </span>
        <div className="mode-switch" role="group" aria-label={t("mode")}>
          {modeButtons.map(({ mode, label, icon: Icon }) => (
            <button
              key={mode}
              className={`mode-button${config.uiMode === mode ? " active" : ""}`}
              onClick={() => onSetMode(mode)}
              title={label}
              aria-label={label}
            >
              <Icon size={13} />
            </button>
          ))}
        </div>
        <span className="version-tag">v{version}</span>
      </footer>
    </div>
  );
}
