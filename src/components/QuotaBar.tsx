import type { ReactNode } from "react";

import type { Language } from "../types";
import type { Translator } from "../i18n";
import { formatResetTime } from "../lib/format";

interface QuotaBarProps {
  label: string;
  percent: number;
  tone: "good" | "warn" | "bad";
  leading?: ReactNode;
  caption?: string;
  resetsAt?: string | null;
  language: Language;
  t: Translator;
  compact?: boolean;
}

export function QuotaBar({
  label,
  percent,
  tone,
  leading,
  caption,
  resetsAt,
  language,
  t,
  compact = false,
}: QuotaBarProps) {
  const width = Math.max(0, Math.min(100, percent));
  return (
    <div className={`quota-row${compact ? " is-compact" : ""}`}>
      <div className="quota-head">
        <span className="quota-label">{label}</span>
        {leading !== undefined ? <span className="quota-leading">{leading}</span> : null}
      </div>
      <div className="quota-track" role="progressbar" aria-valuenow={Math.round(width)} aria-valuemin={0} aria-valuemax={100}>
        <div className={`quota-fill tone-${tone}`} style={{ width: `${width}%` }} />
      </div>
      {!compact ? (
        <div className="quota-meta">
          <span>{caption}</span>
          <span>{resetsAt ? formatResetTime(resetsAt, language, t) : ""}</span>
        </div>
      ) : null}
    </div>
  );
}
