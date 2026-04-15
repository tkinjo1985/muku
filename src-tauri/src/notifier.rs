use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_store::StoreExt;

fn show_toast(app: &AppHandle, body: &str) {
    use tauri_winrt_notification::Toast;

    let app_handle = app.clone();
    let result = Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Muku")
        .text1(body)
        .on_activated(move |_args| {
            eprintln!("[muku] toast activated");
            if let Some(window) = app_handle.get_webview_window("main") {
                crate::force_focus(&window);
            }
            Ok(())
        })
        .show();

    if let Err(e) = result {
        eprintln!("[muku] toast show failed: {e}");
    } else {
        eprintln!("[muku] toast shown: {body}");
    }
}

const SETTINGS_FILE: &str = "settings.json";
const SETTINGS_KEY: &str = "notifications";
const TICK_SECONDS: u64 = 60;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotificationSettings {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_due_minutes_before", rename = "dueMinutesBefore")]
    pub due_minutes_before: i64,
    #[serde(default = "default_notify_on_overdue", rename = "notifyOnOverdue")]
    pub notify_on_overdue: bool,
    #[serde(default = "default_periodic_interval", rename = "periodicIntervalMinutes")]
    pub periodic_interval_minutes: i64,
    #[serde(default = "default_periodic_start", rename = "periodicStartHour")]
    pub periodic_start_hour: i64,
    #[serde(default = "default_periodic_end", rename = "periodicEndHour")]
    pub periodic_end_hour: i64,
}

fn default_enabled() -> bool { true }
fn default_due_minutes_before() -> i64 { 15 }
fn default_notify_on_overdue() -> bool { true }
fn default_periodic_interval() -> i64 { 180 }
fn default_periodic_start() -> i64 { 9 }
fn default_periodic_end() -> i64 { 22 }

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            due_minutes_before: 15,
            notify_on_overdue: true,
            periodic_interval_minutes: 180,
            periodic_start_hour: 9,
            periodic_end_hour: 22,
        }
    }
}

fn jst_offset() -> FixedOffset {
    FixedOffset::east_opt(9 * 3600).unwrap()
}

fn now_jst() -> DateTime<FixedOffset> {
    Utc::now().with_timezone(&jst_offset())
}

fn parse_iso(s: &str) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(s).ok()
}

fn load_settings(app: &AppHandle) -> NotificationSettings {
    let Ok(store) = app.store(SETTINGS_FILE) else {
        return NotificationSettings::default();
    };
    store
        .get(SETTINGS_KEY)
        .and_then(|v| serde_json::from_value::<NotificationSettings>(v).ok())
        .unwrap_or_default()
}

fn within_time_window(now: &DateTime<FixedOffset>, start: i64, end: i64) -> bool {
    use chrono::Timelike;
    let h = now.hour() as i64;
    if start == end {
        return true;
    }
    if start < end {
        h >= start && h < end
    } else {
        h >= start || h < end
    }
}

fn db_path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|p| p.join("muku.db"))
}

async fn connect_pool(path: &PathBuf) -> Option<Pool<Sqlite>> {
    let opts = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(false);
    SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(opts)
        .await
        .ok()
}

#[derive(sqlx::FromRow)]
struct TaskRow {
    id: String,
    title: String,
    due_at: Option<String>,
    last_notified_at: Option<String>,
}

pub struct PeriodicState {
    pub last_sent_at: Option<DateTime<FixedOffset>>,
}

pub fn start(app: AppHandle) {
    let periodic = Arc::new(Mutex::new(PeriodicState { last_sent_at: None }));

    tauri::async_runtime::spawn(async move {
        let Some(path) = db_path(&app) else {
            eprintln!("[muku] notifier: cannot resolve db path");
            return;
        };

        loop {
            tokio::time::sleep(Duration::from_secs(TICK_SECONDS)).await;

            let settings = load_settings(&app);
            if !settings.enabled {
                continue;
            }

            let now = now_jst();

            let Some(pool) = connect_pool(&path).await else {
                continue;
            };

            if let Err(e) = tick_due(&app, &pool, &settings, &now).await {
                eprintln!("[muku] notifier tick_due error: {e}");
            }

            if settings.periodic_interval_minutes > 0
                && within_time_window(
                    &now,
                    settings.periodic_start_hour,
                    settings.periodic_end_hour,
                )
            {
                if let Err(e) = tick_periodic(&app, &pool, &settings, &now, &periodic).await {
                    eprintln!("[muku] notifier tick_periodic error: {e}");
                }
            }

            pool.close().await;
        }
    });
}

async fn insert_assistant_message(pool: &Pool<Sqlite>, content: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO messages (role, content) VALUES ('assistant', ?)")
        .bind(content)
        .execute(pool)
        .await?;
    Ok(())
}

async fn tick_due(
    app: &AppHandle,
    pool: &Pool<Sqlite>,
    settings: &NotificationSettings,
    now: &DateTime<FixedOffset>,
) -> Result<(), sqlx::Error> {
    let rows: Vec<TaskRow> = sqlx::query_as(
        "SELECT id, title, priority, due_at, last_notified_at FROM tasks \
         WHERE status = 'todo' AND due_at IS NOT NULL",
    )
    .fetch_all(pool)
    .await?;

    let window_before = chrono::Duration::minutes(settings.due_minutes_before);
    let overdue_cooldown = chrono::Duration::hours(6);

    for row in rows {
        let Some(due_at_str) = &row.due_at else { continue };
        let Some(due_at) = parse_iso(due_at_str) else { continue };
        let last_notified = row
            .last_notified_at
            .as_deref()
            .and_then(parse_iso);

        let delta = due_at.signed_duration_since(*now);

        let should_notify_pre = delta > chrono::Duration::zero()
            && delta <= window_before
            && last_notified.map_or(true, |t| t < due_at - window_before);

        let should_notify_overdue = settings.notify_on_overdue
            && delta <= chrono::Duration::zero()
            && last_notified.map_or(true, |t| *now - t >= overdue_cooldown);

        if should_notify_pre || should_notify_overdue {
            let toast_body = if should_notify_pre {
                format!("あと {} 分で期限: {}", delta.num_minutes().max(0), row.title)
            } else {
                format!("期限超過: {}", row.title)
            };

            let chat_body = if should_notify_pre {
                format!(
                    "「{}」、あと {} 分で期限です。準備できそう？",
                    row.title,
                    delta.num_minutes().max(0)
                )
            } else {
                format!("「{}」の期限が過ぎたよ。今日中にいけそう？", row.title)
            };

            show_toast(app, &toast_body);
            if let Err(e) = insert_assistant_message(pool, &chat_body).await {
                eprintln!("[muku] insert chat message failed: {e}");
            }
            let _ = app.emit("messages-changed", ());

            let stamp = now.format("%Y-%m-%dT%H:%M:%S%:z").to_string();
            let _ = sqlx::query("UPDATE tasks SET last_notified_at = ? WHERE id = ?")
                .bind(&stamp)
                .bind(&row.id)
                .execute(pool)
                .await;
        }
    }

    Ok(())
}

async fn tick_periodic(
    app: &AppHandle,
    pool: &Pool<Sqlite>,
    settings: &NotificationSettings,
    now: &DateTime<FixedOffset>,
    state: &Arc<Mutex<PeriodicState>>,
) -> Result<(), sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tasks WHERE status = 'todo'",
    )
    .fetch_one(pool)
    .await?;

    if count.0 == 0 {
        return Ok(());
    }

    let should_fire = {
        let guard = state.lock().unwrap();
        match guard.last_sent_at {
            None => true,
            Some(last) => {
                (*now - last) >= chrono::Duration::minutes(settings.periodic_interval_minutes)
            }
        }
    };

    if should_fire {
        show_toast(app, &format!("アクティブなタスク {} 件", count.0));

        if let Ok(mut guard) = state.lock() {
            guard.last_sent_at = Some(*now);
        }
    }

    Ok(())
}
