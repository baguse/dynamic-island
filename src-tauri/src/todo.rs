use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TodoPriority {
    Low,
    Medium,
    High,
}

impl Default for TodoPriority {
    fn default() -> Self {
        TodoPriority::Medium
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub completed: bool,
    #[serde(default)]
    pub priority: TodoPriority,
    pub due_date: Option<String>, // e.g. "2026-10-31" or "Today"
    pub created_at: u64,
    pub completed_at: Option<u64>,
}

pub struct TodoManager {
    pub items: Mutex<Vec<TodoItem>>,
    config_dir: PathBuf,
}

impl TodoManager {
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
        self.config_dir.join("todos.json")
    }

    fn default_markdown_path(&self) -> Option<PathBuf> {
        std::env::var("HOME").ok().map(|h| PathBuf::from(h).join("todo.md"))
    }

    fn load_items(config_dir: &PathBuf) -> Vec<TodoItem> {
        let p = config_dir.join("todos.json");
        if p.exists() {
            if let Ok(bytes) = fs::read(&p) {
                if let Ok(loaded) = serde_json::from_slice::<Vec<TodoItem>>(&bytes) {
                    return loaded;
                }
            }
        }
        Vec::new()
    }

    fn save_items_internal(&self, items: &[TodoItem]) {
        let p = self.file_path();
        if let Ok(json) = serde_json::to_string_pretty(items) {
            let _ = fs::write(p, json);
        }
    }

    pub fn get_todos(&self) -> Vec<TodoItem> {
        let guard = self.items.lock().unwrap();
        guard.clone()
    }

    pub fn add_todo(
        &self,
        title: String,
        priority: Option<TodoPriority>,
        due_date: Option<String>,
    ) -> Result<TodoItem, String> {
        let clean_title = title.trim();
        if clean_title.is_empty() {
            return Err("Task title cannot be empty".to_string());
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let id = format!("todo-{}", now * 1000 + (rand_u32() % 1000) as u64);

        let item = TodoItem {
            id,
            title: clean_title.to_string(),
            completed: false,
            priority: priority.unwrap_or(TodoPriority::Medium),
            due_date,
            created_at: now,
            completed_at: None,
        };

        let mut guard = self.items.lock().unwrap();
        guard.insert(0, item.clone());
        self.save_items_internal(&guard);

        Ok(item)
    }

    pub fn toggle_todo(&self, id: &str) -> Result<TodoItem, String> {
        let mut guard = self.items.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for item in guard.iter_mut() {
            if item.id == id {
                item.completed = !item.completed;
                item.completed_at = if item.completed { Some(now) } else { None };
                let updated = item.clone();
                self.save_items_internal(&guard);
                return Ok(updated);
            }
        }

        Err(format!("Todo item '{id}' not found"))
    }

    pub fn update_todo(
        &self,
        id: &str,
        title: Option<String>,
        priority: Option<TodoPriority>,
        due_date: Option<Option<String>>,
    ) -> Result<TodoItem, String> {
        let mut guard = self.items.lock().unwrap();

        for item in guard.iter_mut() {
            if item.id == id {
                if let Some(t) = title {
                    let clean = t.trim();
                    if !clean.is_empty() {
                        item.title = clean.to_string();
                    }
                }
                if let Some(p) = priority {
                    item.priority = p;
                }
                if let Some(d) = due_date {
                    item.due_date = d;
                }
                let updated = item.clone();
                self.save_items_internal(&guard);
                return Ok(updated);
            }
        }

        Err(format!("Todo item '{id}' not found"))
    }

    pub fn delete_todo(&self, id: &str) -> Result<bool, String> {
        let mut guard = self.items.lock().unwrap();
        let initial_len = guard.len();
        guard.retain(|item| item.id != id);

        if guard.len() < initial_len {
            self.save_items_internal(&guard);
            Ok(true)
        } else {
            Err(format!("Todo item '{id}' not found"))
        }
    }

    pub fn clear_completed(&self) -> Result<usize, String> {
        let mut guard = self.items.lock().unwrap();
        let initial_len = guard.len();
        guard.retain(|item| !item.completed);
        let removed = initial_len - guard.len();
        self.save_items_internal(&guard);
        Ok(removed)
    }

    pub fn export_to_markdown(&self, custom_path: Option<String>) -> Result<String, String> {
        let path = match custom_path {
            Some(p) if !p.trim().is_empty() => PathBuf::from(p.trim()),
            _ => self
                .default_markdown_path()
                .ok_or_else(|| "Could not determine user HOME directory".to_string())?,
        };

        let guard = self.items.lock().unwrap();
        let mut content = String::from("# Dynamic Island — To-Do List\n\n");

        let pending: Vec<&TodoItem> = guard.iter().filter(|i| !i.completed).collect();
        let completed: Vec<&TodoItem> = guard.iter().filter(|i| i.completed).collect();

        content.push_str("## Pending Tasks\n\n");
        if pending.is_empty() {
            content.push_str("_No pending tasks!_\n\n");
        } else {
            for item in pending {
                let prio_tag = match item.priority {
                    TodoPriority::High => " [HIGH]",
                    TodoPriority::Low => " [LOW]",
                    TodoPriority::Medium => "",
                };
                let due_str = match &item.due_date {
                    Some(d) => format!(" (due: {d})"),
                    None => String::new(),
                };
                content.push_str(&format!("- [ ] {}{}{}\n", item.title, prio_tag, due_str));
            }
            content.push('\n');
        }

        content.push_str("## Completed Tasks\n\n");
        if completed.is_empty() {
            content.push_str("_No completed tasks yet._\n\n");
        } else {
            for item in completed {
                content.push_str(&format!("- [x] {}\n", item.title));
            }
            content.push('\n');
        }

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&path, content).map_err(|e| format!("Failed to write markdown file: {e}"))?;

        Ok(path.to_string_lossy().to_string())
    }

    pub fn sync_from_markdown(&self, custom_path: Option<String>) -> Result<Vec<TodoItem>, String> {
        let path = match custom_path {
            Some(p) if !p.trim().is_empty() => PathBuf::from(p.trim()),
            _ => self
                .default_markdown_path()
                .ok_or_else(|| "Could not determine user HOME directory".to_string())?,
        };

        if !path.exists() {
            return Err(format!("Markdown file does not exist at {}", path.display()));
        }

        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read markdown file: {e}"))?;

        let mut parsed_items = Vec::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("- [ ] ") {
                let rest = trimmed.trim_start_matches("- [ ] ").trim();
                let (title, priority, due) = parse_markdown_line_details(rest);
                if !title.is_empty() {
                    parsed_items.push(TodoItem {
                        id: format!("todo-md-{}", now * 1000 + (rand_u32() % 1000) as u64),
                        title,
                        completed: false,
                        priority,
                        due_date: due,
                        created_at: now,
                        completed_at: None,
                    });
                }
            } else if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
                let rest = trimmed[6..].trim();
                let (title, priority, due) = parse_markdown_line_details(rest);
                if !title.is_empty() {
                    parsed_items.push(TodoItem {
                        id: format!("todo-md-{}", now * 1000 + (rand_u32() % 1000) as u64),
                        title,
                        completed: true,
                        priority,
                        due_date: due,
                        created_at: now,
                        completed_at: Some(now),
                    });
                }
            }
        }

        if parsed_items.is_empty() {
            return Err("No task checkboxes (e.g. '- [ ] task') found in markdown file".to_string());
        }

        let mut guard = self.items.lock().unwrap();
        *guard = parsed_items.clone();
        self.save_items_internal(&guard);

        Ok(parsed_items)
    }
}

fn parse_markdown_line_details(line: &str) -> (String, TodoPriority, Option<String>) {
    let mut title = line.to_string();
    let mut priority = TodoPriority::Medium;
    let mut due = None;

    if title.contains("[HIGH]") {
        priority = TodoPriority::High;
        title = title.replace("[HIGH]", "");
    } else if title.contains("[LOW]") {
        priority = TodoPriority::Low;
        title = title.replace("[LOW]", "");
    }

    if let Some(start_idx) = title.find("(due:") {
        if let Some(end_idx) = title[start_idx..].find(')') {
            let due_raw = &title[start_idx + 5..start_idx + end_idx];
            due = Some(due_raw.trim().to_string());
            title = format!("{}{}", &title[..start_idx], &title[start_idx + end_idx + 1..]);
        }
    }

    (title.trim().to_string(), priority, due)
}

fn rand_u32() -> u32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    nanos ^ (nanos >> 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_line_details() {
        let (title, prio, due) = parse_markdown_line_details("Finish security audit [HIGH] (due: Friday)");
        assert_eq!(title, "Finish security audit");
        assert_eq!(prio, TodoPriority::High);
        assert_eq!(due, Some("Friday".to_string()));

        let (title2, prio2, due2) = parse_markdown_line_details("Buy coffee beans");
        assert_eq!(title2, "Buy coffee beans");
        assert_eq!(prio2, TodoPriority::Medium);
        assert_eq!(due2, None);
    }
}
