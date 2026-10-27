use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn default_theme() -> String {
    "dark".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IslandSettings {
    #[serde(default = "default_theme")]
    pub theme: String,
    pub hover_expand_enabled: bool,
    pub hover_delay_ms: u64,
    pub leave_delay_ms: u64,
    pub inactivity_timeout_s: u64,
    pub top_margin: f64,
    pub show_system_in_pill: bool,
    pub show_battery_alert: bool,
    pub show_media_in_pill: bool,
    pub show_build_in_pill: bool,
    pub show_agent_in_pill: bool,
    #[serde(default = "default_true")]
    pub show_git_in_pill: bool,
    #[serde(default = "default_true")]
    pub show_editor_in_pill: bool,
    #[serde(default = "default_true")]
    pub show_ports_in_pill: bool,
    #[serde(default = "default_true")]
    pub show_docker_in_pill: bool,
    #[serde(default = "default_true")]
    pub show_mic_in_pill: bool,
    #[serde(default = "default_true")]
    pub show_shelf_in_pill: bool,
    #[serde(default = "default_true")]
    pub show_clipboard_in_pill: bool,
    #[serde(default)]
    pub watched_repo_path: Option<String>,
    pub autostart_enabled: bool,
}

impl Default for IslandSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            hover_expand_enabled: true,
            hover_delay_ms: 220,
            leave_delay_ms: 450,
            inactivity_timeout_s: 12,
            top_margin: 12.0,
            show_system_in_pill: true,
            show_battery_alert: true,
            show_media_in_pill: true,
            show_build_in_pill: true,
            show_agent_in_pill: true,
            show_git_in_pill: true,
            show_editor_in_pill: true,
            show_ports_in_pill: true,
            show_docker_in_pill: true,
            show_mic_in_pill: true,
            show_shelf_in_pill: true,
            show_clipboard_in_pill: true,
            watched_repo_path: None,
            autostart_enabled: false,
        }
    }
}

pub fn get_config_dir() -> PathBuf {
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(config_home).join("dynamic-island")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config").join("dynamic-island")
    } else {
        PathBuf::from("/tmp/dynamic-island")
    }
}

fn get_settings_file_path() -> PathBuf {
    get_config_dir().join("settings.json")
}

fn get_autostart_desktop_file_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(
        PathBuf::from(home)
            .join(".config")
            .join("autostart")
            .join("dynamic-island.desktop"),
    )
}

pub fn load_settings_from_disk() -> IslandSettings {
    let path = get_settings_file_path();
    let mut settings = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|data| serde_json::from_str::<IslandSettings>(&data).ok())
            .unwrap_or_default()
    } else {
        IslandSettings::default()
    };

    // Sync autostart status with filesystem
    settings.autostart_enabled = is_autostart_configured();
    settings
}

pub fn save_settings_to_disk(settings: &IslandSettings) -> Result<(), String> {
    let dir = get_config_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }

    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(get_settings_file_path(), json).map_err(|e| e.to_string())?;

    // Apply autostart setting
    set_autostart_configured(settings.autostart_enabled)?;

    Ok(())
}

pub fn is_autostart_configured() -> bool {
    get_autostart_desktop_file_path()
        .map(|p| p.exists())
        .unwrap_or(false)
}

pub fn set_autostart_configured(enabled: bool) -> Result<(), String> {
    let path = match get_autostart_desktop_file_path() {
        Some(p) => p,
        None => return Err("Unable to determine HOME directory".to_string()),
    };

    if enabled {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }

        let exe_path = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "dynamic-island".to_string());

        let desktop_content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Version=1.0\n\
             Name=Dynamic Island\n\
             Comment=Floating Linux Dynamic Island widget\n\
             Exec={}\n\
             Terminal=false\n\
             Categories=Utility;\n\
             StartupNotify=false\n\
             X-GNOME-Autostart-enabled=true\n",
            exe_path
        );

        fs::write(&path, desktop_content).map_err(|e| e.to_string())?;
    } else if path.exists() {
        let _ = fs::remove_file(&path);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_defaults() {
        let s = IslandSettings::default();
        assert_eq!(s.theme, "dark");
        assert!(s.hover_expand_enabled);
        assert_eq!(s.hover_delay_ms, 220);
        assert_eq!(s.leave_delay_ms, 450);
        assert_eq!(s.inactivity_timeout_s, 12);
        assert_eq!(s.top_margin, 12.0);
        assert!(s.show_ports_in_pill);
        assert!(s.show_docker_in_pill);
    }
}
