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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AudioBackend {
    PulseAudio,
    WirePlumber,
    Alsa,
    None,
}

pub fn detect_backend() -> AudioBackend {
    // 1. Check if pactl exists and can talk to PulseAudio / PipeWire-Pulse daemon
    if let Ok(out) = Command::new("pactl").arg("info").output() {
        if out.status.success() {
            return AudioBackend::PulseAudio;
        }
    }

    // 2. Check if wpctl (native PipeWire / WirePlumber) is available and functional
    if let Ok(out) = Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SINK@"]).output() {
        if out.status.success() {
            return AudioBackend::WirePlumber;
        }
    }

    // 3. Fallback to ALSA via amixer
    if let Ok(out) = Command::new("amixer").arg("scontrols").output() {
        if out.status.success() && !out.stdout.is_empty() {
            return AudioBackend::Alsa;
        }
    }

    AudioBackend::None
}

pub fn get_audio_status() -> Result<AudioStatus, String> {
    match detect_backend() {
        AudioBackend::PulseAudio => get_audio_status_pactl(),
        AudioBackend::WirePlumber => get_audio_status_wpctl(),
        AudioBackend::Alsa => get_audio_status_amixer(),
        AudioBackend::None => Err("No supported audio control backend found (pactl, wpctl, amixer)".to_string()),
    }
}

pub fn toggle_mic_mute() -> Result<bool, String> {
    match detect_backend() {
        AudioBackend::PulseAudio => toggle_mic_mute_pactl(),
        AudioBackend::WirePlumber => toggle_mic_mute_wpctl(),
        AudioBackend::Alsa => toggle_mic_mute_amixer(),
        AudioBackend::None => Err("No supported audio control backend found".to_string()),
    }
}

pub fn set_mic_volume(vol: u32) -> Result<(), String> {
    let clamped = vol.min(100);
    match detect_backend() {
        AudioBackend::PulseAudio => set_mic_volume_pactl(clamped),
        AudioBackend::WirePlumber => set_mic_volume_wpctl(clamped),
        AudioBackend::Alsa => set_mic_volume_amixer(clamped),
        AudioBackend::None => Err("No supported audio control backend found".to_string()),
    }
}

pub fn toggle_output_mute() -> Result<bool, String> {
    match detect_backend() {
        AudioBackend::PulseAudio => toggle_output_mute_pactl(),
        AudioBackend::WirePlumber => toggle_output_mute_wpctl(),
        AudioBackend::Alsa => toggle_output_mute_amixer(),
        AudioBackend::None => Err("No supported audio control backend found".to_string()),
    }
}

pub fn set_output_volume(vol: u32) -> Result<(), String> {
    let clamped = vol.min(100);
    match detect_backend() {
        AudioBackend::PulseAudio => set_output_volume_pactl(clamped),
        AudioBackend::WirePlumber => set_output_volume_wpctl(clamped),
        AudioBackend::Alsa => set_output_volume_amixer(clamped),
        AudioBackend::None => Err("No supported audio control backend found".to_string()),
    }
}

pub fn set_audio_output(target_id: &str) -> Result<(), String> {
    match detect_backend() {
        AudioBackend::PulseAudio => set_audio_output_pactl(target_id),
        AudioBackend::WirePlumber => {
            // target_id is sink id or name
            let clean_id = target_id.split(':').next().unwrap_or(target_id);
            let out = Command::new("wpctl")
                .args(["set-default", clean_id])
                .output()
                .map_err(|e| format!("Failed to set default sink via wpctl: {e}"))?;
            if !out.status.success() {
                return Err(String::from_utf8_lossy(&out.stderr).to_string());
            }
            Ok(())
        }
        _ => Err("Switching audio output device is not supported on this audio backend".to_string()),
    }
}

// ============================================================================
// 1. PulseAudio / PipeWire-Pulse (pactl)
// ============================================================================

fn get_audio_status_pactl() -> Result<AudioStatus, String> {
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

    let default_sink = match Command::new("pactl")
        .args(["get-default-sink"])
        .output()
    {
        Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        Err(_) => String::new(),
    };

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

fn toggle_mic_mute_pactl() -> Result<bool, String> {
    let out = Command::new("pactl")
        .args(["set-source-mute", "@DEFAULT_SOURCE@", "toggle"])
        .output()
        .map_err(|e| format!("Failed to toggle mic mute: {e}"))?;

    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }

    let check = Command::new("pactl")
        .args(["get-source-mute", "@DEFAULT_SOURCE@"])
        .output()
        .map_err(|e| format!("Failed to check mic mute: {e}"))?;

    let s = String::from_utf8_lossy(&check.stdout).to_lowercase();
    Ok(s.contains("yes"))
}

fn set_mic_volume_pactl(vol: u32) -> Result<(), String> {
    Command::new("pactl")
        .args(["set-source-volume", "@DEFAULT_SOURCE@", &format!("{vol}%")])
        .output()
        .map_err(|e| format!("Failed to set mic volume: {e}"))?;
    Ok(())
}

fn toggle_output_mute_pactl() -> Result<bool, String> {
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

fn set_output_volume_pactl(vol: u32) -> Result<(), String> {
    Command::new("pactl")
        .args(["set-sink-volume", "@DEFAULT_SINK@", &format!("{vol}%")])
        .output()
        .map_err(|e| format!("Failed to set output volume: {e}"))?;
    Ok(())
}

fn set_audio_output_pactl(target_id: &str) -> Result<(), String> {
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
        let mut ports = Vec::new();

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

// ============================================================================
// 2. WirePlumber / Native PipeWire (wpctl)
// ============================================================================

fn parse_wpctl_volume(output: &str) -> (u32, bool) {
    let is_muted = output.contains("[MUTED]");
    let mut vol = 50;
    if let Some(v_str) = output.split_whitespace().nth(1) {
        if let Ok(f) = v_str.parse::<f32>() {
            vol = (f * 100.0).round().min(100.0) as u32;
        }
    }
    (vol, is_muted)
}

fn get_audio_status_wpctl() -> Result<AudioStatus, String> {
    let sink_out = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
        .map_err(|e| format!("Failed to run wpctl: {e}"))?;
    let sink_str = String::from_utf8_lossy(&sink_out.stdout);
    let (output_volume, output_muted) = parse_wpctl_volume(&sink_str);

    let source_out = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SOURCE@"])
        .output()
        .map_err(|e| format!("Failed to run wpctl: {e}"))?;
    let source_str = String::from_utf8_lossy(&source_out.stdout);
    let (mic_volume, mic_muted) = parse_wpctl_volume(&source_str);

    let mut outputs = Vec::new();
    outputs.push(AudioOutputDevice {
        id: "@DEFAULT_AUDIO_SINK@".to_string(),
        sink_name: "@DEFAULT_AUDIO_SINK@".to_string(),
        port_name: None,
        name: "Default Audio Device".to_string(),
        description: "PipeWire Audio Sink".to_string(),
        icon: "speaker".to_string(),
        is_active: true,
    });

    Ok(AudioStatus {
        mic_muted,
        mic_volume,
        output_muted,
        output_volume,
        current_output_id: "@DEFAULT_AUDIO_SINK@".to_string(),
        current_output_name: "Default Audio Device".to_string(),
        outputs,
    })
}

fn toggle_mic_mute_wpctl() -> Result<bool, String> {
    let out = Command::new("wpctl")
        .args(["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"])
        .output()
        .map_err(|e| format!("Failed to toggle mic mute via wpctl: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }

    let check = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SOURCE@"])
        .output()
        .map_err(|e| format!("Failed to check mic mute via wpctl: {e}"))?;
    let s = String::from_utf8_lossy(&check.stdout);
    Ok(s.contains("[MUTED]"))
}

fn set_mic_volume_wpctl(vol: u32) -> Result<(), String> {
    let out = Command::new("wpctl")
        .args(["set-volume", "-l", "1.0", "@DEFAULT_AUDIO_SOURCE@", &format!("{vol}%")])
        .output()
        .map_err(|e| format!("Failed to set mic volume via wpctl: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }
    Ok(())
}

fn toggle_output_mute_wpctl() -> Result<bool, String> {
    let out = Command::new("wpctl")
        .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
        .output()
        .map_err(|e| format!("Failed to toggle output mute via wpctl: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }

    let check = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
        .map_err(|e| format!("Failed to check output mute via wpctl: {e}"))?;
    let s = String::from_utf8_lossy(&check.stdout);
    Ok(s.contains("[MUTED]"))
}

fn set_output_volume_wpctl(vol: u32) -> Result<(), String> {
    let out = Command::new("wpctl")
        .args(["set-volume", "-l", "1.0", "@DEFAULT_AUDIO_SINK@", &format!("{vol}%")])
        .output()
        .map_err(|e| format!("Failed to set output volume via wpctl: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }
    Ok(())
}

// ============================================================================
// 3. ALSA (amixer)
// ============================================================================

fn parse_amixer_volume(output: &str) -> Option<u32> {
    for part in output.split('[') {
        if let Some((num_str, _)) = part.split_once('%') {
            if let Ok(val) = num_str.trim().parse::<u32>() {
                return Some(val);
            }
        }
    }
    None
}

fn parse_amixer_muted(output: &str) -> bool {
    output.contains("[off]")
}

fn get_audio_status_amixer() -> Result<AudioStatus, String> {
    let master_out = Command::new("amixer")
        .args(["sget", "Master"])
        .output()
        .map_err(|e| format!("Failed to run amixer sget Master: {e}"))?;
    let master_str = String::from_utf8_lossy(&master_out.stdout);
    let output_volume = parse_amixer_volume(&master_str).unwrap_or(75);
    let output_muted = parse_amixer_muted(&master_str);

    let capture_out = Command::new("amixer")
        .args(["sget", "Capture"])
        .output();
    let (mic_volume, mic_muted) = if let Ok(ref co) = capture_out {
        let cap_str = String::from_utf8_lossy(&co.stdout);
        (parse_amixer_volume(&cap_str).unwrap_or(50), parse_amixer_muted(&cap_str))
    } else {
        (50, false)
    };

    let mut outputs = Vec::new();
    outputs.push(AudioOutputDevice {
        id: "alsa:Master".to_string(),
        sink_name: "Master".to_string(),
        port_name: None,
        name: "ALSA Master Output".to_string(),
        description: "Default ALSA Hardware Device".to_string(),
        icon: "speaker".to_string(),
        is_active: true,
    });

    Ok(AudioStatus {
        mic_muted,
        mic_volume,
        output_muted,
        output_volume,
        current_output_id: "alsa:Master".to_string(),
        current_output_name: "ALSA Master Output".to_string(),
        outputs,
    })
}

fn toggle_mic_mute_amixer() -> Result<bool, String> {
    let out = Command::new("amixer")
        .args(["sset", "Capture", "toggle"])
        .output()
        .map_err(|e| format!("Failed to toggle Capture mute via amixer: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }

    let check = Command::new("amixer")
        .args(["sget", "Capture"])
        .output()
        .map_err(|e| format!("Failed to read Capture state via amixer: {e}"))?;
    let s = String::from_utf8_lossy(&check.stdout);
    Ok(parse_amixer_muted(&s))
}

fn set_mic_volume_amixer(vol: u32) -> Result<(), String> {
    let out = Command::new("amixer")
        .args(["sset", "Capture", &format!("{vol}%")])
        .output()
        .map_err(|e| format!("Failed to set Capture volume via amixer: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }
    Ok(())
}

fn toggle_output_mute_amixer() -> Result<bool, String> {
    let out = Command::new("amixer")
        .args(["sset", "Master", "toggle"])
        .output()
        .map_err(|e| format!("Failed to toggle Master mute via amixer: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }

    let check = Command::new("amixer")
        .args(["sget", "Master"])
        .output()
        .map_err(|e| format!("Failed to read Master state via amixer: {e}"))?;
    let s = String::from_utf8_lossy(&check.stdout);
    Ok(parse_amixer_muted(&s))
}

fn set_output_volume_amixer(vol: u32) -> Result<(), String> {
    let out = Command::new("amixer")
        .args(["sset", "Master", &format!("{vol}%")])
        .output()
        .map_err(|e| format!("Failed to set Master volume via amixer: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }
    Ok(())
}

// ============================================================================
// Helper Utilities
// ============================================================================

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_volume_percent() {
        let output = "Volume: front-left: 58968 /  90% / -2,75 dB,   front-right: 58968 /  90% / -2,75 dB";
        assert_eq!(parse_volume_percent(output), Some(90));
    }

    #[test]
    fn test_parse_wpctl_volume() {
        let s1 = "Volume: 0.40";
        let (vol1, mute1) = parse_wpctl_volume(s1);
        assert_eq!(vol1, 40);
        assert!(!mute1);

        let s2 = "Volume: 0.75 [MUTED]";
        let (vol2, mute2) = parse_wpctl_volume(s2);
        assert_eq!(vol2, 75);
        assert!(mute2);
    }

    #[test]
    fn test_parse_amixer() {
        let s = "  Front Left: Playback 26214 [40%] [on]";
        assert_eq!(parse_amixer_volume(s), Some(40));
        assert!(!parse_amixer_muted(s));

        let s_off = "  Front Left: Playback 26214 [65%] [off]";
        assert_eq!(parse_amixer_volume(s_off), Some(65));
        assert!(parse_amixer_muted(s_off));
    }

    #[test]
    fn test_determine_icon() {
        assert_eq!(determine_icon("hdmi-output", "HDMI Audio"), "hdmi");
        assert_eq!(determine_icon("bluez_sink", "Wireless Headphones"), "bluetooth");
        assert_eq!(determine_icon("analog-stereo", "Built-in Speakers"), "speaker");
    }
}
