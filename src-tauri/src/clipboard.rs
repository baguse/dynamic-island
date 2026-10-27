use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Emitter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String,
    pub content: String,
    pub preview: String,
    pub timestamp: u64,
    pub char_count: usize,
    pub line_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: String, // "command", "token", "regex", "general"
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClipboardData {
    pub history: Vec<ClipboardItem>,
    pub snippets: Vec<SnippetItem>,
}

pub struct ClipboardManager {
    pub data: Mutex<ClipboardData>,
    config_dir: PathBuf,
    app_handle: Mutex<Option<tauri::AppHandle>>,
}

impl ClipboardManager {
    pub fn new() -> Arc<Self> {
        let config_dir = crate::settings::get_config_dir();
        let _ = fs::create_dir_all(&config_dir);

        let data = Self::load_data(&config_dir);

        let manager = Arc::new(Self {
            data: Mutex::new(data),
            config_dir,
            app_handle: Mutex::new(None),
        });

        // Spawn background polling thread (runs every 750ms for responsive clipboard updates)
        let poll_ref = Arc::clone(&manager);
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(750));
                poll_ref.check_clipboard();
            }
        });

        manager
    }

    pub fn set_app_handle(&self, handle: tauri::AppHandle) {
        if let Ok(mut g) = self.app_handle.lock() {
            *g = Some(handle);
        }
    }

    fn emit_change(&self, data: &ClipboardData) {
        if let Ok(guard) = self.app_handle.lock() {
            if let Some(app) = &*guard {
                let _ = app.emit("clipboard-change", data);
            }
        }
    }

    fn data_path(&self) -> PathBuf {
        self.config_dir.join("clipboard_history.json")
    }

    fn load_data(config_dir: &PathBuf) -> ClipboardData {
        let path = config_dir.join("clipboard_history.json");
        if path.exists() {
            if let Ok(bytes) = fs::read(&path) {
                if let Ok(loaded) = serde_json::from_slice::<ClipboardData>(&bytes) {
                    return loaded;
                }
            }
        }

        // Default seed snippets
        ClipboardData {
            history: Vec::new(),
            snippets: vec![
                SnippetItem {
                    id: "snip-1".to_string(),
                    title: "Git Status Condensed".to_string(),
                    content: "git status -sb".to_string(),
                    category: "command".to_string(),
                },
                SnippetItem {
                    id: "snip-2".to_string(),
                    title: "Docker Prune Clean".to_string(),
                    content: "docker system prune -af --volumes".to_string(),
                    category: "command".to_string(),
                },
                SnippetItem {
                    id: "snip-3".to_string(),
                    title: "UUID v4 Regex".to_string(),
                    content: r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$".to_string(),
                    category: "regex".to_string(),
                },
            ],
        }
    }

    fn save_data(&self) {
        if let Ok(guard) = self.data.lock() {
            if let Ok(json) = serde_json::to_string_pretty(&*guard) {
                let _ = fs::write(self.data_path(), json);
            }
        }
    }

    fn check_clipboard(&self) {
        let current_text = match Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
        {
            Ok(out) => {
                if out.status.success() {
                    String::from_utf8_lossy(&out.stdout).to_string()
                } else {
                    return;
                }
            }
            Err(_) => return,
        };

        let trimmed = current_text.trim();
        if trimmed.is_empty() {
            return;
        }

        let mut guard = match self.data.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        // If identical to most recent entry, do not duplicate
        if let Some(first) = guard.history.first() {
            if first.content == current_text {
                return;
            }
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let preview = if trimmed.len() > 140 {
            format!("{}...", &trimmed[..137])
        } else {
            trimmed.to_string()
        };

        let char_count = current_text.chars().count();
        let line_count = current_text.lines().count().max(1);

        let item = ClipboardItem {
            id: format!("clip-{}", now),
            content: current_text,
            preview,
            timestamp: now,
            char_count,
            line_count,
        };

        guard.history.insert(0, item);
        if guard.history.len() > 30 {
            guard.history.truncate(30);
        }

        self.emit_change(&guard);
        drop(guard);
        self.save_data();
    }

    pub fn get_state(&self) -> ClipboardData {
        self.data.lock().unwrap().clone()
    }

    pub fn copy_text(&self, text: &str) -> Result<(), String> {
        let mut child = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn xclip: {e}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| format!("Failed to write to xclip stdin: {e}"))?;
        }

        let _ = child.wait();
        // Also trigger check so it updates state
        self.check_clipboard();
        Ok(())
    }

    pub fn add_snippet(&self, title: String, content: String, category: String) -> Result<(), String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let item = SnippetItem {
            id: format!("snip-{}", now),
            title,
            content,
            category,
        };

        {
            let mut guard = self.data.lock().unwrap();
            guard.snippets.push(item);
            self.emit_change(&guard);
        }
        self.save_data();
        Ok(())
    }

    pub fn delete_snippet(&self, id: &str) -> Result<(), String> {
        {
            let mut guard = self.data.lock().unwrap();
            guard.snippets.retain(|s| s.id != id);
            self.emit_change(&guard);
        }
        self.save_data();
        Ok(())
    }

    pub fn delete_history_item(&self, id: &str) -> Result<(), String> {
        {
            let mut guard = self.data.lock().unwrap();
            guard.history.retain(|h| h.id != id);
            self.emit_change(&guard);
        }
        self.save_data();
        Ok(())
    }

    pub fn clear_history(&self) -> Result<(), String> {
        {
            let mut guard = self.data.lock().unwrap();
            guard.history.clear();
            self.emit_change(&guard);
        }
        self.save_data();
        Ok(())
    }
}
