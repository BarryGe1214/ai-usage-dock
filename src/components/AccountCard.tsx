import {
  Activity,
  AlertTriangle,
  CircleDollarSign,
  Cloud,
  Cpu,
  Gauge,
  KeyRound,
  RefreshCw,
  Sparkles,
} from "lucide-react";

import type { Translator } from "../i18n";
import type { AccountView, Language, UiMode, UsageSnapshot } from "../types";
import {
  formatCompactNumber,
  formatCredits,
  formatMoney,
  formatNumber,
  formatPercent,
  formatPlan,
  usedPercent,
  remainingPercent,
  severity,
  windowLabel,
} from "../lib/format";
import { QuotaBar } from "./QuotaBar";
import { Sparkline } from "./Sparkline";

interface AccountCardProps {
  snapshot: UsageSnapshot;
  account?: AccountView;
  language: Language;
  t: Translator;
  mode: UiMode;
  compact: boolean;
  detailed: boolean;
  onRetry: () => void;
  onToggleEnabled?: (enabled: boolean) => void;
}

export function AccountCard({
  snapshot,
  account,
  language,
  t,
  mode,
  compact,
  detailed,
  onRetry,
  onToggleEnabled,
}: AccountCardProps) {
  const isCodex = snapshot.provider === "codex";
  const isDisabled = snapshot.status === "disabled";
  const isLoading = snapshot.status === "loading";
  const isError = snapshot.status === "error";
  const hasContent =
    snapshot.windows.length > 0 || snapshot.summary != null || snapshot.daily.length > 0;
  const limits = snapshot.windows
    .filter((window) => window.enabled)
    .slice()
    .sort((a, b) => a.windowSec - b.windowSec);

  return (
    <section
      className={`account-card provider-${snapshot.provider}${
        isDisabled ? " is-disabled" : ""
      }${compact ? " is-compact" : ""}`}
    >
      <header className="account-head">
        <div className={`account-icon provider-${snapshot.provider}`}>
          {isCodex ? <Sparkles size={17} /> : <Cloud size={17} />}
        </div>
        <div className="account-title">
          <div className="account-name-row">
            <h3>{snapshot.label}</h3>
            {isCodex && snapshot.codex?.planType ? (
              <span className="badge plan-badge">
                {formatPlan(snapshot.codex.planType, t)}
              </span>
            ) : null}
            {!isCodex && account && !account.hasSecret ? (
              <span className="badge badge-danger">
                <KeyRound size={11} />
                {t("secretMissing")}
              </span>
            ) : null}
          </div>
          <span className="account-subtitle">
            {isCodex ? t("codexHint") : t("funcloud")}
          </span>
        </div>
        <div className="account-status">
          <span className={`status-pill status-${snapshot.status}`}>
            <span className="status-dot" />
            {isDisabled
              ? t("disabled")
              : isError
                ? t("error")
                : isLoading
                  ? t("loadingShort")
                  : t("success")}
          </span>
          {isError ? (
            <button className="icon-button tiny" onClick={onRetry} title={t("retry")}>
              <RefreshCw size={13} />
            </button>
          ) : null}
        </div>
      </header>

      {isError ? (
        <div className="account-error">
          <AlertTriangle size={14} />
          <span>
            {hasContent ? `${t("staleData")} · ` : ""}
            {snapshot.error || t("accountError")}
          </span>
        </div>
      ) : null}

      {isDisabled ? (
        <div className="account-disabled">
          <span>{t("disabled")}</span>
          {onToggleEnabled ? (
            <button className="ghost-button small" onClick={() => onToggleEnabled(true)}>
              {t("enable")}
            </button>
          ) : null}
        </div>
      ) : null}

      {isLoading && !hasContent ? (
        <div className="account-skeleton" aria-label={t("loading")}>
          <div className="skeleton-line w-60" />
          <div className="skeleton-bar" />
          <div className="skeleton-line w-40" />
          <div className="skeleton-bar" />
        </div>
      ) : null}

      {!isDisabled && hasContent ? (
        <>
          <div className="account-quotas">
            {limits.slice(0, compact ? 2 : undefined).map((window) => {
              const remaining = remainingPercent(window);
              const used = usedPercent(window);
              const tone = severity(remaining);
              const label = windowLabel(window.windowSec, t, compact);
              const amount = `${formatMoney(window.used)} / ${formatMoney(window.limit)}`;
              const leading = isCodex
                ? formatPercent(remaining)
                : compact
                  ? amount
                  : formatPercent(used);
              const caption = isCodex
                ? `${t("used")} ${formatPercent(window.percent)}`
                : `${amount} · ${t("remaining")} ${formatPercent(remaining)}`;
              return (
                <QuotaBar
                  key={window.id}
                  label={label}
                  percent={isCodex ? remaining : used}
                  tone={tone}
                  leading={leading}
                  caption={caption}
                  resetsAt={window.resetsAt}
                  language={language}
                  t={t}
                  compact={compact}
                />
              );
            })}
            {limits.length === 0 ? (
              <div className="muted-line">{isCodex ? t("noData") : t("noLimit")}</div>
            ) : null}
          </div>

          {isCodex ? (
            <div className="codex-meta">
              <span>
                {t("plan")}: {formatPlan(snapshot.codex?.planType, t)}
              </span>
              {formatCredits(snapshot.codex?.credits, t) ? (
                <span>
                  {t("credits")}: {formatCredits(snapshot.codex?.credits, t)}
                </span>
              ) : null}
              {snapshot.codex?.reachedType ? (
                <span className="warn-text">
                  <AlertTriangle size={12} />
                  {snapshot.codex.reachedType}
                </span>
              ) : null}
            </div>
          ) : snapshot.summary ? (
            <>
              <div className={`summary-chips${detailed ? " is-detailed" : ""}`}>
                <span className="summary-chip">
                  <CircleDollarSign size={13} />
                  {t("todayCost")} <strong>{formatMoney(snapshot.summary.todayCost)}</strong>
                </span>
                <span className="summary-chip">
                  {t("totalCost")} <strong>{formatMoney(snapshot.summary.totalCost)}</strong>
                </span>
                {detailed ? (
                  <>
                    <span className="summary-chip">
                      <Cpu size={13} />
                      {t("totalTokens")}{" "}
                      <strong>{formatCompactNumber(snapshot.summary.totalTokens)}</strong>
                    </span>
                    <span className="summary-chip">
                      <Activity size={13} />
                      {t("requests")}{" "}
                      <strong>{formatNumber(snapshot.summary.totalReqCnt, language)}</strong>
                    </span>
                    <span className="summary-chip">
                      <Gauge size={13} />
                      {t("rpm")} / {t("tpm")}{" "}
                      <strong>
                        {formatNumber(snapshot.summary.rpm, language)} /{" "}
                        {formatCompactNumber(snapshot.summary.tpm)}
                      </strong>
                    </span>
                    <span className="summary-chip">
                      {t("avgLatency")}{" "}
                      <strong>{Math.round(snapshot.summary.avgLatencyMs)} ms</strong>
                    </span>
                  </>
                ) : null}
              </div>

              {detailed && snapshot.daily.length > 0 ? (
                <div className="chart-block">
                  <div className="chart-head">
                    <span>{t("dailyCost7d")}</span>
                    <strong>
                      {formatMoney(
                        snapshot.daily.reduce((total, item) => total + item.cost, 0),
                      )}
                    </strong>
                  </div>
                  <Sparkline
                    values={snapshot.daily.map((item) => item.cost)}
                    labels={snapshot.daily.map((item) => item.date)}
                    formatValue={(value) => formatMoney(value)}
                  />
                </div>
              ) : null}

              {detailed && snapshot.models.length > 0 ? (
                <div className="model-list">
                  <div className="chart-head">
                    <span>{t("topModels")}</span>
                    <span className="muted">{t("cost")}</span>
                  </div>
                  {snapshot.models.slice(0, 4).map((model) => (
                    <div className="model-row" key={model.model}>
                      <span className="model-name" title={model.model}>
                        {model.model}
                      </span>
                      <span className="model-cost">{formatMoney(model.cost)}</span>
                    </div>
                  ))}
                </div>
              ) : null}
            </>
          ) : null}
        </>
      ) : null}

      {mode === "sidebar" && isError ? (
        <button className="text-button" onClick={onRetry}>
          <RefreshCw size={13} />
          {t("retry")}
        </button>
      ) : null}
    </section>
  );
}
