import type { ReactNode } from "react";
import {
  AlertTriangle,
  CheckCircle2,
  Download,
  RefreshCw,
  RotateCw,
} from "lucide-react";

import type { Translator } from "../i18n";
import type { UpdateState } from "../types";

interface UpdateNoticeProps {
  variant: "banner" | "row";
  update: UpdateState;
  t: Translator;
  onCheck: () => void;
  onInstall: () => void;
  onRestart: () => void;
}

export function UpdateNotice({
  variant,
  update,
  t,
  onCheck,
  onInstall,
  onRestart,
}: UpdateNoticeProps) {
  const { info, phase, progress, error } = update;

  if (
    variant === "banner" &&
    phase !== "available" &&
    phase !== "downloading" &&
    phase !== "ready"
  ) {
    return null;
  }

  const percent = t("updateDownloading", { value: progress });

  let status = t("updateIdle");
  let icon: ReactNode = <RefreshCw size={14} />;
  let tone = "info";

  switch (phase) {
    case "checking":
      status = t("checkingUpdate");
      icon = <RefreshCw size={14} className="spin" />;
      break;
    case "available":
      status = t("updateAvailable", { version: info?.version ?? "" });
      icon = <Download size={14} />;
      break;
    case "downloading":
      status = percent;
      icon = <Download size={14} />;
      break;
    case "ready":
      status = t("updateReady");
      icon = <RotateCw size={14} />;
      tone = "ready";
      break;
    case "uptodate":
      status = t("upToDate");
      icon = <CheckCircle2 size={14} />;
      break;
    case "error":
      status = error ? `${t("updateError")}: ${error}` : t("updateError");
      icon = <AlertTriangle size={14} />;
      tone = "error";
      break;
    default:
      status = t("updateIdle");
  }

  let action: ReactNode;
  switch (phase) {
    case "available":
      action = (
        <button className="primary-button small" onClick={onInstall}>
          <Download size={12} />
          {t("updateNow")}
        </button>
      );
      break;
    case "downloading":
      action =
        variant === "row" ? (
          <span className="update-progress">{percent}</span>
        ) : null;
      break;
    case "ready":
      action = (
        <button className="primary-button small" onClick={onRestart}>
          <RotateCw size={12} />
          {t("updateRestart")}
        </button>
      );
      break;
    case "checking":
      action = (
        <button className="ghost-button small" disabled>
          <RefreshCw size={12} className="spin" />
          {t("checkingUpdate")}
        </button>
      );
      break;
    default:
      action = (
        <button className="ghost-button small" onClick={onCheck}>
          <RefreshCw size={12} />
          {t("checkUpdate")}
        </button>
      );
  }

  return (
    <div
      className={`update-notice is-${variant} tone-${tone}`}
      aria-live="polite"
    >
      <span className="update-notice-icon">{icon}</span>
      <div className="update-notice-text">
        {variant === "row" ? <strong>{t("updates")}</strong> : null}
        <span>{status}</span>
      </div>
      {action}
    </div>
  );
}
