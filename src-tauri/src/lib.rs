use shu_net_keeper::config::{APPConfig, validate_config};
use shu_net_keeper::core;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "settings.json";
const CONFIG_KEY: &str = "config";

// ─── Shared state ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub connected: bool,
    pub ip: Option<String>,
    pub last_check: Option<String>,
    pub last_error: Option<String>,
    pub login_count: u32,
}

impl Default for DaemonStatus {
    fn default() -> Self {
        Self {
            running: false,
            connected: false,
            ip: None,
            last_check: None,
            last_error: None,
            login_count: 0,
        }
    }
}

pub struct AppState {
    pub daemon_running: Arc<AtomicBool>,
    pub status: Arc<Mutex<DaemonStatus>>,
    pub logs: Arc<Mutex<Vec<String>>>,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn now_str() -> String {
    chrono::Local::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn add_log(logs: &Arc<Mutex<Vec<String>>>, app_handle: &AppHandle, msg: &str) {
    let ts = chrono::Local::now().format("%H:%M:%S").to_string();
    let entry = format!("[{}] {}", ts, msg);
    {
        let mut v = logs.lock().unwrap();
        v.push(entry.clone());
        if v.len() > 300 {
            let excess = v.len() - 300;
            v.drain(0..excess);
        }
    }
    let _ = app_handle.emit("log-entry", entry);
}

fn emit_status(app_handle: &AppHandle, status: &Arc<Mutex<DaemonStatus>>) {
    let s = status.lock().unwrap().clone();
    let _ = app_handle.emit("status-update", s);
}

fn load_config_from_store(app_handle: &AppHandle) -> Result<APPConfig, String> {
    let store = app_handle
        .store(STORE_FILE)
        .map_err(|e| e.to_string())?;
    let value = store
        .get(CONFIG_KEY)
        .ok_or_else(|| "尚未保存配置，请先在「配置」页填写并保存".to_string())?;
    serde_json::from_value(value).map_err(|e| format!("配置解析失败: {}", e))
}

// ─── Daemon loop ─────────────────────────────────────────────────────────────

fn daemon_loop(
    config: shu_net_keeper::config::APPConfigValidated,
    running: Arc<AtomicBool>,
    status: Arc<Mutex<DaemonStatus>>,
    logs: Arc<Mutex<Vec<String>>>,
    app_handle: AppHandle,
) {
    let mut last_ip: Option<String> = None;

    {
        let mut s = status.lock().unwrap();
        s.running = true;
    }
    add_log(&logs, &app_handle, "守护进程已启动");
    emit_status(&app_handle, &status);

    while running.load(Ordering::SeqCst) {
        add_log(&logs, &app_handle, "正在检查网络连接状态...");

        match core::network::check_network_connection(&mut last_ip) {
            Ok(true) => {
                let ip = last_ip.clone();
                add_log(
                    &logs,
                    &app_handle,
                    &format!("✓ 网络连接正常，IP: {}", ip.as_deref().unwrap_or("未知")),
                );
                {
                    let mut s = status.lock().unwrap();
                    s.connected = true;
                    s.ip = ip;
                    s.last_check = Some(now_str());
                    s.last_error = None;
                }
                emit_status(&app_handle, &status);
            }
            Ok(false) => {
                add_log(&logs, &app_handle, "网络未连接，尝试登录...");
                match core::login::network_login(&config.username, &config.password) {
                    Ok(()) => {
                        let current_ip = core::network::get_host_ip()
                            .ok()
                            .flatten()
                            .unwrap_or_else(|| "未知".to_string());
                        let ip_changed =
                            matches!(&last_ip, Some(old) if old != &current_ip);

                        add_log(
                            &logs,
                            &app_handle,
                            &format!("✓ 登录成功，IP: {}", current_ip),
                        );

                        if let Some(smtp) = &config.smtp {
                            match core::email::send_login_notification(
                                smtp,
                                &config.username,
                                &current_ip,
                                ip_changed,
                            ) {
                                Ok(()) => add_log(&logs, &app_handle, "✓ 邮件通知已发送"),
                                Err(e) => add_log(
                                    &logs,
                                    &app_handle,
                                    &format!("✗ 邮件发送失败: {}", e),
                                ),
                            }
                        }

                        {
                            let mut s = status.lock().unwrap();
                            s.connected = true;
                            s.ip = Some(current_ip.clone());
                            s.last_check = Some(now_str());
                            s.last_error = None;
                            s.login_count += 1;
                        }
                        last_ip = Some(current_ip);
                        emit_status(&app_handle, &status);
                    }
                    Err(e) => {
                        add_log(&logs, &app_handle, &format!("✗ 登录失败: {}", e));
                        {
                            let mut s = status.lock().unwrap();
                            s.connected = false;
                            s.last_check = Some(now_str());
                            s.last_error = Some(e.to_string());
                        }
                        emit_status(&app_handle, &status);
                    }
                }
            }
            Err(e) => {
                add_log(&logs, &app_handle, &format!("✗ 网络检查失败: {}", e));
                {
                    let mut s = status.lock().unwrap();
                    s.connected = false;
                    s.last_check = Some(now_str());
                    s.last_error = Some(e.to_string());
                }
                emit_status(&app_handle, &status);
            }
        }

        let interval = config.interval;
        add_log(
            &logs,
            &app_handle,
            &format!("等待 {} 秒后再次检查...", interval),
        );
        for _ in 0..interval {
            if !running.load(Ordering::SeqCst) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    {
        let mut s = status.lock().unwrap();
        s.running = false;
    }
    add_log(&logs, &app_handle, "守护进程已停止");
    emit_status(&app_handle, &status);
}

// ─── Tauri commands ───────────────────────────────────────────────────────────

#[tauri::command]
fn get_config(app_handle: AppHandle) -> Result<Option<APPConfig>, String> {
    let store = app_handle
        .store(STORE_FILE)
        .map_err(|e| e.to_string())?;
    match store.get(CONFIG_KEY) {
        Some(v) => {
            let cfg: APPConfig =
                serde_json::from_value(v).map_err(|e| e.to_string())?;
            Ok(Some(cfg))
        }
        None => Ok(None),
    }
}

#[tauri::command]
fn save_config(config: APPConfig, app_handle: AppHandle) -> Result<(), String> {
    let store = app_handle
        .store(STORE_FILE)
        .map_err(|e| e.to_string())?;
    let value = serde_json::to_value(&config).map_err(|e| e.to_string())?;
    store.set(CONFIG_KEY, value);
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn start_daemon(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    if state.daemon_running.load(Ordering::SeqCst) {
        return Err("守护进程已在运行".to_string());
    }

    let config = load_config_from_store(&app_handle)?;
    let validated = validate_config(&config).map_err(|e| format!("配置验证失败: {}", e))?;

    state.daemon_running.store(true, Ordering::SeqCst);

    let running = Arc::clone(&state.daemon_running);
    let status = Arc::clone(&state.status);
    let logs = Arc::clone(&state.logs);

    std::thread::spawn(move || {
        daemon_loop(validated, running, status, logs, app_handle);
    });

    Ok(())
}

#[tauri::command]
fn stop_daemon(state: State<'_, AppState>) -> Result<(), String> {
    state.daemon_running.store(false, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
fn get_status(state: State<'_, AppState>) -> DaemonStatus {
    state.status.lock().unwrap().clone()
}

#[tauri::command]
fn get_logs(state: State<'_, AppState>) -> Vec<String> {
    state.logs.lock().unwrap().clone()
}

// ─── Entry point ──────────────────────────────────────────────────────────────

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            app.manage(AppState {
                daemon_running: Arc::new(AtomicBool::new(false)),
                status: Arc::new(Mutex::new(DaemonStatus::default())),
                logs: Arc::new(Mutex::new(Vec::new())),
            });

            // ── System tray ───────────────────────────────────────────────
            let show_item =
                MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit_item =
                MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let icon = app
                .default_window_icon()
                .cloned()
                .expect("no app icon found");

            TrayIconBuilder::new()
                .icon(icon)
                .tooltip("SHU Net Keeper")
                .menu(&menu)
                .show_menu_on_left_click(false)
                // Left-click: show/focus the window
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                // Right-click menu events
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "quit" => {
                        if let Some(state) = app.try_state::<AppState>() {
                            state.daemon_running.store(false, Ordering::SeqCst);
                        }
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        // Clicking ✕ hides to tray instead of quitting
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            start_daemon,
            stop_daemon,
            get_status,
            get_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
