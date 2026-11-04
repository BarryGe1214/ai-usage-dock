use std::time::Duration;

use chrono::{Duration as ChronoDuration, Local, TimeZone, Utc};
use serde_json::{Value, json};

use crate::models::{DailyUsage, FuncloudSummary, ModelUsage, QuotaWindow};

#[derive(Debug, Clone, Default)]
pub struct FuncloudData {
    pub windows: Vec<QuotaWindow>,
    pub summary: FuncloudSummary,
    pub daily: Vec<DailyUsage>,
    pub models: Vec<ModelUsage>,
}

pub async fn fetch(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
) -> Result<FuncloudData, String> {
    let key = api_key.trim();
    if key.is_empty() {
        return Err("未配置 Funcloud API Key".to_string());
    }

    let (time_start, time_end) = seven_day_range();

    let limit_future = post(
        client,
        base_url,
        "/v1/model/key-usage/limit",
        key,
        json!({ "data": {} }),
    );
    let summary_future = post(
        client,
        base_url,
        "/v1/model/key-usage/summary",
        key,
        json!({ "data": {} }),
    );
    let (limit_response, summary_response) = tokio::join!(limit_future, summary_future);

    let limit = limit_response?;
    let summary = summary_response?;

    let range_body = json!({
        "data": {
            "time_start": time_start,
            "time_end": time_end
        }
    });
    let daily_future = post(
        client,
        base_url,
        "/v1/model/key-usage/daily",
        key,
        range_body.clone(),
    );
    let models_future = post(
        client,
        base_url,
        "/v1/model/key-usage/models",
        key,
        range_body,
    );
    let (daily_response, models_response) = tokio::join!(daily_future, models_future);

    let daily = daily_response
        .map(|value| parse_daily(&value))
        .unwrap_or_default();
    let models = models_response
        .map(|value| parse_models(&value))
        .unwrap_or_default();

    Ok(FuncloudData {
        windows: parse_windows(&limit),
        summary: parse_summary(&summary),
        daily,
        models,
    })
}

async fn post(
    client: &reqwest::Client,
    base_url: &str,
    path: &str,
    api_key: &str,
    body: Value,
) -> Result<Value, String> {
    let url = format!("{}{}", base_url.trim_end_matches('/'), path);
    let response = client
        .post(&url)
        .bearer_auth(api_key)
        .header("Accept", "application/json")
        .json(&body)
        .timeout(Duration::from_secs(25))
        .send()
        .await
        .map_err(|error| format!("无法连接 Funcloud: {error}"))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| format!("读取 Funcloud 响应失败: {error}"))?;

    let parsed: Value = serde_json::from_str(&text)
        .map_err(|_| format!("Funcloud 返回了非 JSON 响应 (HTTP {status})"))?;

    let code = value_i64(&parsed, "code").unwrap_or(-1);
    if code != 200 {
        let message = parsed
            .get("code_msg")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .or_else(|| parsed.get("code_reason").and_then(Value::as_str))
            .map(ToString::to_string)
            .unwrap_or_else(|| match status.as_u16() {
                401 | 403 => "API Key 无效或已过期".to_string(),
                429 => "请求过于频繁，请稍后重试".to_string(),
                _ => format!("Funcloud 查询失败 (HTTP {status}, code {code})"),
            });
        return Err(message);
    }

    parsed
        .get("resp_data")
        .cloned()
        .ok_or_else(|| "Funcloud 响应缺少 resp_data".to_string())
}

fn parse_windows(value: &Value) -> Vec<QuotaWindow> {
    let mut windows = value
        .get("windows")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let window_sec = value_i64(item, "window_sec")?;
                    let limit = value_f64(item, "limit").unwrap_or(0.0);
                    let used = value_f64(item, "used").unwrap_or(0.0);
                    let remaining = value_f64(item, "remaining").unwrap_or((limit - used).max(0.0));
                    let percent = value_f64(item, "percent").unwrap_or_else(|| {
                        if limit > 0.0 {
                            used / limit * 100.0
                        } else {
                            0.0
                        }
                    });
                    let enabled = item
                        .get("enabled")
                        .and_then(Value::as_bool)
                        .unwrap_or(limit > 0.0);
                    let resets_at = value_i64(item, "next_reset_at")
                        .or_else(|| value_i64(item, "next_reset_at_ms"))
                        .and_then(timestamp_to_rfc3339);
                    Some(QuotaWindow {
                        id: format!("funcloud-{window_sec}"),
                        window_sec,
                        enabled,
                        limit,
                        used,
                        remaining,
                        percent: percent.clamp(0.0, 100.0),
                        resets_at,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    windows.sort_by_key(|window| window.window_sec);
    windows
}

fn parse_summary(value: &Value) -> FuncloudSummary {
    FuncloudSummary {
        today_cost: value_f64(value, "today_cost").unwrap_or(0.0),
        rpm: value_f64(value, "rpm").unwrap_or(0.0),
        tpm: value_f64(value, "tpm").unwrap_or(0.0),
        total_req_cnt: value_f64(value, "total_req_cnt").unwrap_or(0.0),
        total_inputs: value_f64(value, "total_inputs").unwrap_or(0.0),
        total_outputs: value_f64(value, "total_outputs").unwrap_or(0.0),
        total_tokens: value_f64(value, "total_tokens").unwrap_or(0.0),
        total_cache_writes: value_f64(value, "total_cache_writes").unwrap_or(0.0),
        total_cache_reads: value_f64(value, "total_cache_reads").unwrap_or(0.0),
        total_cost: value_f64(value, "total_cost").unwrap_or(0.0),
        avg_latency_ms: value_f64(value, "avg_latency_ms").unwrap_or(0.0),
    }
}

fn parse_daily(value: &Value) -> Vec<DailyUsage> {
    value
        .get("list")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| DailyUsage {
                    date: item
                        .get("date")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    req_cnt: value_f64(item, "req_cnt").unwrap_or(0.0),
                    inputs: value_f64(item, "inputs").unwrap_or(0.0),
                    outputs: value_f64(item, "outputs").unwrap_or(0.0),
                    cache_reads: value_f64(item, "cache_reads").unwrap_or(0.0),
                    cache_writes: value_f64(item, "cache_writes").unwrap_or(0.0),
                    cost: value_f64(item, "cost").unwrap_or(0.0),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_models(value: &Value) -> Vec<ModelUsage> {
    value
        .get("list")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| ModelUsage {
                    model: item
                        .get("model")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    req_cnt: value_f64(item, "req_cnt").unwrap_or(0.0),
                    inputs: value_f64(item, "inputs").unwrap_or(0.0),
                    outputs: value_f64(item, "outputs").unwrap_or(0.0),
                    cache_reads: value_f64(item, "cache_reads").unwrap_or(0.0),
                    total_tokens: value_f64(item, "total_tokens").unwrap_or_else(|| {
                        value_f64(item, "inputs").unwrap_or(0.0)
                            + value_f64(item, "outputs").unwrap_or(0.0)
                    }),
                    cost: value_f64(item, "cost").unwrap_or(0.0),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn seven_day_range() -> (i64, i64) {
    let now = Local::now();
    let start_date = now.date_naive() - ChronoDuration::days(6);
    let start = start_date
        .and_hms_opt(0, 0, 0)
        .and_then(|value| Local.from_local_datetime(&value).single())
        .unwrap_or(now);
    (start.timestamp_millis(), now.timestamp_millis())
}

fn timestamp_to_rfc3339(timestamp: i64) -> Option<String> {
    let millis = if timestamp > 10_000_000_000 {
        timestamp
    } else {
        timestamp.saturating_mul(1000)
    };
    chrono::DateTime::<Utc>::from_timestamp_millis(millis).map(|value| value.to_rfc3339())
}

fn value_i64(value: &Value, key: &str) -> Option<i64> {
    match value.get(key)? {
        Value::Number(number) => number
            .as_i64()
            .or_else(|| number.as_f64().map(|value| value.round() as i64)),
        Value::String(text) => text.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn value_f64(value: &Value, key: &str) -> Option<f64> {
    match value.get(key)? {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text
            .trim()
            .trim_start_matches('$')
            .replace(',', "")
            .parse::<f64>()
            .ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_limit_windows() {
        let value = json!({
            "windows": [
                {
                    "window_sec": 604800,
                    "enabled": true,
                    "limit": 50,
                    "used": 12.5,
                    "remaining": 37.5,
                    "percent": 25,
                    "next_reset_at": 1_760_000_000_000_i64
                },
                {
                    "window_sec": 18000,
                    "enabled": false,
                    "limit": 0,
                    "used": 0,
                    "remaining": 0,
                    "percent": 0
                }
            ]
        });

        let windows = parse_windows(&value);
        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].window_sec, 18000);
        assert!(!windows[0].enabled);
        assert_eq!(windows[1].window_sec, 604800);
        assert!((windows[1].remaining - 37.5).abs() < f64::EPSILON);
        assert!(windows[1].resets_at.is_some());
    }

    #[test]
    fn parses_summary_and_lists() {
        let summary = json!({
            "today_cost": 1.25,
            "rpm": "3",
            "tpm": 1200,
            "total_req_cnt": 42,
            "total_tokens": 9000,
            "total_cost": 8.4,
            "avg_latency_ms": 820
        });
        let parsed = parse_summary(&summary);
        assert!((parsed.today_cost - 1.25).abs() < f64::EPSILON);
        assert!((parsed.rpm - 3.0).abs() < f64::EPSILON);

        let daily = json!({
            "list": [
                { "date": "2026-11-03", "req_cnt": 4, "inputs": 100, "outputs": 50, "cost": "0.42" }
            ]
        });
        let rows = parse_daily(&daily);
        assert_eq!(rows.len(), 1);
        assert!((rows[0].cost - 0.42).abs() < f64::EPSILON);
    }
}
