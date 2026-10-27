use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioOutputDevice {
    pub id: String, // e.g. "alsa_output...:analog-output-headphones" or "sink_name"
    pub sink_name: String,
    pub port_name: Option<String>,
    pub name: String,        // e.g. "Headphones" or "Speakers"
    pub description: String, // e.g. "Built-in Audio Analog Stereo"
    pub icon: String,        // "headphones", "speaker", "bluetooth", "hdmi"
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStatus {
    pub mic_muted: bool,
    pub mic_volume: u32,
    pub output_muted: bool,
    pub output_volume: u32,
    pub current_output_id: String,
    pub current_output_name: String,
    pub outputs: Vec<AudioOutputDevice>,
}

pub fn get_audio_status() -> Result<AudioStatus, String> {
    // 1. Get mic mute
    let mic_muted = match Command::new("pactl")
        .args(["get-source-mute", "@DEFAULT_SOURCE@"])
        .output()
    {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout).to_lowercase();
            s.contains("yes")
        }
        Err(_) => false,
    };

    // 2. Get mic volume
    let mic_volume = match Command::new("pactl")
        .args(["get-source-volume", "@DEFAULT_SOURCE@"])
        .output()
    {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout);
            parse_volume_percent(&s).unwrap_or(50)
        }
        Err(_) => 50,
    };

    // 3. Get sink mute
    let output_muted = match Command::new("pactl")
        .args(["get-sink-mute", "@DEFAULT_SINK@"])
        .output()
    {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout).to_lowercase();
            s.contains("yes")
        }
        Err(_) => false,
    };

    // 4. Get sink volume
    let output_volume = match Command::new("pactl")
        .args(["get-sink-volume", "@DEFAULT_SINK@"])
        .output()
    {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout);
            parse_volume_percent(&s).unwrap_or(75)
        }
        Err(_) => 75,
    };

    // 5. Get default sink name
    let default_sink = match Command::new("pactl")
        .args(["get-default-sink"])
        .output()
    {
        Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        Err(_) => String::new(),
    };

    // 6. Parse all sinks & ports
    let outputs = parse_all_sinks(&default_sink);

    let active_device = outputs.iter().find(|d| d.is_active);
    let current_output_id = active_device
        .map(|d| d.id.clone())
        .unwrap_or_else(|| default_sink.clone());
    let current_output_name = active_device
        .map(|d| d.name.clone())
        .unwrap_or_else(|| "Audio Output".to_string());

    Ok(AudioStatus {
        mic_muted,
        mic_volume,
        output_muted,
        output_volume,
        current_output_id,
        current_output_name,
        outputs,
    })
}

pub fn toggle_mic_mute() -> Result<bool, String> {
    let out = Command::new("pactl")
        .args(["set-source-mute", "@DEFAULT_SOURCE@", "toggle"])
        .output()
        .map_err(|e| format!("Failed to toggle mic mute: {e}"))?;

    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }

    // Return new state
    let check = Command::new("pactl")
        .args(["get-source-mute", "@DEFAULT_SOURCE@"])
        .output()
        .map_err(|e| format!("Failed to check mic mute: {e}"))?;

    let s = String::from_utf8_lossy(&check.stdout).to_lowercase();
    Ok(s.contains("yes"))
}

pub fn set_mic_volume(vol: u32) -> Result<(), String> {
    let clamped = vol.min(100);
    Command::new("pactl")
        .args(["set-source-volume", "@DEFAULT_SOURCE@", &format!("{clamped}%")])
        .output()
        .map_err(|e| format!("Failed to set mic volume: {e}"))?;
    Ok(())
}

pub fn set_output_volume(vol: u32) -> Result<(), String> {
    let clamped = vol.min(100);
    Command::new("pactl")
        .args(["set-sink-volume", "@DEFAULT_SINK@", &format!("{clamped}%")])
        .output()
        .map_err(|e| format!("Failed to set output volume: {e}"))?;
    Ok(())
}

pub fn toggle_output_mute() -> Result<bool, String> {
    Command::new("pactl")
        .args(["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
        .output()
        .map_err(|e| format!("Failed to toggle sink mute: {e}"))?;

    let check = Command::new("pactl")
        .args(["get-sink-mute", "@DEFAULT_SINK@"])
        .output()
        .map_err(|e| format!("Failed to check sink mute: {e}"))?;

    let s = String::from_utf8_lossy(&check.stdout).to_lowercase();
    Ok(s.contains("yes"))
}

pub fn set_audio_output(target_id: &str) -> Result<(), String> {
    // If target_id contains ':', it's "sink_name:port_name"
    if let Some((sink, port)) = target_id.split_once(':') {
        let _ = Command::new("pactl")
            .args(["set-default-sink", sink])
            .output();

        let out = Command::new("pactl")
            .args(["set-sink-port", sink, port])
            .output()
            .map_err(|e| format!("Failed to set sink port: {e}"))?;

        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).to_string());
        }
    } else {
        let out = Command::new("pactl")
            .args(["set-default-sink", target_id])
            .output()
            .map_err(|e| format!("Failed to set default sink: {e}"))?;

        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).to_string());
        }
    }

    Ok(())
}

fn parse_volume_percent(output: &str) -> Option<u32> {
    // Looks like: "Volume: front-left: 58968 /  90% / -2,75 dB..."
    for part in output.split('/') {
        let trimmed = part.trim();
        if let Some(val_str) = trimmed.strip_suffix('%') {
            if let Ok(num) = val_str.trim().parse::<u32>() {
                return Some(num);
            }
        }
    }
    None
}

fn parse_all_sinks(default_sink: &str) -> Vec<AudioOutputDevice> {
    let out = match Command::new("pactl").args(["list", "sinks"]).output() {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => return Vec::new(),
    };

    let mut devices = Vec::new();
    let sections = out.split("Sink #");

    for section in sections {
        if section.trim().is_empty() {
            continue;
        }

        let mut name = String::new();
        let mut description = String::new();
        let mut active_port = String::new();
        let mut ports = Vec::new(); // Vec<(port_id, port_human_name)>

        let mut in_ports_section = false;

        for line in section.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Name:") {
                name = trimmed.trim_start_matches("Name:").trim().to_string();
                in_ports_section = false;
            } else if trimmed.starts_with("Description:") {
                description = trimmed.trim_start_matches("Description:").trim().to_string();
                in_ports_section = false;
            } else if trimmed.starts_with("Active Port:") {
                active_port = trimmed.trim_start_matches("Active Port:").trim().to_string();
                in_ports_section = false;
            } else if trimmed == "Ports:" {
                in_ports_section = true;
            } else if trimmed.starts_with("Formats:") || trimmed.starts_with("Properties:") {
                in_ports_section = false;
            } else if in_ports_section && trimmed.contains(':') {
                // e.g. "analog-output-speaker: Speakers (type: Speaker..."
                if let Some((p_id, p_rest)) = trimmed.split_once(':') {
                    let clean_id = p_id.trim().to_string();
                    let human_name = p_rest
                        .split('(')
                        .next()
                        .unwrap_or(&clean_id)
                        .trim()
                        .to_string();
                    ports.push((clean_id, human_name));
                }
            }
        }

        let is_current_sink = name == default_sink;

        if ports.is_empty() {
            // Standalone sink without multiple ports
            let icon = determine_icon(&name, &description);
            devices.push(AudioOutputDevice {
                id: name.clone(),
                sink_name: name.clone(),
                port_name: None,
                name: if description.is_empty() { name.clone() } else { description.clone() },
                description: description.clone(),
                icon,
                is_active: is_current_sink,
            });
        } else {
            // Has ports (e.g. Speakers vs Headphones)
            for (p_id, p_human) in ports {
                let id = format!("{name}:{p_id}");
                let is_active = is_current_sink && (active_port.is_empty() || active_port == p_id);
                let icon = if p_id.contains("headphone") || p_human.to_lowercase().contains("headphone") {
                    "headphones".to_string()
                } else if p_id.contains("speaker") || p_human.to_lowercase().contains("speaker") {
                    "speaker".to_string()
                } else {
                    determine_icon(&name, &description)
                };

                devices.push(AudioOutputDevice {
                    id,
                    sink_name: name.clone(),
                    port_name: Some(p_id),
                    name: p_human,
                    description: description.clone(),
                    icon,
                    is_active,
                });
            }
        }
    }

    devices
}

fn determine_icon(name: &str, desc: &str) -> String {
    let lower = format!("{name} {desc}").to_lowercase();
    if lower.contains("blue") || lower.contains("bt") {
        "bluetooth".to_string()
    } else if lower.contains("hdmi") || lower.contains("displayport") || lower.contains("dp") {
        "hdmi".to_string()
    } else if lower.contains("headphone") || lower.contains("headset") {
        "headphones".to_string()
    } else {
        "speaker".to_string()
    }
}
