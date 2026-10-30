mod audio_monitor;
mod clipboard;
mod dev_hub;
mod dev_ipc;
mod drop_shelf;
mod git;
mod hardware;
mod media;
mod settings;
mod system;
mod tray;
mod weather;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{
    Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Size, State, WebviewWindow,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub struct AppState {
    pub current_monitor_name: Mutex<Option<String>>,
    pub system_tracker: Mutex<system::SystemTracker>,
    pub dev_manager: Arc<dev_ipc::DevManager>,
    pub clipboard_manager: Arc<clipboard::ClipboardManager>,
    pub shelf_manager: Arc<drop_shelf::ShelfManager>,
    pub last_programmatic_pos: Mutex<Option<(i32, i32, std::time::Instant)>>,
    pub pending_user_pos: Mutex<Option<(f64, f64, Option<String>, std::time::Instant)>>,
    pub is_user_dragging: AtomicBool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IslandNotification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub app_name: Option<String>,
    pub icon: Option<String>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonitorInfo {
    pub name: String,
    pub is_current: bool,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
}

fn resolve_monitor_from_window(
    available: Vec<tauri::Monitor>,
    current: Option<tauri::Monitor>,
    primary: Option<tauri::Monitor>,
    target_name: Option<&str>,
) -> Option<tauri::Monitor> {
    if let Some(name) = target_name {
        if let Some(m) = available.iter().find(|m| m.name().map_or(false, |n| n == name)) {
            return Some(m.clone());
        }
    }

    if let Some(m) = current {
        return Some(m);
    }

    primary.or_else(|| available.into_iter().next())
}

fn resolve_monitor(
    window: &WebviewWindow,
    target_name: Option<&str>,
) -> Option<tauri::Monitor> {
    let available = window.available_monitors().ok().unwrap_or_default();
    let current = window.current_monitor().ok().flatten();
    let primary = window.primary_monitor().ok().flatten();
    resolve_monitor_from_window(available, current, primary, target_name)
}

fn find_monitor_from_coords(
    available: &[tauri::Monitor],
    current: Option<tauri::Monitor>,
    primary: Option<tauri::Monitor>,
    pos: &tauri::PhysicalPosition<i32>,
    win_w: u32,
) -> Option<tauri::Monitor> {
    let center_x = pos.x + (win_w as i32) / 2;
    let center_y = pos.y + 20;

    // 1. Point containment (center_x, center_y) inside monitor bounding box
    for m in available {
        let m_pos = m.position();
        let m_size = m.size();
        if center_x >= m_pos.x
            && center_x < m_pos.x + m_size.width as i32
            && center_y >= m_pos.y
            && center_y < m_pos.y + m_size.height as i32
        {
            return Some(m.clone());
        }
    }

    // 2. Horizontal containment (center_x inside monitor X bounds)
    for m in available {
        let m_pos = m.position();
        let m_size = m.size();
        if center_x >= m_pos.x && center_x < m_pos.x + m_size.width as i32 {
            return Some(m.clone());
        }
    }

    current
        .or(primary)
        .or_else(|| available.first().cloned())
}

fn find_monitor_for_win(
    window: &tauri::Window,
    pos: &tauri::PhysicalPosition<i32>,
    win_w: u32,
) -> Option<tauri::Monitor> {
    let available = window.available_monitors().ok().unwrap_or_default();
    let current = window.current_monitor().ok().flatten();
    let primary = window.primary_monitor().ok().flatten();
    find_monitor_from_coords(&available, current, primary, pos, win_w)
}

fn find_monitor_for_webview(
    window: &WebviewWindow,
    pos: &tauri::PhysicalPosition<i32>,
    win_w: u32,
) -> Option<tauri::Monitor> {
    let available = window.available_monitors().ok().unwrap_or_default();
    let current = window.current_monitor().ok().flatten();
    let primary = window.primary_monitor().ok().flatten();
    find_monitor_from_coords(&available, current, primary, pos, win_w)
}

pub fn compute_target_coordinates(
    monitor: &tauri::Monitor,
    target_physical_width: u32,
    top_margin_override: f64,
) -> (i32, i32) {
    let settings = settings::load_settings_from_disk();
    let scale_factor = monitor.scale_factor();
    let monitor_pos = monitor.position();
    let monitor_size = monitor.size();

    let margin_side = (20.0 * scale_factor).round() as i32;
    let x_offset_phys = (settings.x_offset * scale_factor).round() as i32;

    let offset_x = match settings.horizontal_position.to_lowercase().as_str() {
        "left" => margin_side + x_offset_phys,
        "right" => {
            if monitor_size.width > target_physical_width {
                (monitor_size.width - target_physical_width) as i32 - margin_side + x_offset_phys
            } else {
                x_offset_phys
            }
        }
        "custom" => {
            let center_phys = x_offset_phys.clamp(30, (monitor_size.width as i32 - 30).max(30));
            let half_w = (target_physical_width / 2) as i32;
            center_phys - half_w
        }
        _ => {
            // "center" default
            let center_base = if monitor_size.width > target_physical_width {
                ((monitor_size.width - target_physical_width) / 2) as i32
            } else {
                0
            };
            center_base + x_offset_phys
        }
    };

    let effective_top_margin = if settings.horizontal_position.eq_ignore_ascii_case("custom") {
        let max_top = (monitor_size.height as f64 / scale_factor - 40.0).max(0.0);
        settings.top_margin.clamp(0.0, max_top)
    } else if top_margin_override >= 0.0 {
        top_margin_override
    } else {
        settings.top_margin
    };

    let target_x = monitor_pos.x + offset_x;
    let target_y = monitor_pos.y + (effective_top_margin * scale_factor).round() as i32;

    (target_x, target_y)
}

pub const FIXED_WINDOW_WIDTH: f64 = 800.0;
pub const FIXED_WINDOW_HEIGHT: f64 = 520.0;

#[tauri::command]
fn resize_and_center_island(
    window: WebviewWindow,
    state: State<'_, AppState>,
    width: f64,
    height: f64,
    top_margin: f64,
) -> Result<(), String> {
    // 1. If user is actively dragging the window with mouse right now, don't move it
    let is_actively_dragging = state.is_user_dragging.load(Ordering::SeqCst) || {
        let lock = state.pending_user_pos.lock().unwrap();
        if let Some((_, _, _, inst)) = *lock {
            inst.elapsed() < std::time::Duration::from_millis(150)
        } else {
            false
        }
    };
    if is_actively_dragging {
        return Ok(());
    }

    // 2. If there is any pending user manual position from dragging that just settled, commit it now
    let pending = state.pending_user_pos.lock().unwrap().take();
    if let Some((center_rel, y_rel, mon_name, _)) = pending {
        state.is_user_dragging.store(false, Ordering::SeqCst);
        if let Some(name) = mon_name {
            *state.current_monitor_name.lock().unwrap() = Some(name);
        }
        let mut s = settings::load_settings_from_disk();
        s.horizontal_position = "custom".to_string();
        s.x_offset = center_rel;
        s.top_margin = y_rel;
        let _ = settings::save_settings_to_disk(&s);
        let _ = window.emit("settings-updated", &s);
    }

    let saved_name = state.current_monitor_name.lock().unwrap().clone();
    let monitor = resolve_monitor(&window, saved_name.as_deref())
        .ok_or_else(|| "No monitor detected".to_string())?;

    let scale_factor = monitor.scale_factor();

    let target_physical_width = (FIXED_WINDOW_WIDTH * scale_factor).round() as u32;
    let target_physical_height = (FIXED_WINDOW_HEIGHT * scale_factor).round() as u32;

    let (target_x, target_y) = compute_target_coordinates(&monitor, target_physical_width, top_margin);

    *state.last_programmatic_pos.lock().unwrap() = Some((target_x, target_y, std::time::Instant::now()));

    // Ensure the native window has the fixed size and stable position
    if let Ok(cur_size) = window.inner_size() {
        if cur_size.width != target_physical_width || cur_size.height != target_physical_height {
            let _ = window.set_size(Size::Physical(PhysicalSize {
                width: target_physical_width,
                height: target_physical_height,
            }));
        }
    }
    if let Ok(cur_pos) = window.outer_position() {
        if cur_pos.x != target_x || cur_pos.y != target_y {
            let _ = window.set_position(Position::Physical(PhysicalPosition {
                x: target_x,
                y: target_y,
            }));
        }
    }

    #[cfg(target_os = "linux")]
    {
        use cairo::{RectangleInt, Region};
        use gtk::prelude::*;

        if let Ok(gtk_win) = window.gtk_window() {
            // Relax minimum size requisition across container hierarchy so WebKit2 does not clamp height to 200px
            gtk_win.set_size_request(1, 1);
            if let Ok(vbox) = window.default_vbox() {
                vbox.set_size_request(1, 1);
                vbox.foreach(|widget| {
                    widget.set_size_request(1, 1);
                });
            }

            // Apply Input Shape Mask to GdkWindow so clicks outside the active island pill
            // pass straight through to desktop/applications underneath.
            let logical_pill_w = width.min(FIXED_WINDOW_WIDTH);
            let logical_pill_h = height.min(FIXED_WINDOW_HEIGHT);
            let logical_offset_x = ((FIXED_WINDOW_WIDTH - logical_pill_w) / 2.0).max(0.0);

            let phys_pill_x = (logical_offset_x * scale_factor).round() as i32;
            let phys_pill_y = 0i32;
            let phys_pill_w = (logical_pill_w * scale_factor).round() as i32;
            let phys_pill_h = (logical_pill_h * scale_factor).round() as i32;

            if let Some(gdk_win) = gtk_win.window() {
                let rect = RectangleInt::new(phys_pill_x, phys_pill_y, phys_pill_w, phys_pill_h);
                let region = Region::create_rectangle(&rect);
                gdk_win.input_shape_combine_region(&region, 0, 0);
            }
        }
    }

    Ok(())
}

#[tauri::command]
fn move_island_to_monitor_by_name(
    window: WebviewWindow,
    state: State<'_, AppState>,
    target_name: String,
    width: f64,
    height: f64,
    top_margin: f64,
) -> Result<String, String> {
    let monitors = window.available_monitors().map_err(|e| e.to_string())?;
    let monitor = monitors
        .into_iter()
        .find(|m| m.name().map_or(false, |n| n == &target_name))
        .ok_or_else(|| format!("Monitor '{}' not found", target_name))?;

    let monitor_name = monitor.name().cloned().unwrap_or_else(|| target_name.clone());

    // Save this as the active target monitor
    *state.current_monitor_name.lock().unwrap() = Some(monitor_name.clone());

    resize_and_center_island(window, state, width, height, top_margin)?;

    Ok(monitor_name)
}

#[tauri::command]
fn cycle_next_monitor(
    window: WebviewWindow,
    state: State<'_, AppState>,
    width: f64,
    height: f64,
    top_margin: f64,
) -> Result<String, String> {
    let monitors = window.available_monitors().map_err(|e| e.to_string())?;
    if monitors.is_empty() {
        return Err("No monitors found".to_string());
    }

    let current_name = state.current_monitor_name.lock().unwrap().clone().or_else(|| {
        window
            .current_monitor()
            .ok()
            .flatten()
            .and_then(|m| m.name().cloned())
    });

    let current_idx = monitors
        .iter()
        .position(|m| {
            m.name()
                .map_or(false, |n| current_name.as_ref().map_or(false, |c| c == n))
        })
        .unwrap_or(0);

    let next_idx = (current_idx + 1) % monitors.len();
    let next_monitor = &monitors[next_idx];
    let next_name = next_monitor
        .name()
        .cloned()
        .unwrap_or_else(|| format!("Monitor-{}", next_idx));

    move_island_to_monitor_by_name(window, state, next_name, width, height, top_margin)
}

#[tauri::command]
fn start_window_drag(window: WebviewWindow, state: State<'_, AppState>) -> Result<(), String> {
    state.is_user_dragging.store(true, Ordering::SeqCst);
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
fn cycle_horizontal_position(
    window: WebviewWindow,
    state: State<'_, AppState>,
    width: f64,
    height: f64,
) -> Result<String, String> {
    let mut settings = settings::load_settings_from_disk();
    let next_pos = match settings.horizontal_position.to_lowercase().as_str() {
        "center" => "right",
        "right" => "left",
        _ => "center",
    };
    settings.horizontal_position = next_pos.to_string();
    settings.x_offset = 0.0;
    settings::save_settings_to_disk(&settings)?;
    resize_and_center_island(window.clone(), state, width, height, settings.top_margin)?;
    let _ = window.emit("settings-updated", &settings);
    Ok(next_pos.to_string())
}

#[tauri::command]
fn set_horizontal_position(
    window: WebviewWindow,
    state: State<'_, AppState>,
    position: String,
    x_offset: Option<f64>,
    width: f64,
    height: f64,
) -> Result<String, String> {
    let mut settings = settings::load_settings_from_disk();
    settings.horizontal_position = position.clone();
    if let Some(x) = x_offset {
        settings.x_offset = x;
    }
    settings::save_settings_to_disk(&settings)?;
    resize_and_center_island(window.clone(), state, width, height, settings.top_margin)?;
    let _ = window.emit("settings-updated", &settings);
    Ok(position)
}

#[tauri::command]
fn save_custom_window_position(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.is_user_dragging.store(false, Ordering::SeqCst);
    if let Ok(pos) = window.outer_position() {
        let win_w = window.inner_size().map(|s| s.width).unwrap_or(800);
        if let Some(monitor) = find_monitor_for_webview(&window, &pos, win_w) {
            let mon_name = monitor.name().cloned();
            if let Some(ref name) = mon_name {
                *state.current_monitor_name.lock().unwrap() = Some(name.clone());
            }
            let scale_factor = monitor.scale_factor();
            let monitor_pos = monitor.position();
            let center_phys = (pos.x - monitor_pos.x) + (win_w as i32) / 2;
            let center_rel = center_phys as f64 / scale_factor;
            let y_rel = ((pos.y - monitor_pos.y) as f64 / scale_factor).max(0.0);

            let mut settings = settings::load_settings_from_disk();
            settings.horizontal_position = "custom".to_string();
            settings.x_offset = center_rel;
            settings.top_margin = y_rel;
            settings::save_settings_to_disk(&settings)?;
            *state.last_programmatic_pos.lock().unwrap() = Some((pos.x, pos.y, std::time::Instant::now()));
            state.pending_user_pos.lock().unwrap().take();
            let _ = window.emit("settings-updated", &settings);
        }
    }
    Ok(())
}

#[tauri::command]
fn get_available_monitors(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<Vec<MonitorInfo>, String> {
    let saved_name = state.current_monitor_name.lock().unwrap().clone();
    let current_name = saved_name.or_else(|| {
        window
            .current_monitor()
            .ok()
            .flatten()
            .and_then(|m| m.name().cloned())
    });

    let monitors = window.available_monitors().map_err(|e| e.to_string())?;
    let mut list = Vec::new();

    for m in monitors {
        let name = m.name().cloned().unwrap_or_else(|| "Monitor".to_string());
        let is_current = current_name.as_ref().map_or(false, |c| c == &name);
        let size = m.size();
        list.push(MonitorInfo {
            name,
            is_current,
            width: size.width,
            height: size.height,
            scale_factor: m.scale_factor(),
        });
    }

    Ok(list)
}

#[tauri::command]
fn get_system_telemetry(state: State<'_, AppState>) -> system::SystemTelemetry {
    let mut tracker = state.system_tracker.lock().unwrap();
    tracker.sample()
}

#[tauri::command]
fn get_media_status() -> Option<media::MediaMetadata> {
    media::get_active_player_metadata()
}

#[tauri::command]
fn media_play_pause() -> Result<(), String> {
    media::play_pause()
}

#[tauri::command]
fn media_next() -> Result<(), String> {
    media::next()
}

#[tauri::command]
fn media_previous() -> Result<(), String> {
    media::previous()
}

#[tauri::command]
fn trigger_notification(
    app: tauri::AppHandle,
    title: String,
    message: String,
    app_name: Option<String>,
    icon: Option<String>,
    duration_ms: Option<u64>,
) -> Result<(), String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let notif = IslandNotification {
        id: format!("notif-{}", timestamp),
        title,
        message,
        app_name,
        icon,
        duration_ms,
    };

    app.emit("island-notification", &notif)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn get_dev_status(state: State<'_, AppState>) -> dev_ipc::DevStatusPayload {
    state.dev_manager.get_payload()
}

#[tauri::command]
fn set_agent_status(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    status: dev_ipc::AgentStatus,
) -> Result<(), String> {
    let settings = settings::load_settings_from_disk();
    if !settings.show_agent_in_pill {
        return Ok(());
    }
    *state.dev_manager.agent.lock().unwrap() = status.clone();
    let _ = app.emit("agent-status-change", &status);
    dev_ipc::write_state_to_disk(&state.dev_manager);
    Ok(())
}

#[tauri::command]
fn set_build_status(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    status: dev_ipc::BuildStatus,
) -> Result<(), String> {
    *state.dev_manager.build.lock().unwrap() = status.clone();
    let _ = app.emit("build-status-change", &status);
    dev_ipc::write_state_to_disk(&state.dev_manager);
    Ok(())
}

#[tauri::command]
fn get_settings() -> settings::IslandSettings {
    settings::load_settings_from_disk()
}

#[tauri::command]
fn save_settings(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    settings: settings::IslandSettings,
) -> Result<(), String> {
    if !settings.show_agent_in_pill {
        let mut agent = state.dev_manager.agent.lock().unwrap();
        *agent = dev_ipc::AgentStatus::default();
        drop(agent);

        let (lock, cvar) = &*state.dev_manager.review_pending;
        let mut val = lock.lock().unwrap();
        if val.is_none() {
            *val = Some(true);
            cvar.notify_all();
        }

        let _ = app.emit("agent-status-change", &dev_ipc::AgentStatus::default());
        dev_ipc::write_state_to_disk(&state.dev_manager);
    }
    settings::save_settings_to_disk(&settings)?;
    // Re-register shortcuts in case they changed
    apply_global_shortcuts(&app);
    Ok(())
}

#[tauri::command]
fn submit_review_decision(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    decision: bool,
) -> Result<(), String> {
    state.dev_manager.resolve_review(Some(&app), decision);
    Ok(())
}

#[tauri::command]
fn get_git_status(state: State<'_, AppState>, path: Option<String>) -> git::GitStatus {
    let editor_lock = state.dev_manager.editor.lock().unwrap();
    let editor_ws = editor_lock.as_ref().and_then(|e| {
        dev_ipc::resolve_workspace_dir(
            e.workspace_path.as_deref(),
            e.workspace.as_deref(),
            e.file_path.as_deref(),
        )
    });
    drop(editor_lock);

    let target_path = path
        .map(std::path::PathBuf::from)
        .or_else(|| settings::load_settings_from_disk().watched_repo_path.map(std::path::PathBuf::from))
        .or(editor_ws);

    git::get_git_status(target_path.as_deref())
}

#[tauri::command]
fn refresh_git_status(app: tauri::AppHandle, state: State<'_, AppState>, path: Option<String>) -> Result<git::GitStatus, String> {
    let editor_lock = state.dev_manager.editor.lock().unwrap();
    let editor_ws = editor_lock.as_ref().and_then(|e| {
        dev_ipc::resolve_workspace_dir(
            e.workspace_path.as_deref(),
            e.workspace.as_deref(),
            e.file_path.as_deref(),
        )
    });
    drop(editor_lock);

    let target_path = path
        .map(std::path::PathBuf::from)
        .or_else(|| settings::load_settings_from_disk().watched_repo_path.map(std::path::PathBuf::from))
        .or(editor_ws);

    let status = git::get_git_status(target_path.as_deref());
    let _ = app.emit("git-status-change", &status);
    Ok(status)
}

#[tauri::command]
async fn get_listening_ports() -> Vec<dev_hub::PortInfo> {
    tauri::async_runtime::spawn_blocking(dev_hub::get_listening_ports)
        .await
        .unwrap_or_default()
}

#[tauri::command]
async fn kill_port_process(pid: u32) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || dev_hub::kill_process_by_pid(pid))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_docker_containers() -> Vec<dev_hub::DockerContainerInfo> {
    tauri::async_runtime::spawn_blocking(dev_hub::get_docker_containers)
        .await
        .unwrap_or_default()
}

#[tauri::command]
async fn docker_restart_container(id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || dev_hub::docker_restart(&id))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn docker_stop_container(id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || dev_hub::docker_stop(&id))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn docker_start_container(id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || dev_hub::docker_start(&id))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn open_url_in_browser(url: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || dev_hub::open_browser_url(&url))
        .await
        .map_err(|e| e.to_string())?
}

// Hardware & Audio commands
#[tauri::command]
async fn get_audio_status() -> Result<hardware::AudioStatus, String> {
    tauri::async_runtime::spawn_blocking(hardware::get_audio_status)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn toggle_mic_mute() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(hardware::toggle_mic_mute)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn set_mic_volume(volume: u32) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || hardware::set_mic_volume(volume))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn set_output_volume(volume: u32) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || hardware::set_output_volume(volume))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn toggle_output_mute() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(hardware::toggle_output_mute)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn set_audio_output(target_id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || hardware::set_audio_output(&target_id))
        .await
        .map_err(|e| e.to_string())?
}

// Clipboard & Snippets commands
#[tauri::command]
fn get_clipboard_state(state: State<'_, AppState>) -> Result<clipboard::ClipboardData, String> {
    Ok(state.clipboard_manager.get_state())
}

#[tauri::command]
fn get_clipboard_data(state: State<'_, AppState>) -> Result<clipboard::ClipboardData, String> {
    Ok(state.clipboard_manager.get_state())
}

#[tauri::command]
async fn copy_to_clipboard(
    state: State<'_, AppState>,
    text: Option<String>,
    content: Option<String>,
) -> Result<(), String> {
    let to_copy = text.or(content).unwrap_or_default();
    let mgr = state.clipboard_manager.clone();
    tauri::async_runtime::spawn_blocking(move || mgr.copy_text(&to_copy))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn add_snippet(
    state: State<'_, AppState>,
    title: String,
    content: String,
    category: String,
) -> Result<(), String> {
    state.clipboard_manager.add_snippet(title, content, category)
}

#[tauri::command]
fn add_pinned_snippet(
    state: State<'_, AppState>,
    title: String,
    content: String,
    category: String,
) -> Result<(), String> {
    state.clipboard_manager.add_snippet(title, content, category)
}

#[tauri::command]
fn delete_snippet(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.clipboard_manager.delete_snippet(&id)
}

#[tauri::command]
fn delete_pinned_snippet(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.clipboard_manager.delete_snippet(&id)
}

#[tauri::command]
fn delete_clipboard_item(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.clipboard_manager.delete_history_item(&id)
}

#[tauri::command]
fn clear_clipboard_history(state: State<'_, AppState>) -> Result<(), String> {
    state.clipboard_manager.clear_history()
}

// Drop Shelf commands
#[tauri::command]
fn get_shelf_items(state: State<'_, AppState>) -> Result<Vec<drop_shelf::ShelfItem>, String> {
    Ok(state.shelf_manager.get_items())
}

#[tauri::command]
async fn add_shelf_paths(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<Vec<drop_shelf::ShelfItem>, String> {
    let mgr = state.shelf_manager.clone();
    tauri::async_runtime::spawn_blocking(move || mgr.add_paths(paths))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn add_shelf_text(
    state: State<'_, AppState>,
    text: String,
    title: Option<String>,
) -> Result<drop_shelf::ShelfItem, String> {
    state.shelf_manager.add_text_or_url(text, title)
}

#[tauri::command]
fn remove_shelf_item(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.shelf_manager.remove_item(&id)
}

#[tauri::command]
fn clear_shelf(state: State<'_, AppState>) -> Result<(), String> {
    state.shelf_manager.clear()
}

#[tauri::command]
async fn open_shelf_item_in_file_manager(
    state: State<'_, AppState>,
    path: String,
) -> Result<(), String> {
    let mgr = state.shelf_manager.clone();
    tauri::async_runtime::spawn_blocking(move || mgr.open_in_file_manager(&path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn copy_shelf_file_to_file_manager(path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        // Set x-special/gnome-copied-files: "copy\nfile://{path}" for Nemo/GNOME paste
        let payload = format!("copy\nfile://{}", path);
        let _ = std::process::Command::new("xclip")
            .args(["-selection", "clipboard", "-t", "x-special/gnome-copied-files"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(payload.as_bytes());
                }
                child.wait()
            });

        // Also set text/uri-list
        let uri_payload = format!("file://{}\r\n", path);
        let _ = std::process::Command::new("xclip")
            .args(["-selection", "clipboard", "-t", "text/uri-list"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(uri_payload.as_bytes());
                }
                child.wait()
            });

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Register (or re-register) all global keyboard shortcuts from settings.
/// Existing shortcuts are unregistered first to handle remaps cleanly.
fn apply_global_shortcuts(app: &tauri::AppHandle) {
    let settings = settings::load_settings_from_disk();
    let manager = app.global_shortcut();

    // Unregister all previously registered shortcuts before re-applying
    let _ = manager.unregister_all();

    for (panel, hotkey) in &settings.shortcuts {
        if hotkey.is_empty() {
            continue;
        }
        let panel_clone = panel.clone();
        let app_clone = app.clone();
        match hotkey.parse::<Shortcut>() {
            Ok(shortcut) => {
                let result = manager.on_shortcut(shortcut, move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let _ = app_clone.emit("shortcut-open-panel", &panel_clone);
                    }
                });
                if let Err(e) = result {
                    eprintln!("[dynamic-island] Failed to register shortcut {hotkey} for {panel}: {e}");
                }
            }
            Err(e) => {
                eprintln!("[dynamic-island] Invalid shortcut format '{hotkey}' for '{panel}': {e}");
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    unsafe {
        // Disable WebKitGTK DMABUF renderer to eliminate transparency noise artifacts on Linux X11
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        // Disable GStreamer DMABUF sink to align with the disabled DMABUF renderer;
        // prevents green video screens and buffer allocation deadlocks
        std::env::set_var("WEBKIT_GST_DMABUF_SINK_DISABLED", "1");
        std::env::set_var("NO_AT_BRIDGE", "1");
        std::env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");
    }

    let dev_manager = Arc::new(dev_ipc::DevManager::new());
    let clipboard_manager = clipboard::ClipboardManager::new();
    let shelf_manager = drop_shelf::ShelfManager::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_drag::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState {
            current_monitor_name: Mutex::new(None),
            system_tracker: Mutex::new(system::SystemTracker::new()),
            dev_manager: dev_manager.clone(),
            clipboard_manager,
            shelf_manager,
            last_programmatic_pos: Mutex::new(None),
            pending_user_pos: Mutex::new(None),
            is_user_dragging: AtomicBool::new(false),
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::Moved(pos) => {
                    if let Some(state) = window.try_state::<AppState>() {
                        let is_prog = {
                            let mut prog = state.last_programmatic_pos.lock().unwrap();
                            if let Some((px, py, inst)) = *prog {
                                if (pos.x - px).abs() <= 2 && (pos.y - py).abs() <= 2 {
                                    if inst.elapsed() >= std::time::Duration::from_millis(100) {
                                        *prog = None;
                                    }
                                    true
                                } else if inst.elapsed() < std::time::Duration::from_millis(300) {
                                    true
                                } else {
                                    *prog = None;
                                    false
                                }
                            } else {
                                false
                            }
                        };

                        if !is_prog {
                            let win_w = window.inner_size().map(|s| s.width).unwrap_or(800);
                            if let Some(monitor) = find_monitor_for_win(window, &pos, win_w) {
                                let mon_name = monitor.name().cloned();
                                if let Some(ref name) = mon_name {
                                    *state.current_monitor_name.lock().unwrap() = Some(name.clone());
                                }
                                let scale_factor = monitor.scale_factor();
                                let monitor_pos = monitor.position();
                                let center_phys = (pos.x - monitor_pos.x) + (win_w as i32) / 2;
                                let center_rel = center_phys as f64 / scale_factor;
                                let y_rel = ((pos.y - monitor_pos.y) as f64 / scale_factor).max(0.0);

                                *state.pending_user_pos.lock().unwrap() =
                                    Some((center_rel, y_rel, mon_name, std::time::Instant::now()));
                            }
                        }
                    }
                }
                tauri::WindowEvent::Focused(focused) => {
                    if !focused {
                        if let Some(state) = window.try_state::<AppState>() {
                            state.is_user_dragging.store(false, Ordering::SeqCst);
                            let pending = state.pending_user_pos.lock().unwrap().take();
                            if let Some((center_rel, y_rel, mon_name, _)) = pending {
                                if let Some(name) = mon_name {
                                    *state.current_monitor_name.lock().unwrap() = Some(name);
                                }
                                let mut settings = settings::load_settings_from_disk();
                                settings.horizontal_position = "custom".to_string();
                                settings.x_offset = center_rel;
                                settings.top_margin = y_rel;
                                let _ = settings::save_settings_to_disk(&settings);
                                let _ = window.emit("settings-updated", &settings);
                            }
                        }
                    }
                }
                _ => {}
            }
        })
        .setup(move |app| {
            let state = app.state::<AppState>();
            state.clipboard_manager.set_app_handle(app.handle().clone());

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Start Developer IPC Unix Domain Socket server
            dev_ipc::start_ipc_server(app.handle().clone(), dev_manager);

            // Setup Linux System Tray
            if let Err(e) = tray::setup_tray(&app.handle()) {
                eprintln!("[dynamic-island] Note: Tray setup failed: {e}");
            }

            // Register global keyboard shortcuts
            apply_global_shortcuts(&app.handle());

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_always_on_top(true);
                let _ = window.set_skip_taskbar(true);

                // Initial position on primary/first monitor (compact pill)
                let state = app.state::<AppState>();
                let init_settings = settings::load_settings_from_disk();
                let _ = resize_and_center_island(window, state, 340.0, 52.0, init_settings.top_margin);
            }

            // Background worker: Fast debounce commit for manual window dragging (Alt+drag or native drag)
            let drag_handle = app.handle().clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(50));

                    if let Some(state) = drag_handle.try_state::<AppState>() {
                        let should_commit = {
                            let lock = state.pending_user_pos.lock().unwrap();
                            if let Some((_, _, _, inst)) = *lock {
                                inst.elapsed() >= std::time::Duration::from_millis(150)
                            } else {
                                false
                            }
                        };

                        if should_commit {
                            state.is_user_dragging.store(false, Ordering::SeqCst);
                            if let Some((x_rel, y_rel, mon_name, _)) = state.pending_user_pos.lock().unwrap().take() {
                                if let Some(name) = mon_name {
                                    *state.current_monitor_name.lock().unwrap() = Some(name);
                                }
                                let mut s = settings::load_settings_from_disk();
                                s.horizontal_position = "custom".to_string();
                                s.x_offset = x_rel;
                                s.top_margin = y_rel;
                                let _ = settings::save_settings_to_disk(&s);
                                let _ = drag_handle.emit("settings-updated", &s);
                            }
                        }
                    }
                }
            });

            // Background worker: PipeWire / PulseAudio Real-Time Equalizer Spectrum Monitor
            audio_monitor::start_audio_monitor(app.handle().clone(), Arc::new(AtomicBool::new(true)));

            // Background worker: MPRIS media & System Telemetry updates
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut last_media_status: Option<media::MediaMetadata> = None;
                let mut counter = 0u32;

                loop {
                    std::thread::sleep(std::time::Duration::from_millis(800));

                    // 1. MPRIS check (every 800ms)
                    let current = media::get_active_player_metadata();
                    if current != last_media_status {
                        let _ = handle.emit("media-status-change", &current);
                        last_media_status = current;
                    }

                    // 2. Audio status check (every ~1.6s = 2 iterations)
                    counter += 1;
                    if counter % 2 == 0 {
                        if let Ok(audio_status) = hardware::get_audio_status() {
                            let _ = handle.emit("audio-status-change", &audio_status);
                        }
                    }

                    // 3. System Telemetry (every ~2.4s = 3 iterations)
                    if counter % 3 == 0 {
                        if let Some(state) = handle.try_state::<AppState>() {
                            if let Ok(mut tracker) = state.system_tracker.lock() {
                                let telemetry = tracker.sample();
                                let _ = handle.emit("system-telemetry", &telemetry);
                            }
                        }
                    }

                    // 4. Git Status check (every ~3.2s = 4 iterations)
                    if counter % 4 == 0 {
                        if let Some(state) = handle.try_state::<AppState>() {
                            let settings = settings::load_settings_from_disk();
                            let editor_lock = state.dev_manager.editor.lock().unwrap();
                            let editor_ws = editor_lock.as_ref().and_then(|e| {
                                dev_ipc::resolve_workspace_dir(
                                    e.workspace_path.as_deref(),
                                    e.workspace.as_deref(),
                                    e.file_path.as_deref(),
                                )
                            });
                            drop(editor_lock);

                            let target_path = settings.watched_repo_path.map(std::path::PathBuf::from).or(editor_ws);
                            let git_status = git::get_git_status(target_path.as_deref());
                            let _ = handle.emit("git-status-change", &git_status);
                        }
                    }

                    // 5. Weather check (initial on counter=1, then every ~15 minutes = 1125 iterations)
                    if counter == 1 || counter % 1125 == 0 {
                        let city = settings::load_settings_from_disk().weather_city;
                        if let Ok(w) = weather::get_weather(city) {
                            let _ = handle.emit("weather-update", &w);
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            resize_and_center_island,
            get_available_monitors,
            move_island_to_monitor_by_name,
            cycle_next_monitor,
            start_window_drag,
            cycle_horizontal_position,
            set_horizontal_position,
            save_custom_window_position,
            get_system_telemetry,
            get_media_status,
            media_play_pause,
            media_next,
            media_previous,
            trigger_notification,
            get_dev_status,
            set_agent_status,
            set_build_status,
            get_settings,
            save_settings,
            submit_review_decision,
            get_git_status,
            refresh_git_status,
            get_listening_ports,
            kill_port_process,
            get_docker_containers,
            docker_restart_container,
            docker_stop_container,
            docker_start_container,
            open_url_in_browser,
            get_audio_status,
            toggle_mic_mute,
            set_mic_volume,
            set_output_volume,
            toggle_output_mute,
            set_audio_output,
            get_clipboard_state,
            get_clipboard_data,
            copy_to_clipboard,
            add_pinned_snippet,
            add_snippet,
            delete_pinned_snippet,
            delete_snippet,
            delete_clipboard_item,
            clear_clipboard_history,
            get_shelf_items,
            add_shelf_paths,
            add_shelf_text,
            remove_shelf_item,
            clear_shelf,
            open_shelf_item_in_file_manager,
            weather::fetch_weather,
            copy_shelf_file_to_file_manager,
        ])
        .run(tauri::generate_context!())
        .expect("error while running dynamic-island application");
}
