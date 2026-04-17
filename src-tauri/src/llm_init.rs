use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::AsyncWriteExt;

use crate::{
    job_guard, resolve_model_path, resolve_model_url, resolve_models_dir, spawn_llama_server,
    SidecarHandle,
};

const LLAMA_HEALTH_URL: &str = "http://127.0.0.1:18080/health";
const SERVER_READY_TIMEOUT_SECS: u64 = 300;
const EMIT_THROTTLE: Duration = Duration::from_millis(250);

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LlmStatus {
    Checking,
    Downloading { downloaded: u64, total: u64 },
    ModelLoading,
    Ready,
    Error { message: String },
}

pub struct LlmStatusState(pub Mutex<LlmStatus>);

impl Default for LlmStatusState {
    fn default() -> Self {
        Self(Mutex::new(LlmStatus::Checking))
    }
}

fn emit_status(app: &AppHandle, status: LlmStatus) {
    if let Some(state) = app.try_state::<LlmStatusState>() {
        if let Ok(mut guard) = state.0.lock() {
            *guard = status.clone();
        }
    }
    let _ = app.emit("llm-status", status);
}

pub async fn init(app: AppHandle) {
    emit_status(&app, LlmStatus::Checking);
    let model_path = resolve_model_path(&app);

    if !model_path.exists() {
        if let Err(e) = download_model(&app, &model_path).await {
            eprintln!("[muku] download error: {e}");
            emit_status(&app, LlmStatus::Error {
                message: format!("モデルのダウンロードに失敗しました: {e}"),
            });
            return;
        }
    }

    emit_status(&app, LlmStatus::ModelLoading);

    match spawn_llama_server(&app) {
        Ok(child) => {
            if let Err(e) = job_guard::assign(&child) {
                eprintln!("[muku] job_guard::assign failed: {e}");
            }
            app.manage(SidecarHandle(Mutex::new(Some(child))));
        }
        Err(e) => {
            emit_status(&app, LlmStatus::Error {
                message: format!("llama-server を起動できません: {e}"),
            });
            return;
        }
    }

    match wait_for_server_ready().await {
        Ok(_) => emit_status(&app, LlmStatus::Ready),
        Err(e) => emit_status(&app, LlmStatus::Error {
            message: format!("モデルのロードに失敗しました: {e}"),
        }),
    }
}

fn format_reqwest_error(err: reqwest::Error) -> String {
    use std::error::Error;
    let mut msg = format!("{err}");
    let mut src = err.source();
    while let Some(s) = src {
        msg.push_str(&format!(" | caused by: {s}"));
        src = s.source();
    }
    msg
}

async fn download_model(app: &AppHandle, dest: &Path) -> Result<(), String> {
    let parent = dest
        .parent()
        .ok_or_else(|| "invalid model path".to_string())?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|e| e.to_string())?;

    let tmp_path: PathBuf = dest.with_extension("gguf.tmp");
    let existing: u64 = tokio::fs::metadata(&tmp_path)
        .await
        .ok()
        .map(|m| m.len())
        .unwrap_or(0);

    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let mut req = client.get(resolve_model_url(app));
    if existing > 0 {
        req = req.header("Range", format!("bytes={existing}-"));
    }

    let resp = req.send().await.map_err(format_reqwest_error)?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let total = match resp.headers().get("content-range") {
        Some(v) => v
            .to_str()
            .ok()
            .and_then(|s| s.rsplit('/').next())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0),
        None => resp.content_length().unwrap_or(0) + existing,
    };

    let mut file = tokio::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&tmp_path)
        .await
        .map_err(|e| e.to_string())?;

    let mut downloaded = existing;
    let mut last_emit = Instant::now() - EMIT_THROTTLE;
    emit_status(app, LlmStatus::Downloading { downloaded, total });

    let mut resp = resp;
    loop {
        let chunk = resp.chunk().await.map_err(format_reqwest_error)?;
        let Some(chunk) = chunk else { break };
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        if last_emit.elapsed() >= EMIT_THROTTLE {
            last_emit = Instant::now();
            emit_status(app, LlmStatus::Downloading { downloaded, total });
        }
    }
    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);

    tokio::fs::rename(&tmp_path, dest)
        .await
        .map_err(|e| e.to_string())?;

    emit_status(app, LlmStatus::Downloading { downloaded, total });
    Ok(())
}

async fn wait_for_server_ready() -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;
    let start = Instant::now();
    loop {
        if start.elapsed() > Duration::from_secs(SERVER_READY_TIMEOUT_SECS) {
            return Err("タイムアウト".to_string());
        }
        if let Ok(resp) = client.get(LLAMA_HEALTH_URL).send().await {
            if resp.status().is_success() {
                return Ok(());
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

#[tauri::command]
pub fn get_llm_status(state: tauri::State<'_, LlmStatusState>) -> LlmStatus {
    state.0.lock().map(|s| s.clone()).unwrap_or(LlmStatus::Checking)
}

#[tauri::command]
pub fn retry_llm_init(app: AppHandle, state: tauri::State<'_, LlmStatusState>) -> Result<(), String> {
    let current = state.0.lock().map(|s| s.clone()).unwrap_or(LlmStatus::Checking);
    if !matches!(current, LlmStatus::Error { .. }) {
        return Err("already initializing or ready".to_string());
    }
    if let Some(handle) = app.try_state::<SidecarHandle>() {
        if let Ok(mut guard) = handle.0.lock() {
            if let Some(mut child) = guard.take() {
                let _ = child.kill();
            }
        }
    }
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        init(app_clone).await;
    });
    Ok(())
}

#[tauri::command]
pub fn get_models_dir(app: AppHandle) -> String {
    resolve_models_dir(&app).to_string_lossy().to_string()
}

#[tauri::command]
pub fn open_models_dir(app: AppHandle) -> Result<(), String> {
    let dir = resolve_models_dir(&app);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        std::process::Command::new("explorer")
            .arg(&dir)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(&dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn switch_model(app: AppHandle, selection: String) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;

    if crate::ModelSelection::parse(&selection).is_none() {
        return Err(format!("unknown model selection: {selection}"));
    }

    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("model", serde_json::Value::String(selection));
    store.save().map_err(|e| e.to_string())?;

    restart_llm(app);
    Ok(())
}

#[tauri::command]
pub fn switch_compute(app: AppHandle, mode: String) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;

    if crate::ComputeMode::parse(&mode).is_none() {
        return Err(format!("unknown compute mode: {mode}"));
    }

    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("compute", serde_json::Value::String(mode));
    store.save().map_err(|e| e.to_string())?;

    restart_llm(app);
    Ok(())
}

fn restart_llm(app: AppHandle) {
    if let Some(handle) = app.try_state::<SidecarHandle>() {
        if let Ok(mut guard) = handle.0.lock() {
            if let Some(mut child) = guard.take() {
                let _ = child.kill();
            }
        }
    }

    emit_status(&app, LlmStatus::Checking);

    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        init(app_clone).await;
    });
}
