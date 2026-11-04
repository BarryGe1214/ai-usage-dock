import type { Language, QuotaWindow } from "../types";
import type { Translator } from "../i18n";

export function formatMoney(value: number | null | undefined): string {
  const amount = Number.isFinite(value ?? NaN) ? Number(value) : 0;
  if (Math.abs(amount) >= 1000) {
    return `$${amount.toLocaleString("en-US", {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    })}`;
  }
  return `$${amount.toFixed(2)}`;
}

export function formatNumber(value: number | null | undefined, language: Language): string {
  const amount = Number.isFinite(value ?? NaN) ? Number(value) : 0;
  return amount.toLocaleString(language === "zh" ? "zh-CN" : "en-US", {
    maximumFractionDigits: 1,
  });
}

export function formatCompactNumber(value: number | null | undefined): string {
  const amount = Number.isFinite(value ?? NaN) ? Number(value) : 0;
  if (Math.abs(amount) >= 1_000_000) {
    return `${(amount / 1_000_000).toFixed(2)}M`;
  }
  if (Math.abs(amount) >= 1_000) {
    return `${(amount / 1_000).toFixed(1)}K`;
  }
  return amount.toFixed(0);
}

export function formatPercent(value: number | null | undefined): string {
  const amount = Number.isFinite(value ?? NaN) ? Number(value) : 0;
  return `${Math.round(amount)}%`;
}

export function remainingPercent(window: QuotaWindow): number {
  if (window.limit > 0) {
    return Math.max(0, Math.min(100, (window.remaining / window.limit) * 100));
  }
  return Math.max(0, Math.min(100, 100 - window.percent));
}

export function usedPercent(window: QuotaWindow): number {
  if (window.limit > 0) {
    return Math.max(0, Math.min(100, (window.used / window.limit) * 100));
  }
  return Math.max(0, Math.min(100, window.percent));
}

export function severity(remaining: number): "good" | "warn" | "bad" {
  if (remaining <= 15) return "bad";
  if (remaining <= 40) return "warn";
  return "good";
}

export function windowLabel(
  seconds: number,
  t: Translator,
  short = false,
): string {
  if (!Number.isFinite(seconds) || seconds <= 0) return t("quotaWindow");
  if (seconds === 18_000) return short ? t("window5hShort") : t("window5h");
  if (seconds === 86_400) return short ? "24h" : t("windowDaily");
  if (seconds === 604_800) return short ? t("windowWeeklyShort") : t("windowWeekly");
  if (seconds === 2_592_000) return t("windowMonthly");
  if (seconds >= 31_536_000) return t("windowTotal");
  if (seconds >= 86_400) {
    return t("windowDays", { value: Math.round(seconds / 86_400) });
  }
  if (seconds >= 3_600) {
    return t("windowHours", { value: Math.round(seconds / 3_600) });
  }
  if (seconds >= 60) {
    return t("windowMinutes", { value: Math.round(seconds / 60) });
  }
  return t("windowSeconds", { value: Math.round(seconds) });
}

export function formatResetTime(
  iso: string | null | undefined,
  language: Language,
  t: Translator,
): string {
  if (!iso) return t("resetUnknown");
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return t("resetUnknown");
  const diff = date.getTime() - Date.now();
  if (diff <= 0) return t("resetsSoon");

  const minutes = Math.floor(diff / 60_000);
  if (minutes < 60) {
    return t("resetAt", { value: `${minutes}m` });
  }
  const hours = Math.floor(minutes / 60);
  if (hours < 48) {
    return t("resetAt", { value: `${hours}h` });
  }
  const days = Math.floor(hours / 24);
  return t("resetAt", {
    value:
      language === "zh"
        ? `${days} 天`
        : `${days}d`,
  });
}

export function formatRelative(
  iso: string | null | undefined,
  t: Translator,
): string {
  if (!iso) return t("never");
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return t("never");
  const seconds = Math.max(0, Math.floor((Date.now() - date.getTime()) / 1000));
  if (seconds < 10) return t("justNow");
  if (seconds < 60) return t("secondsAgo", { value: seconds });
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return t("minutesAgo", { value: minutes });
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return t("hoursAgo", { value: hours });
  const days = Math.floor(hours / 24);
  return t("daysAgo", { value: days });
}

export function formatDateLabel(date: string, language: Language): string {
  const parsed = new Date(`${date}T00:00:00`);
  if (Number.isNaN(parsed.getTime())) return date;
  return parsed.toLocaleDateString(language === "zh" ? "zh-CN" : "en-US", {
    month: "numeric",
    day: "numeric",
  });
}

export function formatPlan(plan: string | null | undefined, t: Translator): string {
  if (!plan) return t("unknownPlan");
  return plan.charAt(0).toUpperCase() + plan.slice(1);
}

export function formatCredits(value: unknown, t: Translator): string | null {
  if (value === null || value === undefined) return null;
  if (typeof value === "number") {
    return t("creditsRemaining", { value: formatNumber(value, "en") });
  }
  if (typeof value === "string") {
    return value;
  }
  if (typeof value === "object") {
    const record = value as Record<string, unknown>;
    if (record.unlimited === true) return t("creditsUnlimited");
    const balance = record.balance ?? record.remaining ?? record.credits;
    if (typeof balance === "number") {
      return t("creditsRemaining", { value: formatNumber(balance, "en") });
    }
    if (typeof balance === "string") {
      return t("creditsRemaining", { value: balance });
    }
  }
  return null;
}
