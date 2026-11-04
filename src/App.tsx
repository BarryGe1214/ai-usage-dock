import { useCallback, useEffect, useMemo, useState } from "react";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import type {
  AccountView,
  AppConfig,
  Bootstrap,
  Language,
  UiMode,
  UsageSnapshot,
} from "./types";
import { makeTranslator } from "./i18n";
import {
  getBootstrap,
  hideMain,
  isTauri,
  onAccountsUpdated,
  onConfigUpdated,
  onLayoutUpdated,
  onUsageUpdated,
  openSettings,
  refreshAccount,
  refreshAll,
  setAccountEnabled,
  setFloatingExpanded,
  setUiMode,
} from "./lib/bridge";
import { Dock } from "./components/Dock";
import { Logo } from "./components/Logo";
import { SettingsPage } from "./components/SettingsPage";

const isSettingsView =
  new URLSearchParams(window.location.search).get("view") === "settings" ||
  (isTauri && getCurrentWindow().label === "settings");

export default function App() {
  const [bootstrap, setBootstrap] = useState<Bootstrap | null>(null);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [accounts, setAccounts] = useState<AccountView[]>([]);
  const [snapshots, setSnapshots] = useState<UsageSnapshot[]>([]);
  const [bootError, setBootError] = useState<string | null>(null);

  const loadBootstrap = useCallback(async () => {
    setBootError(null);
    try {
      const data = await getBootstrap();
      setBootstrap(data);
      setConfig(data.config);
      setAccounts(data.accounts);
      setSnapshots(data.snapshots);
    } catch (error) {
      setBootError(error instanceof Error ? error.message : String(error));
    }
  }, []);

  useEffect(() => {
    void loadBootstrap();
  }, [loadBootstrap]);

  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];
    let cancelled = false;

    Promise.all([
      onUsageUpdated(setSnapshots),
      onConfigUpdated(setConfig),
      onAccountsUpdated(setAccounts),
      onLayoutUpdated((layout) =>
        setConfig((previous) =>
          previous
            ? {
                ...previous,
                uiMode: layout.uiMode,
                floatingExpanded: layout.floatingExpanded,
                sidebarEdge: layout.sidebarEdge,
              }
            : previous,
        ),
      ),
    ]).then((fns) => {
      if (cancelled) {
        fns.forEach((fn) => fn());
      } else {
        unlisteners.push(...fns);
      }
    });

    return () => {
      cancelled = true;
      unlisteners.forEach((fn) => fn());
    };
  }, []);

  const language: Language = config?.language ?? "zh";
  const t = useMemo(() => makeTranslator(language), [language]);

  useEffect(() => {
    const theme = config?.theme ?? "system";
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => {
      const dark = theme === "dark" || (theme === "system" && media.matches);
      document.documentElement.dataset.theme = dark ? "dark" : "light";
    };
    apply();
    if (theme === "system") {
      media.addEventListener("change", apply);
      return () => media.removeEventListener("change", apply);
    }
  }, [config?.theme]);

  useEffect(() => {
    document.documentElement.lang = language === "zh" ? "zh-CN" : "en";
    document.title = isSettingsView
      ? `${t("settingsTitle")} · ${t("appName")}`
      : t("appName");
    document.body.dataset.view = isSettingsView ? "settings" : "dock";
  }, [language, t]);

  async function handleRefresh() {
    const next = await refreshAll();
    setSnapshots(next);
  }

  async function handleRefreshAccount(id: string) {
    const next = await refreshAccount(id);
    setSnapshots(next);
  }

  async function handleToggleAccount(id: string, enabled: boolean) {
    const next = await setAccountEnabled(id, enabled);
    setConfig(next);
  }

  async function handleToggleExpand() {
    if (!config) return;
    const next = await setFloatingExpanded(!config.floatingExpanded);
    setConfig(next);
  }

  async function handleSetMode(mode: UiMode) {
    const next = await setUiMode(mode);
    setConfig(next);
  }

  if (bootError) {
    return (
      <div className="boot-screen">
        <Logo size={46} />
        <h1>{t("appName")}</h1>
        <p className="boot-error">{bootError}</p>
        <button className="primary-button" onClick={() => void loadBootstrap()}>
          {t("retry")}
        </button>
      </div>
    );
  }

  if (!config || !bootstrap) {
    return (
      <div className="boot-screen">
        <Logo size={46} />
        <h1>{t("appName")}</h1>
        <span className="spinner" />
        <p>{t("loading")}</p>
      </div>
    );
  }

  if (isSettingsView) {
    return (
      <SettingsPage
        config={config}
        accounts={accounts}
        snapshots={snapshots}
        t={t}
        version={bootstrap.version}
        codexPath={bootstrap.codexPath}
        onConfigChange={setConfig}
      />
    );
  }

  return (
    <Dock
      config={config}
      accounts={accounts}
      snapshots={snapshots}
      language={language}
      t={t}
      version={bootstrap.version}
      onRefresh={handleRefresh}
      onRefreshAccount={handleRefreshAccount}
      onToggleAccount={handleToggleAccount}
      onOpenSettings={() => void openSettings()}
      onHide={() => void hideMain()}
      onToggleExpand={() => void handleToggleExpand()}
      onSetMode={(mode) => void handleSetMode(mode)}
    />
  );
}
