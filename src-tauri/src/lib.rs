mod commands;
mod db;
mod job_guard;
mod llm;
mod llm_init;
mod notifier;

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

#[cfg(windows)]
fn set_app_user_model_id() {
    use windows::core::HSTRING;
    use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;
    unsafe {
        let id: HSTRING = "com.takumi.muku.Muku".into();
        let _ = SetCurrentProcessExplicitAppUserModelID(&id);
    }
}

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WindowEvent};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub struct SidecarHandle(pub Mutex<Option<Child>>);

fn resolve_binaries_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries")
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
            .join("binaries")
    }
}

use tauri_plugin_store::StoreExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelSelection {
    E2B,
    E4B,
}

impl ModelSelection {
    pub fn id(self) -> &'static str {
        match self {
            ModelSelection::E2B => "e2b",
            ModelSelection::E4B => "e4b",
        }
    }

    pub fn filename(self) -> &'static str {
        match self {
            ModelSelection::E2B => "gemma-4-E2B-it-Q4_K_M.gguf",
            ModelSelection::E4B => "gemma-4-E4B-it-Q4_K_M.gguf",
        }
    }

    pub fn url(self) -> &'static str {
        match self {
            ModelSelection::E2B => "https://huggingface.co/unsloth/gemma-4-E2B-it-GGUF/resolve/main/gemma-4-E2B-it-Q4_K_M.gguf",
            ModelSelection::E4B => "https://huggingface.co/ggml-org/gemma-4-E4B-it-GGUF/resolve/main/gemma-4-E4B-it-Q4_K_M.gguf",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "e2b" => Some(ModelSelection::E2B),
            "e4b" => Some(ModelSelection::E4B),
            _ => None,
        }
    }
}

pub fn read_model_selection(app: &AppHandle) -> ModelSelection {
    app.store("settings.json")
        .ok()
        .and_then(|s| s.get("model"))
        .and_then(|v| v.as_str().map(str::to_string))
        .and_then(|s| ModelSelection::parse(&s))
        .unwrap_or(ModelSelection::E2B)
}

fn models_dir(app: &AppHandle) -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("models")
    } else {
        app.path()
            .app_local_data_dir()
            .ok()
            .map(|p| p.join("models"))
            .unwrap_or_else(|| {
                std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("models")
            })
    }
}

pub fn resolve_model_path(app: &AppHandle) -> PathBuf {
    models_dir(app).join(read_model_selection(app).filename())
}

pub fn resolve_model_url(app: &AppHandle) -> &'static str {
    read_model_selection(app).url()
}

pub fn spawn_llama_server(app: &AppHandle) -> std::io::Result<Child> {
    let bin_dir = resolve_binaries_dir();
    let exe = bin_dir.join("llama-server-x86_64-pc-windows-msvc.exe");
    let model = resolve_model_path(app);

    eprintln!("[muku] spawning llama-server: {}", exe.display());
    eprintln!("[muku] model: {}", model.display());

    let mut cmd = Command::new(&exe);
    cmd.current_dir(&bin_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
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
    ]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd.spawn()
}

fn kill_sidecar(app: &AppHandle) {
    if let Some(handle) = app.try_state::<SidecarHandle>() {
        if let Ok(mut guard) = handle.0.lock() {
            if let Some(mut child) = guard.take() {
                let _ = child.kill();
            }
        }
    }
}

pub fn force_focus(window: &tauri::WebviewWindow) {
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_always_on_top(true);
    let _ = window.set_focus();
    let _ = window.set_always_on_top(false);
}

fn toggle_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let visible = window.is_visible().unwrap_or(false);
    let focused = window.is_focused().unwrap_or(false);
    if visible && focused {
        let _ = window.hide();
    } else {
        force_focus(&window);
    }
}

fn toggle_shortcut_candidates() -> Vec<(&'static str, Shortcut)> {
    let ctrl_alt = Some(Modifiers::CONTROL | Modifiers::ALT);
    let ctrl_shift_alt = Some(Modifiers::CONTROL | Modifiers::SHIFT | Modifiers::ALT);
    let win_alt = Some(Modifiers::SUPER | Modifiers::ALT);
    vec![
        ("Ctrl+Alt+M", Shortcut::new(ctrl_alt, Code::KeyM)),
        ("Ctrl+Shift+Alt+M", Shortcut::new(ctrl_shift_alt, Code::KeyM)),
        ("Win+Alt+M", Shortcut::new(win_alt, Code::KeyM)),
        ("Ctrl+Shift+Alt+Space", Shortcut::new(ctrl_shift_alt, Code::Space)),
    ]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(windows)]
    set_app_user_model_id();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                force_focus(&window);
            }
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:muku.db", db::migrations())
                .build(),
        )
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle_window(app);
                    }
                })
                .build(),
        )
        .setup(move |app| {
            let gs = app.global_shortcut();
            let mut registered: Option<&'static str> = None;
            for (label, shortcut) in toggle_shortcut_candidates() {
                match gs.register(shortcut) {
                    Ok(_) => {
                        eprintln!("[muku] registered toggle shortcut: {label}");
                        registered = Some(label);
                        break;
                    }
                    Err(e) => eprintln!("[muku] shortcut {label} unavailable: {e}"),
                }
            }
            if registered.is_none() {
                eprintln!("[muku] no toggle shortcut could be registered; use tray icon instead");
            }

            app.manage(llm_init::LlmStatusState::default());
            let init_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                llm_init::init(init_handle).await;
            });

            let show_hide = MenuItem::with_id(app, "toggle", "表示/非表示", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_hide, &quit])?;

            notifier::start(app.handle().clone());

            let _tray = TrayIconBuilder::with_id("muku-tray")
                .icon(app.default_window_icon().cloned().unwrap())
                .tooltip("Muku - AIタスクマネージャー")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "toggle" => toggle_window(app),
                    "quit" => {
                        kill_sidecar(app);
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::chat::chat_send,
            llm_init::get_llm_status,
            llm_init::retry_llm_init,
            llm_init::switch_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
