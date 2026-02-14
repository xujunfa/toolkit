use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

// ---------------------------------------------------------------------------
// DB model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ZenmuxConfig {
    pub ctoken: String,
    pub session_id: String,
    pub session_id_sig: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Internal API deserialization (camelCase from external JSON)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ZenmuxApiRawItem {
    tier_code: String,
    period_type: String,
    period_duration: String,
    cycle_start_time: String,
    cycle_end_time: String,
    used_rate: f64,
    quota_status: i32,
    status: i32,
}

#[derive(Debug, Deserialize)]
struct ZenmuxApiResponse {
    #[allow(dead_code)]
    success: bool,
    data: Vec<ZenmuxApiRawItem>,
}

// ---------------------------------------------------------------------------
// IPC-facing structs (snake_case — codegen reads these field names)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZenmuxQuotaItem {
    pub tier_code: String,
    pub period_type: String,
    pub period_duration: String,
    pub cycle_start_time: String,
    pub cycle_end_time: String,
    pub used_rate: f64,
    pub quota_status: i32,
    pub status: i32,
}

impl From<ZenmuxApiRawItem> for ZenmuxQuotaItem {
    fn from(raw: ZenmuxApiRawItem) -> Self {
        Self {
            tier_code: raw.tier_code,
            period_type: raw.period_type,
            period_duration: raw.period_duration,
            cycle_start_time: raw.cycle_start_time,
            cycle_end_time: raw.cycle_end_time,
            used_rate: raw.used_rate,
            quota_status: raw.quota_status,
            status: raw.status,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZenmuxUsageData {
    pub items: Vec<ZenmuxQuotaItem>,
    pub tray_text: String,
    pub fetched_at: String,
}

// ---------------------------------------------------------------------------
// Managed polling state
// ---------------------------------------------------------------------------

pub struct ZenmuxPollingState {
    pub handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

// ---------------------------------------------------------------------------
// Cookie parsing
// ---------------------------------------------------------------------------

pub fn parse_cookie_string(cookie: &str) -> Result<(String, String, String), String> {
    let mut ctoken: Option<String> = None;
    let mut session_id: Option<String> = None;
    let mut session_id_sig: Option<String> = None;

    for part in cookie.split(';') {
        let part = part.trim();
        if let Some((key, value)) = part.split_once('=') {
            let key = key.trim();
            let value = value.trim().to_string();
            match key {
                "ctoken" => ctoken = Some(value),
                "sessionId" => session_id = Some(value),
                "sessionId.sig" => session_id_sig = Some(value),
                _ => {}
            }
        }
    }

    let ctoken = ctoken.ok_or("Missing 'ctoken' in cookie string")?;
    let session_id = session_id.ok_or("Missing 'sessionId' in cookie string")?;
    let session_id_sig = session_id_sig.ok_or("Missing 'sessionId.sig' in cookie string")?;

    Ok((ctoken, session_id, session_id_sig))
}

// ---------------------------------------------------------------------------
// Tray text formatting
// ---------------------------------------------------------------------------

#[cfg(test)]
pub fn format_tray_text(items: &[ZenmuxQuotaItem]) -> String {
    if items.is_empty() {
        return "ZM: --".to_string();
    }

    let mut parts: Vec<String> = Vec::new();

    for item in items {
        let remaining_pct = ((1.0 - item.used_rate) * 100.0).round() as i32;
        let label = match item.period_type.as_str() {
            "hour_5" => "5h",
            "week" => "W",
            "day" => "D",
            _ => &item.period_type,
        };
        parts.push(format!("{}:{}%", label, remaining_pct));
    }

    if parts.is_empty() {
        "ZM: --".to_string()
    } else {
        parts.join(" ")
    }
}

/// Two-line tray text split for the native AppKit renderer.
pub struct TrayLines {
    pub line1: String,
    pub line2: String,
    /// Flat single-line version for ZenmuxUsageData.tray_text (frontend display).
    pub combined: String,
}

/// Split quota items into two tray lines.
///
/// - 0 items  → line1="ZM", line2="--"
/// - 1 item   → line1=item text, line2="" (rendered as single-line)
/// - 2+ items → line1=first item, line2=remaining items joined with space
pub fn format_tray_lines(items: &[ZenmuxQuotaItem]) -> TrayLines {
    if items.is_empty() {
        return TrayLines {
            line1: "ZM".to_string(),
            line2: "--".to_string(),
            combined: "ZM: --".to_string(),
        };
    }

    let parts: Vec<String> = items
        .iter()
        .map(|item| {
            let remaining_pct = ((1.0 - item.used_rate) * 100.0).round() as i32;
            let label = match item.period_type.as_str() {
                "hour_5" => "5h",
                "week" => "W",
                "day" => "D",
                _ => &item.period_type,
            };
            format!("{}:{}%", label, remaining_pct)
        })
        .collect();

    let combined = parts.join(" ");

    if parts.len() == 1 {
        TrayLines {
            line1: parts[0].clone(),
            line2: String::new(),
            combined,
        }
    } else {
        TrayLines {
            line1: parts[0].clone(),
            line2: parts[1..].join(" "),
            combined,
        }
    }
}

/// Update the tray icon with two-line text via the native AppKit bridge.
fn update_tray_two_line(app: &AppHandle, items: &[ZenmuxQuotaItem]) {
    let lines = format_tray_lines(items);

    if let Some(tray) = app.tray_by_id("main-tray") {
        if lines.line2.is_empty() {
            // Single item: use plain title
            let _ = tray.with_inner_tray_icon(move |inner| {
                crate::tray_text::set_plain_title(inner, &lines.line1);
            });
        } else {
            // Multiple items: use two-line attributed title
            let _ = tray.with_inner_tray_icon(move |inner| {
                crate::tray_text::set_two_line_title(inner, &lines.line1, &lines.line2);
            });
        }
    }
}

// ---------------------------------------------------------------------------
// DB helpers (two-tier pattern)
// ---------------------------------------------------------------------------

pub async fn get_zenmux_config_by_pool(db: &SqlitePool) -> Result<ZenmuxConfig, String> {
    let row = sqlx::query_as::<_, ZenmuxConfig>(
        "SELECT ctoken, session_id, session_id_sig, updated_at FROM zenmux_config WHERE id = 1",
    )
    .fetch_optional(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.unwrap_or(ZenmuxConfig {
        ctoken: String::new(),
        session_id: String::new(),
        session_id_sig: String::new(),
        updated_at: String::new(),
    }))
}

pub async fn set_zenmux_config_by_pool(
    db: &SqlitePool,
    cookie: &str,
) -> Result<ZenmuxConfig, String> {
    let (ctoken, session_id, session_id_sig) = parse_cookie_string(cookie)?;

    sqlx::query(
        "INSERT INTO zenmux_config (id, ctoken, session_id, session_id_sig, updated_at) VALUES (1, ?, ?, ?, datetime('now'))
         ON CONFLICT(id) DO UPDATE SET ctoken = excluded.ctoken, session_id = excluded.session_id, session_id_sig = excluded.session_id_sig, updated_at = datetime('now')",
    )
    .bind(&ctoken)
    .bind(&session_id)
    .bind(&session_id_sig)
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    get_zenmux_config_by_pool(db).await
}

// ---------------------------------------------------------------------------
// HTTP fetch
// ---------------------------------------------------------------------------

pub async fn fetch_zenmux_usage(config: &ZenmuxConfig) -> Result<ZenmuxUsageData, String> {
    let url = format!(
        "https://zenmux.ai/api/subscription/get_current_usage?ctoken={}",
        config.ctoken
    );
    let cookie_header = format!(
        "ctoken={}; sessionId={}; sessionId.sig={}",
        config.ctoken, config.session_id, config.session_id_sig
    );

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("accept", "application/json")
        .header("cookie", &cookie_header)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("API returned status {}", resp.status()));
    }

    let api_resp: ZenmuxApiResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    let items: Vec<ZenmuxQuotaItem> = api_resp.data.into_iter().map(|r| r.into()).collect();
    let tray_text = format_tray_lines(&items).combined;
    let fetched_at = chrono::Utc::now().to_rfc3339();

    Ok(ZenmuxUsageData {
        items,
        tray_text,
        fetched_at,
    })
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_zenmux_config(db: State<'_, SqlitePool>) -> Result<ZenmuxConfig, String> {
    get_zenmux_config_by_pool(db.inner()).await
}

#[tauri::command]
pub async fn set_zenmux_config(
    db: State<'_, SqlitePool>,
    cookie: String,
) -> Result<ZenmuxConfig, String> {
    set_zenmux_config_by_pool(db.inner(), &cookie).await
}

#[tauri::command]
pub async fn get_zenmux_usage(
    app: AppHandle,
    db: State<'_, SqlitePool>,
) -> Result<ZenmuxUsageData, String> {
    let config = get_zenmux_config_by_pool(db.inner()).await?;
    if config.ctoken.is_empty() {
        return Err("ZenMux config not set. Please save your cookie first.".to_string());
    }

    let usage = fetch_zenmux_usage(&config).await?;

    // Update tray with two-line display
    update_tray_two_line(&app, &usage.items);

    // Emit event
    let _ = app.emit("zenmux-usage-updated", &usage);

    Ok(usage)
}

#[tauri::command]
pub async fn start_zenmux_polling(
    app: AppHandle,
    db: State<'_, SqlitePool>,
    polling_state: State<'_, ZenmuxPollingState>,
) -> Result<String, String> {
    let config = get_zenmux_config_by_pool(db.inner()).await?;
    if config.ctoken.is_empty() {
        return Err("ZenMux config not set. Please save your cookie first.".to_string());
    }

    let mut guard = polling_state.handle.lock().await;
    if let Some(h) = guard.take() {
        h.abort();
    }

    let handle = spawn_polling_loop(app, config);
    *guard = Some(handle);

    Ok("Polling started".to_string())
}

#[tauri::command]
pub async fn stop_zenmux_polling(
    app: AppHandle,
    polling_state: State<'_, ZenmuxPollingState>,
) -> Result<String, String> {
    let mut guard = polling_state.handle.lock().await;
    if let Some(h) = guard.take() {
        h.abort();
    }

    // Reset tray
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_title(Some("Toolkit"));
    }

    Ok("Polling stopped".to_string())
}

// ---------------------------------------------------------------------------
// Polling loop helper
// ---------------------------------------------------------------------------

pub fn spawn_polling_loop(app: AppHandle, config: ZenmuxConfig) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

        loop {
            interval.tick().await;

            match fetch_zenmux_usage(&config).await {
                Ok(usage) => {
                    update_tray_two_line(&app, &usage.items);
                    let _ = app.emit("zenmux-usage-updated", &usage);
                }
                Err(_) => {
                    if let Some(tray) = app.tray_by_id("main-tray") {
                        let _ = tray.set_title(Some("ZM: --"));
                    }
                }
            }
        }
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cookie_valid() {
        let cookie =
            "ctoken=abc123; sessionId=sess-456; sessionId.sig=sig-789";
        let (ctoken, session_id, session_id_sig) = parse_cookie_string(cookie).unwrap();
        assert_eq!(ctoken, "abc123");
        assert_eq!(session_id, "sess-456");
        assert_eq!(session_id_sig, "sig-789");
    }

    #[test]
    fn parse_cookie_with_extra_fields() {
        let cookie =
            "other=xxx; ctoken=abc; sessionId=sess; extra=yyy; sessionId.sig=sig";
        let (ctoken, session_id, session_id_sig) = parse_cookie_string(cookie).unwrap();
        assert_eq!(ctoken, "abc");
        assert_eq!(session_id, "sess");
        assert_eq!(session_id_sig, "sig");
    }

    #[test]
    fn parse_cookie_missing_ctoken() {
        let cookie = "sessionId=sess; sessionId.sig=sig";
        let result = parse_cookie_string(cookie);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ctoken"));
    }

    #[test]
    fn parse_cookie_missing_session_id() {
        let cookie = "ctoken=abc; sessionId.sig=sig";
        let result = parse_cookie_string(cookie);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("sessionId"));
    }

    #[test]
    fn parse_cookie_missing_session_id_sig() {
        let cookie = "ctoken=abc; sessionId=sess";
        let result = parse_cookie_string(cookie);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("sessionId.sig"));
    }

    #[test]
    fn format_tray_lines_empty() {
        let lines = format_tray_lines(&[]);
        assert_eq!(lines.line1, "ZM");
        assert_eq!(lines.line2, "--");
        assert_eq!(lines.combined, "ZM: --");
    }

    #[test]
    fn format_tray_lines_single_item() {
        let items = vec![ZenmuxQuotaItem {
            tier_code: "max".to_string(),
            period_type: "hour_5".to_string(),
            period_duration: "5".to_string(),
            cycle_start_time: "".to_string(),
            cycle_end_time: "".to_string(),
            used_rate: 0.24,
            quota_status: 0,
            status: 0,
        }];
        let lines = format_tray_lines(&items);
        assert_eq!(lines.line1, "5h:76%");
        assert_eq!(lines.line2, "");
        assert_eq!(lines.combined, "5h:76%");
    }

    #[test]
    fn format_tray_lines_two_items() {
        let items = vec![
            ZenmuxQuotaItem {
                tier_code: "max".to_string(),
                period_type: "hour_5".to_string(),
                period_duration: "5".to_string(),
                cycle_start_time: "".to_string(),
                cycle_end_time: "".to_string(),
                used_rate: 0.24,
                quota_status: 0,
                status: 0,
            },
            ZenmuxQuotaItem {
                tier_code: "max".to_string(),
                period_type: "week".to_string(),
                period_duration: "168".to_string(),
                cycle_start_time: "".to_string(),
                cycle_end_time: "".to_string(),
                used_rate: 0.62,
                quota_status: 0,
                status: 0,
            },
        ];
        let lines = format_tray_lines(&items);
        assert_eq!(lines.line1, "5h:76%");
        assert_eq!(lines.line2, "W:38%");
        assert_eq!(lines.combined, "5h:76% W:38%");
    }

    #[test]
    fn format_tray_lines_three_items() {
        let items = vec![
            ZenmuxQuotaItem {
                tier_code: "max".to_string(),
                period_type: "hour_5".to_string(),
                period_duration: "5".to_string(),
                cycle_start_time: "".to_string(),
                cycle_end_time: "".to_string(),
                used_rate: 0.10,
                quota_status: 0,
                status: 0,
            },
            ZenmuxQuotaItem {
                tier_code: "max".to_string(),
                period_type: "week".to_string(),
                period_duration: "168".to_string(),
                cycle_start_time: "".to_string(),
                cycle_end_time: "".to_string(),
                used_rate: 0.50,
                quota_status: 0,
                status: 0,
            },
            ZenmuxQuotaItem {
                tier_code: "max".to_string(),
                period_type: "day".to_string(),
                period_duration: "24".to_string(),
                cycle_start_time: "".to_string(),
                cycle_end_time: "".to_string(),
                used_rate: 0.75,
                quota_status: 0,
                status: 0,
            },
        ];
        let lines = format_tray_lines(&items);
        assert_eq!(lines.line1, "5h:90%");
        assert_eq!(lines.line2, "W:50% D:25%");
        assert_eq!(lines.combined, "5h:90% W:50% D:25%");
    }

    #[test]
    fn format_tray_text_both_periods() {
        let items = vec![
            ZenmuxQuotaItem {
                tier_code: "max".to_string(),
                period_type: "hour_5".to_string(),
                period_duration: "5".to_string(),
                cycle_start_time: "".to_string(),
                cycle_end_time: "".to_string(),
                used_rate: 0.24,
                quota_status: 0,
                status: 0,
            },
            ZenmuxQuotaItem {
                tier_code: "max".to_string(),
                period_type: "week".to_string(),
                period_duration: "168".to_string(),
                cycle_start_time: "".to_string(),
                cycle_end_time: "".to_string(),
                used_rate: 0.62,
                quota_status: 0,
                status: 0,
            },
        ];
        let text = format_tray_text(&items);
        assert_eq!(text, "5h:76% W:38%");
    }

    #[test]
    fn format_tray_text_empty_list() {
        let text = format_tray_text(&[]);
        assert_eq!(text, "ZM: --");
    }

    #[test]
    fn api_response_deserialization() {
        let json = r#"{
            "success": true,
            "data": [
                {
                    "tierCode": "max",
                    "periodType": "week",
                    "periodDuration": "168",
                    "cycleStartTime": "2026-02-07T00:40:05.000Z",
                    "cycleEndTime": "2026-02-14T00:40:05.000Z",
                    "usedRate": 0.6198,
                    "quotaStatus": 0,
                    "status": 0
                },
                {
                    "tierCode": "max",
                    "periodType": "hour_5",
                    "periodDuration": "5",
                    "cycleStartTime": "2026-02-13T16:13:06.000Z",
                    "cycleEndTime": "2026-02-13T21:13:06.000Z",
                    "usedRate": 0.2354,
                    "quotaStatus": 0,
                    "status": 0
                }
            ]
        }"#;

        let resp: ZenmuxApiResponse = serde_json::from_str(json).expect("deserialize");
        assert!(resp.success);
        assert_eq!(resp.data.len(), 2);
        assert_eq!(resp.data[0].tier_code, "max");
        assert_eq!(resp.data[0].period_type, "week");
        assert_eq!(resp.data[1].period_type, "hour_5");

        // Convert to IPC-facing type
        let items: Vec<ZenmuxQuotaItem> = resp.data.into_iter().map(|r| r.into()).collect();
        assert_eq!(items[0].tier_code, "max");
        assert_eq!(items[1].used_rate, 0.2354);
    }

    #[tokio::test]
    async fn db_config_crud() {
        let db = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect sqlite memory");
        sqlx::query(
            "CREATE TABLE zenmux_config (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                ctoken TEXT NOT NULL DEFAULT '',
                session_id TEXT NOT NULL DEFAULT '',
                session_id_sig TEXT NOT NULL DEFAULT '',
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&db)
        .await
        .expect("create table");

        // Get returns empty default when no row
        let config = get_zenmux_config_by_pool(&db).await.unwrap();
        assert_eq!(config.ctoken, "");
        assert_eq!(config.session_id, "");

        // Set via cookie string
        let cookie = "ctoken=tok1; sessionId=sid1; sessionId.sig=sig1";
        let config = set_zenmux_config_by_pool(&db, cookie).await.unwrap();
        assert_eq!(config.ctoken, "tok1");
        assert_eq!(config.session_id, "sid1");
        assert_eq!(config.session_id_sig, "sig1");

        // Upsert overwrites
        let cookie2 = "ctoken=tok2; sessionId=sid2; sessionId.sig=sig2";
        let config = set_zenmux_config_by_pool(&db, cookie2).await.unwrap();
        assert_eq!(config.ctoken, "tok2");

        // Only one row
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM zenmux_config")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(count.0, 1);
    }
}
