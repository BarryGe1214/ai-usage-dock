use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdout, Command};

use crate::models::{CodexCheck, CodexInfo, QuotaWindow};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone)]
pub struct CodexData {
    pub windows: Vec<QuotaWindow>,
    pub info: CodexInfo,
}

pub fn locate_codex() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var("CODEX_CLI_PATH") {
        let path = PathBuf::from(explicit);
        if path.exists() {
            return Some(path);
        }
    }

    #[cfg(windows)]
    {
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let bin_root = PathBuf::from(local_app_data)
                .join("OpenAI")
                .join("Codex")
                .join("bin");
            if let Ok(entries) = std::fs::read_dir(&bin_root) {
                let mut candidates = entries
                    .flatten()
                    .map(|entry| entry.path().join("codex.exe"))
                    .filter(|path| path.is_file())
                    .collect::<Vec<_>>();
                candidates.sort_by_key(|path| {
                    std::fs::metadata(path)
                        .and_then(|metadata| metadata.modified())
                        .ok()
                });
                if let Some(path) = candidates.pop() {
                    return Some(path);
                }
            }
            let direct = bin_root.join("codex.exe");
            if direct.is_file() {
                return Some(direct);
            }
        }
    }

    which::which("codex").ok()
}

pub async fn fetch() -> Result<CodexData, String> {
    let path = locate_codex().ok_or_else(|| {
        "未找到 Codex CLI。请先安装 Codex 并完成 `codex login`，或在设置中检查安装路径。"
            .to_string()
    })?;

    match tokio::time::timeout(REQUEST_TIMEOUT, run_session(&path)).await {
        Ok(result) => result,
        Err(_) => Err("读取 Codex 额度超时，请确认 Codex CLI 可以正常运行。".to_string()),
    }
}

pub async fn check() -> CodexCheck {
    let Some(path) = locate_codex() else {
        return CodexCheck {
            installed: false,
            path: None,
            logged_in: false,
            plan_type: None,
            error: Some("未找到 Codex CLI".to_string()),
        };
    };

    match fetch().await {
        Ok(data) => CodexCheck {
            installed: true,
            path: Some(path.display().to_string()),
            logged_in: true,
            plan_type: data.info.plan_type,
            error: None,
        },
        Err(error) => CodexCheck {
            installed: true,
            path: Some(path.display().to_string()),
            logged_in: false,
            plan_type: None,
            error: Some(error),
        },
    }
}

async fn run_session(path: &Path) -> Result<CodexData, String> {
    let mut command = command_for(path);
    command
        .arg("app-server")
        .arg("--listen")
        .arg("stdio://")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let mut child = command
        .spawn()
        .map_err(|error| format!("无法启动 Codex CLI: {error}"))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "无法连接 Codex stdin".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "无法连接 Codex stdout".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "无法连接 Codex stderr".to_string())?;

    let stderr_task = tokio::spawn(async move {
        let mut buffer = String::new();
        let mut reader = BufReader::new(stderr);
        let _ = reader.read_to_string(&mut buffer).await;
        buffer
    });

    let result = async {
        let mut lines = BufReader::new(stdout).lines();

        let initialize = json!({
            "id": 1,
            "method": "initialize",
            "params": {
                "clientInfo": {
                    "name": "ai-usage-dock",
                    "title": "AI Usage Dock",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": null
            }
        });
        write_message(&mut stdin, &initialize).await?;
        read_response(&mut lines, 1).await?;

        let request = json!({
            "id": 2,
            "method": "account/rateLimits/read"
        });
        write_message(&mut stdin, &request).await?;
        let response = read_response(&mut lines, 2).await?;
        parse_snapshot(&response)
    }
    .await;

    let _ = child.kill().await;
    let stderr_text = stderr_task.await.unwrap_or_default();

    match result {
        Ok(data) => Ok(data),
        Err(error) => {
            let stderr_text = stderr_text.trim();
            if stderr_text.is_empty() || error.contains(stderr_text) {
                Err(error)
            } else {
                Err(format!("{error}\n{stderr_text}"))
            }
        }
    }
}

async fn write_message(
    stdin: &mut tokio::process::ChildStdin,
    message: &Value,
) -> Result<(), String> {
    let payload = format!("{message}\n");
    stdin
        .write_all(payload.as_bytes())
        .await
        .map_err(|error| format!("写入 Codex 请求失败: {error}"))?;
    stdin
        .flush()
        .await
        .map_err(|error| format!("刷新 Codex 请求失败: {error}"))
}

async fn read_response(
    lines: &mut tokio::io::Lines<BufReader<ChildStdout>>,
    id: i64,
) -> Result<Value, String> {
    loop {
        let line = lines
            .next_line()
            .await
            .map_err(|error| format!("读取 Codex 响应失败: {error}"))?
            .ok_or_else(|| "Codex app-server 提前退出".to_string())?;

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let Ok(message) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if message.get("id").and_then(Value::as_i64) != Some(id) {
            continue;
        }

        if let Some(error) = message.get("error") {
            let detail = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Codex 返回未知错误");
            return Err(detail.to_string());
        }

        return Ok(message.get("result").cloned().unwrap_or(Value::Null));
    }
}

fn parse_snapshot(result: &Value) -> Result<CodexData, String> {
    let snapshot = result
        .get("rateLimitsByLimitId")
        .and_then(|value| value.get("codex"))
        .or_else(|| {
            result
                .get("rateLimits")
                .filter(|value| value.get("primary").is_some() || value.get("secondary").is_some())
        })
        .ok_or_else(|| "Codex 没有返回可用的额度窗口".to_string())?;

    let mut windows = [
        parse_window(snapshot.get("primary"), "codex-primary"),
        parse_window(snapshot.get("secondary"), "codex-secondary"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    if windows.is_empty() {
        return Err("Codex 返回的额度窗口为空".to_string());
    }
    windows.sort_by_key(|window| window.window_sec);

    let info = CodexInfo {
        plan_type: snapshot
            .get("planType")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        limit_name: snapshot
            .get("limitName")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        reached_type: snapshot
            .get("rateLimitReachedType")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        credits: snapshot.get("credits").cloned(),
    };

    Ok(CodexData { windows, info })
}

fn parse_window(value: Option<&Value>, id: &str) -> Option<QuotaWindow> {
    let value = value?;
    let used = value_f64(value.get("usedPercent")?)?;
    let used = used.clamp(0.0, 100.0);
    let duration_mins = value
        .get("windowDurationMins")
        .and_then(value_f64)
        .unwrap_or(0.0);

    let resets_at = match value.get("resetsAt") {
        Some(Value::Number(number)) => number
            .as_i64()
            .and_then(|seconds| DateTime::<Utc>::from_timestamp(seconds, 0))
            .map(|value| value.to_rfc3339()),
        Some(Value::String(text)) => DateTime::parse_from_rfc3339(text)
            .ok()
            .map(|value| value.with_timezone(&Utc).to_rfc3339()),
        _ => None,
    };

    Some(QuotaWindow {
        id: id.to_string(),
        window_sec: (duration_mins * 60.0).round() as i64,
        enabled: true,
        limit: 100.0,
        used,
        remaining: 100.0 - used,
        percent: used,
        resets_at,
    })
}

fn value_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn command_for(path: &Path) -> Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let mut command = if extension == "cmd" || extension == "bat" {
            let mut command = std::process::Command::new("cmd");
            command.arg("/C").arg(path);
            command
        } else {
            std::process::Command::new(path)
        };
        command.creation_flags(0x08000000);
        return Command::from(command);
    }

    #[cfg(not(windows))]
    {
        Command::new(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_codex_rate_limit_snapshot() {
        let result = json!({
            "rateLimitsByLimitId": {
                "codex": {
                    "limitId": "codex",
                    "limitName": "Codex",
                    "planType": "plus",
                    "primary": {
                        "usedPercent": 23,
                        "windowDurationMins": 300,
                        "resetsAt": 1_760_000_000_i64
                    },
                    "secondary": {
                        "usedPercent": 61,
                        "windowDurationMins": 10080,
                        "resetsAt": 1_760_600_000_i64
                    }
                }
            }
        });

        let data = parse_snapshot(&result).expect("snapshot should parse");
        assert_eq!(data.windows.len(), 2);
        assert_eq!(data.windows[0].window_sec, 18_000);
        assert!((data.windows[0].remaining - 77.0).abs() < f64::EPSILON);
        assert_eq!(data.windows[1].window_sec, 604_800);
        assert_eq!(data.info.plan_type.as_deref(), Some("plus"));
        assert!(data.windows[0].resets_at.is_some());
    }
}
