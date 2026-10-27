use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShelfItem {
    pub id: String,
    pub name: String,
    pub path: Option<String>,
    pub text_content: Option<String>,
    pub item_type: String, // "image", "code", "archive", "doc", "url", "file", "text"
    pub size_bytes: Option<u64>,
    pub size_formatted: Option<String>,
    pub extension: Option<String>,
    pub timestamp: u64,
}

pub struct ShelfManager {
    pub items: Mutex<Vec<ShelfItem>>,
    config_dir: PathBuf,
}

impl ShelfManager {
    pub fn new() -> Arc<Self> {
        let config_dir = crate::settings::get_config_dir();
        let _ = fs::create_dir_all(&config_dir);
        let items = Self::load_items(&config_dir);

        Arc::new(Self {
            items: Mutex::new(items),
            config_dir,
        })
    }

    fn file_path(&self) -> PathBuf {
        self.config_dir.join("drop_shelf.json")
    }

    fn load_items(config_dir: &PathBuf) -> Vec<ShelfItem> {
        let p = config_dir.join("drop_shelf.json");
        if p.exists() {
            if let Ok(bytes) = fs::read(&p) {
                if let Ok(loaded) = serde_json::from_slice::<Vec<ShelfItem>>(&bytes) {
                    return loaded;
                }
            }
        }
        Vec::new()
    }

    fn save_items(&self) {
        if let Ok(guard) = self.items.lock() {
            if let Ok(json) = serde_json::to_string_pretty(&*guard) {
                let _ = fs::write(self.file_path(), json);
            }
        }
    }

    pub fn get_items(&self) -> Vec<ShelfItem> {
        self.items.lock().unwrap().clone()
    }

    pub fn add_paths(&self, paths: Vec<String>) -> Result<Vec<ShelfItem>, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut added = Vec::new();
        let mut guard = self.items.lock().unwrap();

        for p_str in paths {
            let p = Path::new(&p_str);
            if !p.exists() {
                continue;
            }

            // Check if already in shelf
            if guard.iter().any(|item| item.path.as_deref() == Some(&p_str)) {
                continue;
            }

            let file_name = p
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| p_str.clone());

            let ext = p
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            let item_type = match ext.as_str() {
                "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico" => "image",
                "rs" | "ts" | "js" | "svelte" | "py" | "json" | "toml" | "html" | "css" | "sh" | "c" | "cpp" | "go" => "code",
                "zip" | "tar" | "gz" | "7z" | "bz2" | "xz" | "deb" => "archive",
                "pdf" | "md" | "txt" | "doc" | "docx" | "rtf" => "doc",
                _ => "file",
            }
            .to_string();

            let (size_bytes, size_formatted) = if let Ok(meta) = fs::metadata(p) {
                let bytes = meta.len();
                let fmt = if bytes < 1024 {
                    format!("{bytes} B")
                } else if bytes < 1024 * 1024 {
                    format!("{:.1} KB", bytes as f64 / 1024.0)
                } else if bytes < 1024 * 1024 * 1024 {
                    format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
                } else {
                    format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
                };
                (Some(bytes), Some(fmt))
            } else {
                (None, None)
            };

            let item = ShelfItem {
                id: format!("shelf-{}-{}", now, guard.len()),
                name: file_name,
                path: Some(p_str),
                text_content: None,
                item_type,
                size_bytes,
                size_formatted,
                extension: if ext.is_empty() { None } else { Some(ext) },
                timestamp: now,
            };

            added.push(item.clone());
            guard.insert(0, item);
        }

        drop(guard);
        self.save_items();
        Ok(added)
    }

    pub fn add_text_or_url(&self, content: String, title: Option<String>) -> Result<ShelfItem, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let trimmed = content.trim();
        let is_url = trimmed.starts_with("http://") || trimmed.starts_with("https://");
        let byte_len = trimmed.len() as u64;
        let char_count = trimmed.chars().count();

        let name = if let Some(t) = title {
            t
        } else if is_url {
            trimmed.to_string()
        } else {
            let first_line = trimmed.lines().next().unwrap_or("Snippet");
            if first_line.len() > 30 {
                format!("{}...", &first_line[..27])
            } else {
                first_line.to_string()
            }
        };

        let item = ShelfItem {
            id: format!("shelf-{}-{}", now, rand_id()),
            name,
            path: None,
            text_content: Some(content),
            item_type: if is_url { "url".to_string() } else { "text".to_string() },
            size_bytes: Some(byte_len),
            size_formatted: Some(format!("{} chars", char_count)),
            extension: None,
            timestamp: now,
        };

        {
            let mut guard = self.items.lock().unwrap();
            guard.insert(0, item.clone());
        }
        self.save_items();
        Ok(item)
    }

    pub fn remove_item(&self, id: &str) -> Result<(), String> {
        {
            let mut guard = self.items.lock().unwrap();
            guard.retain(|i| i.id != id);
        }
        self.save_items();
        Ok(())
    }

    pub fn clear(&self) -> Result<(), String> {
        {
            let mut guard = self.items.lock().unwrap();
            guard.clear();
        }
        self.save_items();
        Ok(())
    }

    pub fn open_in_file_manager(&self, path_str: &str) -> Result<(), String> {
        let p = Path::new(path_str);
        if !p.exists() {
            return Err("File path does not exist".to_string());
        }

        // Try nemo --select first (Cinnamon desktop default)
        if Command::new("nemo")
            .arg(path_str)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }

        // Fallback to xdg-open
        let dir = if p.is_dir() {
            p
        } else {
            p.parent().unwrap_or(p)
        };

        Command::new("xdg-open")
            .arg(dir)
            .spawn()
            .map_err(|e| format!("Failed to open file manager: {e}"))?;

        Ok(())
    }
}

fn rand_id() -> u32 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    now % 10000
}
