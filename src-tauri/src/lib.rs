mod commands;
mod db;
mod llm;

use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Mutex;

use tauri::Manager;

pub struct SidecarHandle(pub Mutex<Option<Child>>);

fn resolve_binaries_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries")
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

fn resolve_model_path() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("models/gemma-4-E4B-it-Q4_K_M.gguf")
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
            .join("models/gemma-4-E4B-it-Q4_K_M.gguf")
    }
}

fn spawn_llama_server() -> std::io::Result<Child> {
    let bin_dir = resolve_binaries_dir();
    let exe = bin_dir.join("llama-server-x86_64-pc-windows-msvc.exe");
    let model = resolve_model_path();

    eprintln!("[muku] spawning llama-server: {}", exe.display());
    eprintln!("[muku] model: {}", model.display());

    Command::new(&exe)
        .current_dir(&bin_dir)
        .args([
            "-m",
            model.to_str().unwrap_or_default(),
            "--host",
            "127.0.0.1",
            "--port",
            "18080",
            "-c",
            "4096",
            "-ngl",
            "99",
            "--jinja",
        ])
        .spawn()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:muku.db", db::migrations())
                .build(),
        )
        .setup(|app| {
            match spawn_llama_server() {
                Ok(child) => {
                    app.manage(SidecarHandle(Mutex::new(Some(child))));
                }
                Err(e) => {
                    eprintln!("[muku] failed to spawn llama-server: {e}");
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                if let Some(handle) = window.app_handle().try_state::<SidecarHandle>() {
                    if let Ok(mut guard) = handle.0.lock() {
                        if let Some(mut child) = guard.take() {
                            let _ = child.kill();
                        }
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![commands::chat::chat_send])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
