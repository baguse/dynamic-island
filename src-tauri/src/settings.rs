use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn default_theme() -> String {
    "dark".to_string()
}

fn default_true() -> bool {
    true
}

fn default_module_order() -> Vec<String> {
    vec![
        "media".to_string(),
        "agent".to_string(),
        "build".to_string(),
        "system".to_string(),
        "git".to_string(),
        "editor".to_string(),
        "ports".to_string(),
        "docker".to_string(),
        "mic".to_string(),
        "clipboard".to_string(),
        "shelf".to_string(),
        "todo".to_string(),
        "focus".to_string(),
        "prayer".to_string(),
    ]
}

fn default_prayer_method() -> String {
    "KEMENAG".to_string()
}

fn default_prayer_asr() -> String {
    "standard".to_string()
}

fn default_prayer_pre_mins() -> u32 {
    10
}

fn default_prayer_audio() -> String {
    "chime".to_string()
}

fn default_horizontal_position() -> String {
    "center".to_string()
}

fn default_companion_style() -> String {
    "vinyl".to_string()
}

fn default_equalizer_style() -> String {
    "bars".to_string()
}

fn default_ambient_sky_theme() -> String {
    "auto".to_string()
}

fn default_ambient_sky_intensity() -> String {
    "subtle".to_string()
}

fn default_mascot_type() -> String {
    "cat".to_string()
}

pub fn default_shortcuts() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("toggle".to_string(), "Super+Shift+I".to_string());
    map.insert("agent".to_string(), "Super+Shift+A".to_string());
    map.insert("media".to_string(), "Super+Shift+M".to_string());
    map.insert("git".to_string(), "Super+Shift+G".to_string());
    map.insert("system".to_string(), "Super+Shift+S".to_string());
    map.insert("clipboard".to_string(), "Super+Shift+C".to_string());
    map.insert("editor".to_string(), "Super+Shift+E".to_string());
    map.insert("docker".to_string(), "Super+Shift+D".to_string());
    map.insert("ports".to_string(), "Super+Shift+P".to_string());
    map.insert("todo".to_string(), "Super+Shift+T".to_string());
    map.insert("prayer".to_string(), "Super+Shift+R".to_string());
    map.insert("settings".to_string(), "Super+Shift+Comma".to_string());
    map
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
    #[serde(default = "default_horizontal_position")]
    pub horizontal_position: String,
    #[serde(default)]
    pub x_offset: f64,
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
    #[serde(default = "default_true")]
    pub show_todo_in_pill: bool,
    #[serde(default = "default_true")]
    pub show_focus_in_pill: bool,
    #[serde(default = "default_true")]
    pub show_prayer_in_pill: bool,
    #[serde(default)]
    pub prayer_city: Option<String>,
    #[serde(default = "default_prayer_method")]
    pub prayer_calculation_method: String,
    #[serde(default = "default_prayer_asr")]
    pub prayer_asr_school: String,
    #[serde(default = "default_true")]
    pub prayer_notifications_enabled: bool,
    #[serde(default = "default_prayer_pre_mins")]
    pub prayer_pre_reminder_mins: u32,
    #[serde(default = "default_prayer_audio")]
    pub prayer_audio_mode: String,
    #[serde(default = "default_module_order")]
    pub module_order: Vec<String>,
    #[serde(default)]
    pub watched_repo_path: Option<String>,
    #[serde(default = "default_true")]
    pub show_media_companion: bool,
    #[serde(default)]
    pub media_companion_path: Option<String>,
    #[serde(default = "default_companion_style")]
    pub media_companion_style: String,
    #[serde(default = "default_equalizer_style")]
    pub equalizer_style: String,
    #[serde(default = "default_true")]
    pub ambient_sky_enabled: bool,
    #[serde(default = "default_ambient_sky_theme")]
    pub ambient_sky_theme: String,
    #[serde(default = "default_ambient_sky_intensity")]
    pub ambient_sky_intensity: String,
    #[serde(default)]
    pub weather_city: Option<String>,
    #[serde(default = "default_true")]
    pub mascot_enabled: bool,
    #[serde(default = "default_mascot_type")]
    pub mascot_type: String,
    #[serde(default = "default_true")]
    pub mascot_interactive: bool,
    pub autostart_enabled: bool,
    #[serde(default = "default_shortcuts")]
    pub shortcuts: HashMap<String, String>,
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
            horizontal_position: "center".to_string(),
            x_offset: 0.0,
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
            show_todo_in_pill: true,
            show_focus_in_pill: true,
            show_prayer_in_pill: true,
            prayer_city: None,
            prayer_calculation_method: default_prayer_method(),
            prayer_asr_school: default_prayer_asr(),
            prayer_notifications_enabled: true,
            prayer_pre_reminder_mins: default_prayer_pre_mins(),
            prayer_audio_mode: default_prayer_audio(),
            module_order: default_module_order(),
            watched_repo_path: None,
            show_media_companion: true,
            media_companion_path: None,
            media_companion_style: "vinyl".to_string(),
            equalizer_style: "bars".to_string(),
            ambient_sky_enabled: true,
            ambient_sky_theme: "auto".to_string(),
            ambient_sky_intensity: "subtle".to_string(),
            weather_city: None,
            mascot_enabled: true,
            mascot_type: "cat".to_string(),
            mascot_interactive: true,
            autostart_enabled: false,
            shortcuts: default_shortcuts(),
        }
    }
}

pub fn get_config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("dynamic-island");
        }
    }

    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(config_home).join("dynamic-island")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config").join("dynamic-island")
    } else {
        std::env::temp_dir().join("dynamic-island")
    }
}

fn get_settings_file_path() -> PathBuf {
    get_config_dir().join("settings.json")
}

#[cfg(not(target_os = "windows"))]
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

#[cfg(target_os = "windows")]
pub fn is_autostart_configured() -> bool {
    std::process::Command::new("reg")
        .args(["query", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", "DynamicIsland"])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
pub fn set_autostart_configured(enabled: bool) -> Result<(), String> {
    if enabled {
        let exe_path = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "dynamic-island.exe".to_string());

        let _ = std::process::Command::new("reg")
            .args([
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "DynamicIsland",
                "/t",
                "REG_SZ",
                "/d",
                &exe_path,
                "/f",
            ])
            .output()
            .map_err(|e| e.to_string())?;
    } else {
        let _ = std::process::Command::new("reg")
            .args([
                "delete",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "DynamicIsland",
                "/f",
            ])
            .output();
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn is_autostart_configured() -> bool {
    get_autostart_desktop_file_path()
        .map(|p| p.exists())
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
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
            .map(|p| {
                let p_str = p.to_string_lossy().to_string();
                // If running a debug binary, avoid registering target/debug/dynamic-island
                // which attempts to connect to Vite (http://localhost:1420) on system startup.
                if p_str.contains("target/debug") || p_str.contains("target\\debug") {
                    if std::path::Path::new("/usr/bin/dynamic-island").exists() {
                        "/usr/bin/dynamic-island".to_string()
                    } else if std::path::Path::new("/usr/local/bin/dynamic-island").exists() {
                        "/usr/local/bin/dynamic-island".to_string()
                    } else {
                        let release_candidate = p
                            .parent()
                            .and_then(|parent| parent.parent())
                            .map(|target_dir| target_dir.join("release").join("dynamic-island"));
                        if let Some(rel) = release_candidate {
                            if rel.exists() {
                                return rel.to_string_lossy().to_string();
                            }
                        }
                        "dynamic-island".to_string()
                    }
                } else {
                    p_str
                }
            })
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
        assert!(s.show_todo_in_pill);
        assert!(s.show_focus_in_pill);
        assert!(s.show_prayer_in_pill);
        assert_eq!(s.module_order.len(), 14);
        assert_eq!(s.module_order[0], "media");
        assert!(s.module_order.contains(&"prayer".to_string()));
        assert!(s.show_media_companion);
        assert_eq!(s.media_companion_style, "vinyl");
        assert_eq!(s.equalizer_style, "bars");
        assert!(s.ambient_sky_enabled);
        assert_eq!(s.ambient_sky_theme, "auto");
        assert_eq!(s.ambient_sky_intensity, "subtle");
        assert!(s.mascot_enabled);
        assert_eq!(s.mascot_type, "cat");
        assert!(s.mascot_interactive);
    }

    #[test]
    fn test_module_order_backwards_compatibility() {
        let json = r#"{"theme":"neon","hover_expand_enabled":true,"hover_delay_ms":200,"leave_delay_ms":400,"inactivity_timeout_s":10,"top_margin":10.0,"show_system_in_pill":true,"show_battery_alert":true,"show_media_in_pill":true,"show_build_in_pill":true,"show_agent_in_pill":true,"autostart_enabled":false}"#;
        let s: IslandSettings = serde_json::from_str(json).unwrap();
        assert_eq!(s.module_order.len(), 14);
        assert_eq!(s.module_order[0], "media");
        assert!(s.module_order.contains(&"prayer".to_string()));
        assert!(s.show_media_companion);
        assert_eq!(s.media_companion_style, "vinyl");
        assert_eq!(s.equalizer_style, "bars");
        assert!(s.ambient_sky_enabled);
        assert_eq!(s.ambient_sky_theme, "auto");
        assert_eq!(s.ambient_sky_intensity, "subtle");
        assert!(s.mascot_enabled);
        assert_eq!(s.mascot_type, "cat");
        assert!(s.mascot_interactive);
    }

    #[test]
    fn test_custom_module_order() {
        let json = r#"{"theme":"dark","hover_expand_enabled":true,"hover_delay_ms":220,"leave_delay_ms":450,"inactivity_timeout_s":12,"top_margin":12.0,"show_system_in_pill":true,"show_battery_alert":true,"show_media_in_pill":true,"show_build_in_pill":true,"show_agent_in_pill":true,"module_order":["shelf","clipboard","git"],"autostart_enabled":false}"#;
        let s: IslandSettings = serde_json::from_str(json).unwrap();
        assert_eq!(s.module_order, vec!["shelf", "clipboard", "git"]);
    }
}
