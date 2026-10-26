use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentState {
    Idle,
    Thinking,
    Working,
    Review,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub name: String,
    pub state: AgentState,
    pub message: String,
    pub task: Option<String>,
    pub progress: Option<u8>,
    pub changed_files: Option<Vec<String>>,
    pub is_demo: bool,
}

impl Default for AgentStatus {
    fn default() -> Self {
        Self {
            name: "Antigravity Agent".to_string(),
            state: AgentState::Idle,
            message: "Ready for your next task".to_string(),
            task: None,
            progress: None,
            changed_files: None,
            is_demo: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BuildState {
    Idle,
    Running,
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildStatus {
    pub command: String,
    pub state: BuildState,
    pub duration_ms: Option<u64>,
    pub message: Option<String>,
    pub exit_code: Option<i32>,
    pub started_at: Option<u64>,
}

impl Default for BuildStatus {
    fn default() -> Self {
        Self {
            command: String::new(),
            state: BuildState::Idle,
            duration_ms: None,
            message: None,
            exit_code: None,
            started_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EditorPresence {
    pub editor: String,
    pub workspace: Option<String>,
    pub workspace_path: Option<String>,
    pub file_name: Option<String>,
    pub file_path: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub language: Option<String>,
    pub state: String,
    pub git_branch: Option<String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevStatusPayload {
    pub agent: AgentStatus,
    pub build: BuildStatus,
    #[serde(default)]
    pub editor: Option<EditorPresence>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "target", rename_all = "lowercase")]
pub enum IpcMessage {
    Agent {
        name: Option<String>,
        state: AgentState,
        message: String,
        task: Option<String>,
        progress: Option<u8>,
        changed_files: Option<Vec<String>>,
        #[serde(default)]
        wait: Option<bool>,
        #[serde(default)]
        timeout_sec: Option<u64>,
    },
    Build {
        command: String,
        state: BuildState,
        duration_ms: Option<u64>,
        message: Option<String>,
        exit_code: Option<i32>,
    },
    Notification {
        title: String,
        message: String,
        app_name: Option<String>,
        icon: Option<String>,
        duration_ms: Option<u64>,
    },
    Toggle,
    Screen,
    Settings,
    Status,
    Decision,
    #[serde(rename = "review_decision")]
    ReviewDecision {
        decision: bool,
    },
    #[serde(rename = "git_refresh")]
    GitRefresh,
    #[serde(rename = "editor")]
    Editor {
        editor: Option<String>,
        workspace: Option<String>,
        workspace_path: Option<String>,
        file_name: Option<String>,
        file_path: Option<String>,
        line: Option<u32>,
        column: Option<u32>,
        language: Option<String>,
        state: Option<String>,
        git_branch: Option<String>,
    },
}

pub struct DevManager {
    pub agent: Mutex<AgentStatus>,
    pub build: Mutex<BuildStatus>,
    pub editor: Mutex<Option<EditorPresence>>,
    pub review_pending: Arc<(Mutex<Option<bool>>, Condvar)>,
    pub last_decision: Mutex<Option<(bool, std::time::Instant)>>,
}

impl DevManager {
    pub fn new() -> Self {
        Self {
            agent: Mutex::new(AgentStatus::default()),
            build: Mutex::new(BuildStatus::default()),
            editor: Mutex::new(None),
            review_pending: Arc::new((Mutex::new(None), Condvar::new())),
            last_decision: Mutex::new(None),
        }
    }

    pub fn resolve_review(&self, app_handle: Option<&AppHandle>, decision: bool) {
        *self.last_decision.lock().unwrap() = Some((decision, std::time::Instant::now()));
        write_review_to_disk(decision);

        {
            let (lock, cvar) = &*self.review_pending;
            let mut val = lock.lock().unwrap();
            *val = Some(decision);
            cvar.notify_all();
        }

        let mut agent = self.agent.lock().unwrap();
        if decision {
            agent.state = AgentState::Working;
            agent.message = "Changes approved! Proceeding with task...".to_string();
        } else {
            agent.state = AgentState::Idle;
            agent.message = "Changes discarded by user.".to_string();
            agent.changed_files = None;
        }
        let updated_agent = agent.clone();
        drop(agent);

        if let Some(app) = app_handle {
            let _ = app.emit("agent-status-change", &updated_agent);
        }
        write_state_to_disk(self);
    }

    pub fn wait_for_review_decision(&self, timeout: std::time::Duration) -> Option<bool> {
        let (lock, cvar) = &*self.review_pending;
        let mut val = lock.lock().unwrap();

        if let Some(dec) = *val {
            *val = None;
            return Some(dec);
        }

        let start = std::time::Instant::now();
        while val.is_none() {
            let elapsed = start.elapsed();
            if elapsed >= timeout {
                return None;
            }
            let remaining = timeout - elapsed;
            let result = cvar.wait_timeout(val, remaining).unwrap();
            val = result.0;
            if result.1.timed_out() {
                break;
            }
        }
        let res = *val;
        *val = None;
        res
    }

    pub fn get_last_decision(&self) -> Option<bool> {
        let last = self.last_decision.lock().unwrap();
        if let Some((decision, instant)) = *last {
            if instant.elapsed() < std::time::Duration::from_secs(300) {
                return Some(decision);
            }
        }
        if let Ok(content) = fs::read_to_string(get_review_file_path()) {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(dec) = data.get("decision").and_then(|v| v.as_str()) {
                    let ts = data.get("timestamp").and_then(|v| v.as_u64()).unwrap_or(0);
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    if now.saturating_sub(ts) < 300 {
                        return match dec {
                            "approved" => Some(true),
                            "rejected" => Some(false),
                            _ => None,
                        };
                    }
                }
            }
        }
        None
    }

    pub fn get_payload(&self) -> DevStatusPayload {
        DevStatusPayload {
            agent: self.agent.lock().unwrap().clone(),
            build: self.build.lock().unwrap().clone(),
            editor: self.editor.lock().unwrap().clone(),
        }
    }
}

pub fn resolve_workspace_dir(
    workspace_path: Option<&str>,
    workspace_name: Option<&str>,
    file_path: Option<&str>,
) -> Option<PathBuf> {
    // 1. If explicit workspace_path is provided and exists
    if let Some(p) = workspace_path {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }

    // 2. If file_path is an absolute path that exists
    if let Some(fp) = file_path {
        let path = PathBuf::from(fp);
        if path.is_absolute() && path.exists() {
            if path.is_dir() {
                return Some(path);
            } else if let Some(parent) = path.parent() {
                return Some(parent.to_path_buf());
            }
        }
    }

    // 3. Search Cursor, Code, and VSCodium workspaceStorage for matching workspace name
    if let Some(name) = workspace_name {
        if let Ok(home) = std::env::var("HOME") {
            let home_p = PathBuf::from(home);
            for editor_cfg in ["Cursor", "Code", "VSCodium"] {
                let storage_dir = home_p.join(".config").join(editor_cfg).join("User").join("workspaceStorage");
                if let Ok(entries) = fs::read_dir(&storage_dir) {
                    for entry in entries.flatten() {
                        let ws_file = entry.path().join("workspace.json");
                        if ws_file.exists() {
                            if let Ok(content) = fs::read_to_string(&ws_file) {
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                    if let Some(folder_uri) = json.get("folder").and_then(|v| v.as_str()) {
                                        if let Some(stripped) = folder_uri.strip_prefix("file://") {
                                            let folder_path = PathBuf::from(stripped);
                                            if folder_path.file_name().and_then(|n| n.to_str()) == Some(name) && folder_path.exists() {
                                                return Some(folder_path);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

pub fn get_socket_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let p = PathBuf::from(runtime_dir).join("dynamic-island.sock");
        return p;
    }

    unsafe extern "C" {
        fn getuid() -> u32;
    }
    let uid = unsafe { getuid() };
    PathBuf::from(format!("/tmp/dynamic-island-{}.sock", uid))
}

pub fn get_review_file_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("dynamic-island-review.json")
    } else {
        unsafe extern "C" {
            fn getuid() -> u32;
        }
        let uid = unsafe { getuid() };
        PathBuf::from(format!("/tmp/dynamic-island-review-{}.json", uid))
    }
}

pub fn get_state_file_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("dynamic-island-state.json")
    } else {
        unsafe extern "C" {
            fn getuid() -> u32;
        }
        let uid = unsafe { getuid() };
        PathBuf::from(format!("/tmp/dynamic-island-state-{}.json", uid))
    }
}

pub fn write_state_to_disk(dev_manager: &DevManager) {
    let payload = dev_manager.get_payload();
    if let Ok(json) = serde_json::to_string(&payload) {
        let _ = fs::write(get_state_file_path(), json);
    }
}

pub fn write_review_to_disk(decision: bool) {
    let decision_str = if decision { "approved" } else { "rejected" };
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let json = serde_json::json!({
        "decision": decision_str,
        "timestamp": timestamp,
    });
    let _ = fs::write(get_review_file_path(), json.to_string());
}

pub fn start_ipc_server(app_handle: AppHandle, dev_manager: Arc<DevManager>) {
    let socket_path = get_socket_path();

    if socket_path.exists() {
        let _ = fs::remove_file(&socket_path);
    }

    std::thread::spawn(move || {
        let listener = match UnixListener::bind(&socket_path) {
            Ok(l) => {
                log::info!("Dynamic Island IPC listening on {:?}", socket_path);
                l
            }
            Err(e) => {
                log::warn!("Failed to bind Dynamic Island IPC socket {:?}: {}", socket_path, e);
                return;
            }
        };

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let app_h = app_handle.clone();
                    let dev_m = dev_manager.clone();
                    std::thread::spawn(move || {
                        handle_client(stream, &app_h, &dev_m);
                    });
                }
                Err(e) => {
                    log::debug!("IPC client connection error: {}", e);
                }
            }
        }

        let _ = fs::remove_file(&socket_path);
    });
}

fn handle_client(mut stream: UnixStream, app_handle: &AppHandle, dev_manager: &DevManager) {
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(310)));
    let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(10)));
    let reader = BufReader::new(match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    });

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match serde_json::from_str::<IpcMessage>(trimmed) {
            Ok(msg) => {
                let resp = process_ipc_message(msg, app_handle, dev_manager);
                let _ = writeln!(stream, "{}", resp);
                let _ = stream.flush();
            }
            Err(e) => {
                let _ = writeln!(stream, "{{\"ok\":false,\"error\":\"{}\"}}", e);
                let _ = stream.flush();
            }
        }
    }
}

pub fn process_ipc_message(
    msg: IpcMessage,
    app_handle: &AppHandle,
    dev_manager: &DevManager,
) -> String {
    match msg {
        IpcMessage::Agent {
            name,
            state,
            message,
            task,
            progress,
            changed_files,
            wait,
            timeout_sec,
        } => {
            let mut agent = dev_manager.agent.lock().unwrap();
            let agent_name = name.unwrap_or_else(|| agent.name.clone());

            let new_status = AgentStatus {
                name: agent_name,
                state: state.clone(),
                message: message.clone(),
                task,
                progress,
                changed_files,
                is_demo: false,
            };

            *agent = new_status.clone();
            drop(agent);
            let _ = app_handle.emit("agent-status-change", &new_status);
            write_state_to_disk(dev_manager);

            if state == AgentState::Review {
                *dev_manager.last_decision.lock().unwrap() = None;
                *dev_manager.review_pending.0.lock().unwrap() = None;
                let _ = fs::write(
                    get_review_file_path(),
                    serde_json::json!({
                        "decision": "pending",
                        "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
                    }).to_string()
                );

                let notif = serde_json::json!({
                    "id": format!("agent-review-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()),
                    "title": "AI Agent Review",
                    "message": message,
                    "app_name": "AI Assistant",
                    "icon": "terminal",
                    "duration_ms": 6000
                });
                let _ = app_handle.emit("island-notification", &notif);

                if wait.unwrap_or(false) {
                    let duration = std::time::Duration::from_secs(timeout_sec.unwrap_or(300));
                    return match dev_manager.wait_for_review_decision(duration) {
                        Some(true) => r#"{"ok":true,"decision":"approved"}"#.to_string(),
                        Some(false) => r#"{"ok":true,"decision":"rejected"}"#.to_string(),
                        None => r#"{"ok":false,"decision":"timeout","error":"User did not respond within timeout"}"#.to_string(),
                    };
                }
            }

            r#"{"ok":true}"#.to_string()
        }

        IpcMessage::Build {
            command,
            state,
            duration_ms,
            message,
            exit_code,
        } => {
            let mut build = dev_manager.build.lock().unwrap();
            let started_at = if state == BuildState::Running {
                Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                )
            } else {
                build.started_at
            };

            let new_build = BuildStatus {
                command: command.clone(),
                state: state.clone(),
                duration_ms,
                message: message.clone(),
                exit_code,
                started_at,
            };

            *build = new_build.clone();
            let _ = app_handle.emit("build-status-change", &new_build);
            write_state_to_disk(dev_manager);

            if state == BuildState::Success {
                let duration_str = duration_ms
                    .map(|d| format!(" in {:.1}s", d as f64 / 1000.0))
                    .unwrap_or_default();
                let notif = serde_json::json!({
                    "id": format!("build-succ-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()),
                    "title": "Build Succeeded",
                    "message": format!("'{command}' passed{duration_str}"),
                    "app_name": "Build Monitor",
                    "icon": "terminal",
                    "duration_ms": 4000
                });
                let _ = app_handle.emit("island-notification", &notif);
            } else if state == BuildState::Failed {
                let code_str = exit_code
                    .map(|c| format!(" (exit code {})", c))
                    .unwrap_or_default();
                let notif = serde_json::json!({
                    "id": format!("build-fail-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()),
                    "title": "Build Failed",
                    "message": format!("'{command}' failed{code_str}"),
                    "app_name": "Build Monitor",
                    "icon": "system",
                    "duration_ms": 5500
                });
                let _ = app_handle.emit("island-notification", &notif);
            }

            r#"{"ok":true}"#.to_string()
        }

        IpcMessage::Notification {
            title,
            message,
            app_name,
            icon,
            duration_ms,
        } => {
            let notif = serde_json::json!({
                "id": format!("ipc-notif-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()),
                "title": title,
                "message": message,
                "app_name": app_name,
                "icon": icon,
                "duration_ms": duration_ms
            });
            let _ = app_handle.emit("island-notification", &notif);
            r#"{"ok":true}"#.to_string()
        }

        IpcMessage::Toggle => {
            let _ = app_handle.emit("tray-toggle-island", ());
            r#"{"ok":true}"#.to_string()
        }

        IpcMessage::Screen => {
            let _ = app_handle.emit("tray-cycle-monitor", ());
            r#"{"ok":true}"#.to_string()
        }

        IpcMessage::Settings => {
            let _ = app_handle.emit("tray-open-settings", ());
            r#"{"ok":true}"#.to_string()
        }

        IpcMessage::Status => {
            let payload = dev_manager.get_payload();
            serde_json::to_string(&serde_json::json!({
                "ok": true,
                "agent": payload.agent,
                "build": payload.build,
                "editor": payload.editor,
            }))
            .unwrap_or_else(|_| r#"{"ok":false}"#.to_string())
        }

        IpcMessage::Decision => match dev_manager.get_last_decision() {
            Some(true) => r#"{"ok":true,"decision":"approved"}"#.to_string(),
            Some(false) => r#"{"ok":true,"decision":"rejected"}"#.to_string(),
            None => r#"{"ok":true,"decision":"none"}"#.to_string(),
        },

        IpcMessage::ReviewDecision { decision } => {
            dev_manager.resolve_review(Some(app_handle), decision);
            r#"{"ok":true}"#.to_string()
        }

        IpcMessage::GitRefresh => {
            let settings = crate::settings::load_settings_from_disk();
            let editor_lock = dev_manager.editor.lock().unwrap();
            let editor_ws = editor_lock.as_ref().and_then(|e| {
                resolve_workspace_dir(
                    e.workspace_path.as_deref(),
                    e.workspace.as_deref(),
                    e.file_path.as_deref(),
                )
            });
            drop(editor_lock);

            let target_path = settings.watched_repo_path.map(PathBuf::from).or(editor_ws);
            let git_status = crate::git::get_git_status(target_path.as_deref());
            let _ = app_handle.emit("git-status-change", &git_status);
            r#"{"ok":true}"#.to_string()
        }

        IpcMessage::Editor {
            editor,
            workspace,
            workspace_path,
            file_name,
            file_path,
            line,
            column,
            language,
            state,
            git_branch,
        } => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let resolved_ws = resolve_workspace_dir(
                workspace_path.as_deref(),
                workspace.as_deref(),
                file_path.as_deref(),
            );

            let ws_path_str = resolved_ws
                .as_ref()
                .and_then(|p| p.to_str())
                .map(|s| s.to_string())
                .or(workspace_path);

            let presence = EditorPresence {
                editor: editor.unwrap_or_else(|| "Editor".to_string()),
                workspace,
                workspace_path: ws_path_str,
                file_name,
                file_path,
                line,
                column,
                language,
                state: state.unwrap_or_else(|| "editing".to_string()),
                git_branch,
                updated_at: now,
            };

            *dev_manager.editor.lock().unwrap() = Some(presence.clone());
            let _ = app_handle.emit("editor-presence-change", &presence);
            write_state_to_disk(dev_manager);

            // Automatically synchronize Git status with the resolved editor workspace!
            if let Some(target) = resolved_ws {
                let git_status = crate::git::get_git_status(Some(&target));
                let _ = app_handle.emit("git-status-change", &git_status);
            }

            r#"{"ok":true}"#.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_message_deserialization() {
        let agent_json = r#"{"target":"agent","state":"working","message":"Testing Rust IPC","progress":45}"#;
        let msg: IpcMessage = serde_json::from_str(agent_json).expect("deserialize agent");
        if let IpcMessage::Agent { state, message, progress, .. } = msg {
            assert_eq!(state, AgentState::Working);
            assert_eq!(message, "Testing Rust IPC");
            assert_eq!(progress, Some(45));
        } else {
            panic!("Expected IpcMessage::Agent");
        }

        let build_json = r#"{"target":"build","command":"cargo test","state":"success","duration_ms":1200}"#;
        let msg: IpcMessage = serde_json::from_str(build_json).expect("deserialize build");
        if let IpcMessage::Build { command, state, duration_ms, .. } = msg {
            assert_eq!(command, "cargo test");
            assert_eq!(state, BuildState::Success);
            assert_eq!(duration_ms, Some(1200));
        } else {
            panic!("Expected IpcMessage::Build");
        }
    }

    #[test]
    fn test_dev_manager() {
        let dm = DevManager::new();
        let payload = dm.get_payload();
        assert_eq!(payload.agent.state, AgentState::Idle);
        assert_eq!(payload.build.state, BuildState::Idle);
    }

    #[test]
    fn test_review_decision_approval_flow() {
        let dm = Arc::new(DevManager::new());
        let dm_clone = dm.clone();

        let handle = std::thread::spawn(move || {
            dm_clone.wait_for_review_decision(std::time::Duration::from_millis(500))
        });

        std::thread::sleep(std::time::Duration::from_millis(50));
        dm.resolve_review(None, true);

        let result = handle.join().expect("join");
        assert_eq!(result, Some(true));
        assert_eq!(dm.agent.lock().unwrap().state, AgentState::Working);
    }

    #[test]
    fn test_review_decision_rejection_flow() {
        let dm = Arc::new(DevManager::new());
        let dm_clone = dm.clone();

        let handle = std::thread::spawn(move || {
            dm_clone.wait_for_review_decision(std::time::Duration::from_millis(500))
        });

        std::thread::sleep(std::time::Duration::from_millis(50));
        dm.resolve_review(None, false);

        let result = handle.join().expect("join");
        assert_eq!(result, Some(false));
        assert_eq!(dm.agent.lock().unwrap().state, AgentState::Idle);
    }

    #[test]
    fn test_review_decision_cached_before_wait() {
        let dm = DevManager::new();
        dm.resolve_review(None, true);
        let result = dm.wait_for_review_decision(std::time::Duration::from_millis(100));
        assert_eq!(result, Some(true));
        assert_eq!(dm.get_last_decision(), Some(true));
    }

    #[test]
    fn test_ipc_status_message_deserialization() {
        let status_json = r#"{"target":"status"}"#;
        let msg: IpcMessage = serde_json::from_str(status_json).expect("deserialize status");
        assert!(matches!(msg, IpcMessage::Status));

        let decision_json = r#"{"target":"decision"}"#;
        let msg: IpcMessage = serde_json::from_str(decision_json).expect("deserialize decision");
        assert!(matches!(msg, IpcMessage::Decision));
    }

    #[test]
    fn test_review_decision_timeout() {
        let dm = DevManager::new();
        let result = dm.wait_for_review_decision(std::time::Duration::from_millis(50));
        assert_eq!(result, None);
    }
}
