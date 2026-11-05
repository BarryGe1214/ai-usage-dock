import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import {
  AlertTriangle,
  Check,
  Cloud,
  ExternalLink,
  Eye,
  EyeOff,
  KeyRound,
  Languages,
  LogOut,
  Monitor,
  Pencil,
  Plus,
  RefreshCw,
  ShieldCheck,
  Sparkles,
  Sun,
  Trash2,
  X,
} from "lucide-react";

import type { Translator } from "../i18n";
import type { AccountView, AppConfig, CodexCheck, UpdateState, UsageSnapshot } from "../types";
import {
  addFuncloudAccount,
  checkCodex,
  openUrl,
  quitApp,
  removeAccount,
  setAccountEnabled,
  updateFuncloudAccount,
  updatePreferences,
} from "../lib/bridge";
import { formatPlan } from "../lib/format";
import { Logo } from "./Logo";
import { UpdateNotice } from "./UpdateNotice";

interface SettingsPageProps {
  config: AppConfig;
  accounts: AccountView[];
  snapshots: UsageSnapshot[];
  t: Translator;
  version: string;
  codexPath?: string | null;
  onConfigChange: (config: AppConfig) => void;
  onClose: () => void;
  update: UpdateState;
  onCheckUpdate: () => void;
  onInstallUpdate: () => void;
  onRestartUpdate: () => void;
}

interface ToastState {
  message: string;
  tone: "success" | "error";
}

interface AccountFormState {
  mode: "add" | "edit";
  id?: string;
  label: string;
  apiKey: string;
  showKey: boolean;
}

const REPOSITORY_URL = "https://github.com/BarryGe1214/ai-usage-dock";

export function SettingsPage({
  config,
  accounts,
  snapshots,
  t,
  version,
  codexPath,
  onConfigChange,
  onClose,
  update,
  onCheckUpdate,
  onInstallUpdate,
  onRestartUpdate,
}: SettingsPageProps) {
  const [draft, setDraft] = useState<AppConfig>(config);
  const [codexState, setCodexState] = useState<CodexCheck | null>(null);
  const [checkingCodex, setCheckingCodex] = useState(false);
  const [form, setForm] = useState<AccountFormState | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<AccountView | null>(null);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [baseUrl, setBaseUrl] = useState(config.funcloudBaseUrl);

  useEffect(() => setDraft(config), [config]);
  useEffect(() => setBaseUrl(config.funcloudBaseUrl), [config.funcloudBaseUrl]);
  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(null), 2800);
    return () => window.clearTimeout(timer);
  }, [toast]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !form && !deleteTarget) {
        onClose();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [deleteTarget, form, onClose]);

  const codexSnapshot = snapshots.find((snapshot) => snapshot.provider === "codex");
  const funcloudAccounts = accounts.filter(
    (account) => account.provider === "funcloud",
  );
  const codexAccount = accounts.find((account) => account.provider === "codex");

  function showToast(message: string, tone: ToastState["tone"] = "success") {
    setToast({ message, tone });
  }

  function errorMessage(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
  }

  async function applyPatch(patch: Partial<AppConfig>) {
    const next = { ...draft, ...patch };
    setDraft(next);
    onConfigChange(next);
    try {
      const saved = await updatePreferences(next);
      setDraft(saved);
      onConfigChange(saved);
    } catch (error) {
      setDraft(config);
      onConfigChange(config);
      showToast(errorMessage(error), "error");
    }
  }

  async function handleCheckCodex() {
    if (checkingCodex) return;
    setCheckingCodex(true);
    try {
      const result = await checkCodex();
      setCodexState(result);
      if (result.loggedIn) {
        showToast(
          `${t("codexDetected")}${result.planType ? ` · ${formatPlan(result.planType, t)}` : ""}`,
        );
      } else {
        showToast(result.error || t("codexNotLoggedIn"), "error");
      }
    } catch (error) {
      showToast(errorMessage(error), "error");
    } finally {
      setCheckingCodex(false);
    }
  }

  async function handleToggleAccount(account: AccountView, enabled: boolean) {
    try {
      await setAccountEnabled(account.id, enabled);
    } catch (error) {
      showToast(errorMessage(error), "error");
    }
  }

  async function handleSubmitForm() {
    if (!form) return;
    if (!form.label.trim()) {
      showToast(t("addFailed"), "error");
      return;
    }
    if (form.mode === "add" && !form.apiKey.trim()) {
      showToast(t("addFailed"), "error");
      return;
    }
    try {
      if (form.mode === "add") {
        await addFuncloudAccount(form.label.trim(), form.apiKey.trim());
        showToast(t("accountAdded"));
      } else if (form.id) {
        await updateFuncloudAccount(
          form.id,
          form.label.trim(),
          form.apiKey.trim() ? form.apiKey.trim() : null,
        );
        showToast(t("accountUpdated"));
      }
      setForm(null);
    } catch (error) {
      showToast(errorMessage(error), "error");
    }
  }

  async function handleDelete() {
    if (!deleteTarget) return;
    try {
      await removeAccount(deleteTarget.id);
      showToast(t("accountRemoved"));
      setDeleteTarget(null);
    } catch (error) {
      showToast(errorMessage(error), "error");
    }
  }

  async function handleSaveBaseUrl() {
    if (baseUrl.trim() === config.funcloudBaseUrl) return;
    await applyPatch({ funcloudBaseUrl: baseUrl.trim() });
    showToast(t("saved"));
  }

  const codexConnected = codexState
    ? codexState.loggedIn
    : codexSnapshot?.status === "ok";
  const codexPlan = codexState?.planType ?? codexSnapshot?.codex?.planType;
  const codexDisplayPath = codexState?.path ?? codexPath;
  const codexStatus = codexAccount && !codexAccount.enabled
    ? "disabled"
    : codexConnected
      ? "ok"
      : codexState
        ? "error"
        : (codexSnapshot?.status ?? "loading");
  const codexStatusLabel =
    codexAccount && !codexAccount.enabled
      ? t("disabled")
      : codexConnected
        ? `${t("codexDetected")}${codexPlan ? ` · ${formatPlan(codexPlan, t)}` : ""}`
        : codexState?.installed
          ? t("codexNotLoggedIn")
          : codexState
            ? t("codexNotInstalled")
            : t("notAvailable");

  return (
    <div className="settings-window">
      <header className="settings-header">
        <div className="settings-heading" data-tauri-drag-region>
          <Logo size={38} />
          <div data-tauri-drag-region>
            <h1 data-tauri-drag-region>{t("settingsTitle")}</h1>
            <p data-tauri-drag-region>{t("settingsSubtitle")}</p>
          </div>
        </div>
        <button
          className="icon-button"
          onClick={onClose}
          title={t("close")}
          aria-label={t("close")}
        >
          <X size={17} />
        </button>
      </header>

      <div className="settings-scroll">
        <section className="settings-section">
          <h2>{t("general")}</h2>
          <div className="settings-card">
            <SettingRow
              icon={<Languages size={16} />}
              label={t("language")}
              description="中文 / English"
            >
              <Segmented
                value={draft.language}
                options={[
                  { value: "zh", label: "中文" },
                  { value: "en", label: "English" },
                ]}
                onChange={(value) => void applyPatch({ language: value })}
              />
            </SettingRow>

            <SettingRow icon={<Sun size={16} />} label={t("theme")}>
              <Segmented
                value={draft.theme}
                options={[
                  { value: "system", label: t("themeSystem") },
                  { value: "dark", label: t("themeDark") },
                  { value: "light", label: t("themeLight") },
                ]}
                onChange={(value) => void applyPatch({ theme: value })}
              />
            </SettingRow>

            <SettingRow icon={<Monitor size={16} />} label={t("mode")}>
              <Segmented
                value={draft.uiMode}
                options={[
                  { value: "floating", label: t("modeFloating") },
                  { value: "sidebar", label: t("modeSidebar") },
                  { value: "tray", label: t("modeTray") },
                ]}
                onChange={(value) => void applyPatch({ uiMode: value })}
              />
            </SettingRow>

            {draft.uiMode === "sidebar" ? (
              <SettingRow icon={<Monitor size={16} />} label={t("sidebarEdge")}>
                <Segmented
                  value={draft.sidebarEdge}
                  options={[
                    { value: "left", label: t("edgeLeft") },
                    { value: "right", label: t("edgeRight") },
                  ]}
                  onChange={(value) => void applyPatch({ sidebarEdge: value })}
                />
              </SettingRow>
            ) : null}

            <SettingRow
              icon={<RefreshCw size={16} />}
              label={t("refreshInterval")}
              description={`${draft.refreshIntervalSecs}s`}
            >
              <select
                className="select"
                value={draft.refreshIntervalSecs}
                onChange={(event) =>
                  void applyPatch({ refreshIntervalSecs: Number(event.target.value) })
                }
              >
                <option value={30}>{t("every30s")}</option>
                <option value={60}>{t("every1m")}</option>
                <option value={120}>{t("every2m")}</option>
                <option value={300}>{t("every5m")}</option>
                <option value={600}>{t("every10m")}</option>
                <option value={1800}>{t("every30m")}</option>
              </select>
            </SettingRow>

            <SettingRow
              icon={<Monitor size={16} />}
              label={t("alwaysOnTop")}
              description={t("modeFloating")}
            >
              <Toggle
                checked={draft.alwaysOnTop}
                onChange={(checked) => void applyPatch({ alwaysOnTop: checked })}
              />
            </SettingRow>

            <SettingRow
              icon={<LogOut size={16} />}
              label={t("startAtLogin")}
              description={t("startAtLoginHint")}
            >
              <Toggle
                checked={draft.startAtLogin}
                onChange={(checked) => void applyPatch({ startAtLogin: checked })}
              />
            </SettingRow>
          </div>
        </section>

        <section className="settings-section">
          <h2>{t("accounts")}</h2>
          <p className="section-hint">{t("accountsHint")}</p>

          <div className="account-settings-card">
            <div className="account-settings-head">
              <div className="account-settings-title">
                <span className="account-icon provider-codex">
                  <Sparkles size={16} />
                </span>
                <div>
                  <h3>{t("codexAccount")}</h3>
                  <span className="muted">{t("codexOnlyCurrent")}</span>
                </div>
              </div>
              <div className="account-settings-actions">
                <span className={`status-pill status-${codexStatus}`}>
                  <span className="status-dot" />
                  {codexStatusLabel}
                </span>
                <Toggle
                  checked={codexAccount?.enabled ?? true}
                  onChange={(checked) =>
                    codexAccount ? void handleToggleAccount(codexAccount, checked) : undefined
                  }
                />
                <button
                  className="ghost-button small"
                  onClick={() => void handleCheckCodex()}
                  disabled={checkingCodex}
                >
                  <RefreshCw size={13} className={checkingCodex ? "spin" : undefined} />
                  {checkingCodex ? t("checking") : t("codexCheck")}
                </button>
              </div>
            </div>
            {codexDisplayPath ? (
              <div className="codex-path">
                <span>{t("codexPath")}</span>
                <code>{codexDisplayPath}</code>
              </div>
            ) : null}
            {codexState?.error && !codexState.loggedIn ? (
              <div className="inline-alert">
                <AlertTriangle size={14} />
                <span>{codexState.error}</span>
              </div>
            ) : null}
          </div>

          <div className="account-settings-card">
            <div className="account-settings-head">
              <div className="account-settings-title">
                <span className="account-icon provider-funcloud">
                  <Cloud size={16} />
                </span>
                <div>
                  <h3>{t("funcloudAccounts")}</h3>
                  <span className="muted">
                    {t("accountsCount", { value: funcloudAccounts.length })}
                  </span>
                </div>
              </div>
              <button
                className="primary-button small"
                onClick={() =>
                  setForm({ mode: "add", label: "", apiKey: "", showKey: false })
                }
              >
                <Plus size={13} />
                {t("addAccount")}
              </button>
            </div>

            <div className="account-rows">
              {funcloudAccounts.length === 0 ? (
                <div className="account-empty">{t("noAccountsHint")}</div>
              ) : (
                funcloudAccounts.map((account) => (
                  <div className="account-row" key={account.id}>
                    <div className="account-row-main">
                      <span className={`status-dot ${account.enabled ? "ok" : "off"}`} />
                      <div className="account-row-text">
                        <strong>{account.label}</strong>
                        <span className="muted">
                          <KeyRound size={11} />
                          {account.hasSecret ? t("secretStored") : t("secretMissing")}
                        </span>
                      </div>
                    </div>
                    <div className="account-row-actions">
                      <Toggle
                        checked={account.enabled}
                        onChange={(checked) => void handleToggleAccount(account, checked)}
                      />
                      <button
                        className="icon-button tiny"
                        title={t("edit")}
                        onClick={() =>
                          setForm({
                            mode: "edit",
                            id: account.id,
                            label: account.label,
                            apiKey: "",
                            showKey: false,
                          })
                        }
                      >
                        <Pencil size={13} />
                      </button>
                      <button
                        className="icon-button tiny danger"
                        title={t("delete")}
                        onClick={() => setDeleteTarget(account)}
                      >
                        <Trash2 size={13} />
                      </button>
                    </div>
                  </div>
                ))
              )}
            </div>

            {form ? (
              <div className="account-form">
                <div className="form-head">
                  <h3>{form.mode === "add" ? t("addAccount") : t("editAccount")}</h3>
                  <button
                    className="icon-button tiny"
                    onClick={() => setForm(null)}
                    title={t("cancel")}
                  >
                    <X size={13} />
                  </button>
                </div>
                <label className="field">
                  <span>{t("accountName")}</span>
                  <input
                    className="input"
                    value={form.label}
                    placeholder={t("accountNamePlaceholder")}
                    autoFocus
                    onChange={(event) =>
                      setForm({ ...form, label: event.target.value })
                    }
                  />
                </label>
                <label className="field">
                  <span>{t("apiKey")}</span>
                  <div className="input-with-action">
                    <input
                      className="input"
                      type={form.showKey ? "text" : "password"}
                      value={form.apiKey}
                      placeholder={
                        form.mode === "edit"
                          ? t("apiKeyKeepHint")
                          : t("apiKeyPlaceholder")
                      }
                      onChange={(event) =>
                        setForm({ ...form, apiKey: event.target.value })
                      }
                    />
                    <button
                      className="icon-button tiny"
                      onClick={() => setForm({ ...form, showKey: !form.showKey })}
                      title={form.showKey ? t("hide") : t("apiKey")}
                      type="button"
                    >
                      {form.showKey ? <EyeOff size={14} /> : <Eye size={14} />}
                    </button>
                  </div>
                </label>
                <p className="form-hint">
                  <ShieldCheck size={13} />
                  {t("apiKeyStoredHint")}
                </p>
                <div className="form-actions">
                  <button className="ghost-button" onClick={() => setForm(null)}>
                    {t("cancel")}
                  </button>
                  <button className="primary-button" onClick={() => void handleSubmitForm()}>
                    <Check size={14} />
                    {t("save")}
                  </button>
                </div>
              </div>
            ) : null}
          </div>
        </section>

        <section className="settings-section">
          <h2>{t("advanced")}</h2>
          <div className="settings-card">
            <SettingRow
              icon={<Cloud size={16} />}
              label={t("apiBaseUrl")}
              description={t("apiBaseUrlHint")}
            >
              <div className="input-with-action">
                <input
                  className="input"
                  value={baseUrl}
                  onChange={(event) => setBaseUrl(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter") void handleSaveBaseUrl();
                  }}
                />
                <button className="ghost-button small" onClick={() => void handleSaveBaseUrl()}>
                  {t("save")}
                </button>
              </div>
            </SettingRow>
          </div>
        </section>

        <section className="settings-section">
          <h2>{t("about")}</h2>
          <div className="settings-card about-card">
            <div className="about-row">
              <span>{t("version")}</span>
              <strong>v{version}</strong>
            </div>
            <div className="about-row">
              <span>{t("repository")}</span>
              <button
                className="link-button"
                onClick={() => void openUrl(REPOSITORY_URL)}
              >
                BarryGe1214/ai-usage-dock
                <ExternalLink size={12} />
              </button>
            </div>
            <div className="about-row">
              <span>{t("license")}</span>
              <strong>MIT</strong>
            </div>
            <UpdateNotice
              variant="row"
              update={update}
              t={t}
              onCheck={onCheckUpdate}
              onInstall={onInstallUpdate}
              onRestart={onRestartUpdate}
            />
            <p className="privacy-note">
              <ShieldCheck size={14} />
              {t("privacyNote")}
            </p>
          </div>
        </section>
      </div>

      <footer className="settings-footer">
        <span className="muted">{t("savedLocally")}</span>
        <button className="danger-button" onClick={() => void quitApp()}>
          <LogOut size={14} />
          {t("quit")}
        </button>
      </footer>

      {toast ? (
        <div className={`toast tone-${toast.tone}`}>
          {toast.tone === "success" ? <Check size={14} /> : <AlertTriangle size={14} />}
          {toast.message}
        </div>
      ) : null}

      {deleteTarget ? (
        <div className="modal-overlay" onMouseDown={() => setDeleteTarget(null)}>
          <div className="modal" onMouseDown={(event) => event.stopPropagation()}>
            <div className="modal-icon">
              <Trash2 size={20} />
            </div>
            <h3>{t("deleteConfirm")}</h3>
            <p>{t("deleteConfirmBody", { name: deleteTarget.label })}</p>
            <div className="modal-actions">
              <button className="ghost-button" onClick={() => setDeleteTarget(null)}>
                {t("cancel")}
              </button>
              <button className="danger-button" onClick={() => void handleDelete()}>
                <Trash2 size={14} />
                {t("delete")}
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </div>
  );
}

function SettingRow({
  icon,
  label,
  description,
  children,
}: {
  icon: ReactNode;
  label: string;
  description?: string;
  children: ReactNode;
}) {
  return (
    <div className="setting-row">
      <div className="setting-label">
        <span className="setting-icon">{icon}</span>
        <div>
          <strong>{label}</strong>
          {description ? <span className="muted">{description}</span> : null}
        </div>
      </div>
      <div className="setting-control">{children}</div>
    </div>
  );
}

function Segmented<T extends string>({
  value,
  options,
  onChange,
}: {
  value: T;
  options: Array<{ value: T; label: string }>;
  onChange: (value: T) => void;
}) {
  return (
    <div className="segmented">
      {options.map((option) => (
        <button
          key={option.value}
          className={value === option.value ? "active" : undefined}
          onClick={() => onChange(option.value)}
          type="button"
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

function Toggle({
  checked,
  onChange,
}: {
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      className={`toggle${checked ? " is-on" : ""}`}
      onClick={() => onChange(!checked)}
    >
      <span />
    </button>
  );
}
