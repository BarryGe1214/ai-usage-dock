import { useCallback, useEffect, useMemo, useState } from "react";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import type {
  AccountView,
  AppConfig,
  Bootstrap,
  Language,
  UiMode,
  UpdateState,
  UsageSnapshot,
} from "./types";
import { makeTranslator } from "./i18n";
import {
  checkUpdate,
  getBootstrap,
  hideMain,
  installUpdate,
  isTauri,
  onAccountsUpdated,
  onConfigUpdated,
  onLayoutUpdated,
  onUpdateProgress,
  onUsageUpdated,
  onViewChanged,
  openSettings,
  quitApp,
  refreshAccount,
  refreshAll,
  restartApp,
  setAccountEnabled,
  setFloatingExpanded,
  setUiMode,
} from "./lib/bridge";
import { Dock } from "./components/Dock";
import { Logo } from "./components/Logo";
import { SettingsPage } from "./components/SettingsPage";

type AppView = "dock" | "settings";

const initialView: AppView =
  new URLSearchParams(window.location.search).get("view") === "settings" ||
  (isTauri && getCurrentWindow().label === "settings")
    ? "settings"
    : "dock";

export default function App() {
  const [view, setView] = useState<AppView>(initialView);
  const [bootstrap, setBootstrap] = useState<Bootstrap | null>(null);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [accounts, setAccounts] = useState<AccountView[]>([]);
  const [snapshots, setSnapshots] = useState<UsageSnapshot[]>([]);
  const [bootError, setBootError] = useState<string | null>(null);
  const [update, setUpdate] = useState<UpdateState>({
    info: null,
    phase: "idle",
    progress: 0,
  });

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

  const runUpdateCheck = useCallback(async (manual: boolean) => {
    if (!isTauri) {
      if (manual) setUpdate({ info: null, phase: "uptodate", progress: 0 });
      return;
    }
    setUpdate((previous) => ({ ...previous, phase: "checking", error: null }));
    try {
      const info = await checkUpdate();
      if (info) {
        setUpdate({ info, phase: "available", progress: 0 });
      } else {
        setUpdate({ info: null, phase: "uptodate", progress: 0 });
      }
    } catch (error) {
      setUpdate({
        info: null,
        phase: manual ? "error" : "idle",
        progress: 0,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }, []);

  useEffect(() => {
    void runUpdateCheck(false);
  }, [runUpdateCheck]);

  const installAvailableUpdate = useCallback(async () => {
    if (!update.info || !isTauri) return;
    setUpdate((previous) => ({
      ...previous,
      phase: "downloading",
      progress: 0,
      error: null,
    }));
    try {
      await installUpdate();
      setUpdate((previous) => ({ ...previous, phase: "ready", progress: 100 }));
    } catch (error) {
      setUpdate({
        info: update.info,
        phase: "error",
        progress: 0,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }, [update.info]);

  const restartForUpdate = useCallback(() => {
    void restartApp();
  }, []);

  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];
    let cancelled = false;

    Promise.all([
      onUsageUpdated(setSnapshots),
      onConfigUpdated(setConfig),
      onAccountsUpdated(setAccounts),
      onViewChanged(setView),
      onUpdateProgress((progress) =>
        setUpdate((previous) => {
          if (previous.phase !== "available" && previous.phase !== "downloading") {
            return previous;
          }
          const total = progress.total ?? 0;
          const percent =
            total > 0
              ? Math.min(100, Math.round((progress.downloaded / total) * 100))
              : previous.progress;
          return { ...previous, phase: "downloading", progress: percent };
        }),
      ),
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
    document.title =
      view === "settings" ? `${t("settingsTitle")} · ${t("appName")}` : t("appName");
    document.body.dataset.view = view;
  }, [language, t, view]);

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

  if (view === "settings") {
    return (
      <SettingsPage
        config={config}
        accounts={accounts}
        snapshots={snapshots}
        t={t}
        version={bootstrap.version}
        codexPath={bootstrap.codexPath}
        onConfigChange={setConfig}
        update={update}
        onCheckUpdate={() => void runUpdateCheck(true)}
        onInstallUpdate={() => void installAvailableUpdate()}
        onRestartUpdate={restartForUpdate}
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
      onQuit={() => void quitApp()}
      onToggleExpand={() => void handleToggleExpand()}
      onSetMode={(mode) => void handleSetMode(mode)}
      update={update}
      onCheckUpdate={() => void runUpdateCheck(true)}
      onInstallUpdate={() => void installAvailableUpdate()}
      onRestartUpdate={restartForUpdate}
    />
  );
}
