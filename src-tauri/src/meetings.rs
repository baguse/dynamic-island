use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MeetingPlatform {
    Zoom,
    Discord,
    GoogleMeet,
    Slack,
    Teams,
    Webex,
    Other,
}

impl MeetingPlatform {
    pub fn display_name(&self) -> &'static str {
        match self {
            MeetingPlatform::Zoom => "Zoom",
            MeetingPlatform::Discord => "Discord",
            MeetingPlatform::GoogleMeet => "Google Meet",
            MeetingPlatform::Slack => "Slack",
            MeetingPlatform::Teams => "Microsoft Teams",
            MeetingPlatform::Webex => "Cisco Webex",
            MeetingPlatform::Other => "Meeting",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            MeetingPlatform::Zoom => "🟦",
            MeetingPlatform::Discord => "🎧",
            MeetingPlatform::GoogleMeet => "📹",
            MeetingPlatform::Slack => "💬",
            MeetingPlatform::Teams => "👥",
            MeetingPlatform::Webex => "🌐",
            MeetingPlatform::Other => "📞",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingInfo {
    pub is_active: bool,
    pub platform: MeetingPlatform,
    pub app_name: String,
    pub title: String,
    pub mic_active: bool,
    pub is_muted: bool,
    pub elapsed_seconds: u64,
    pub window_id: Option<String>,
}

#[derive(Default)]
struct MeetingTracker {
    current_meeting: Option<MeetingInfo>,
    meeting_start_instant: Option<Instant>,
    last_platform: Option<MeetingPlatform>,
}

static TRACKER: OnceLock<Mutex<MeetingTracker>> = OnceLock::new();

fn get_tracker() -> &'static Mutex<MeetingTracker> {
    TRACKER.get_or_init(|| Mutex::new(MeetingTracker::default()))
}

/// Raw source-output item extracted from `pactl list source-outputs`
#[derive(Debug, Clone)]
struct AudioSourceOutput {
    app_name: String,
    binary: String,
    media_name: String,
    is_corked: bool,
    is_muted: bool,
}

/// Parse output of `pactl list source-outputs`
fn parse_source_outputs(output: &str) -> Vec<AudioSourceOutput> {
    let mut results = Vec::new();
    let mut current_app = String::new();
    let mut current_binary = String::new();
    let mut current_media = String::new();
    let mut current_corked = false;
    let mut current_muted = false;
    let mut in_block = false;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Source Output #") {
            if in_block && (!current_app.is_empty() || !current_binary.is_empty()) {
                results.push(AudioSourceOutput {
                    app_name: current_app.clone(),
                    binary: current_binary.clone(),
                    media_name: current_media.clone(),
                    is_corked: current_corked,
                    is_muted: current_muted,
                });
            }
            in_block = true;
            current_app.clear();
            current_binary.clear();
            current_media.clear();
            current_corked = false;
            current_muted = false;
            continue;
        }

        if !in_block {
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("Mute: ") {
            current_muted = rest.trim().eq_ignore_ascii_case("yes");
        } else if let Some(rest) = trimmed.strip_prefix("Corked: ") {
            current_corked = rest.trim().eq_ignore_ascii_case("yes");
        } else if trimmed.starts_with("application.name = ") {
            current_app = clean_quoted(trimmed.strip_prefix("application.name = ").unwrap_or(""));
        } else if trimmed.starts_with("application.process.binary = ") {
            current_binary = clean_quoted(trimmed.strip_prefix("application.process.binary = ").unwrap_or(""));
        } else if trimmed.starts_with("media.name = ") {
            current_media = clean_quoted(trimmed.strip_prefix("media.name = ").unwrap_or(""));
        }
    }

    if in_block && (!current_app.is_empty() || !current_binary.is_empty()) {
        results.push(AudioSourceOutput {
            app_name: current_app,
            binary: current_binary,
            media_name: current_media,
            is_corked: current_corked,
            is_muted: current_muted,
        });
    }

    results
}

fn clean_quoted(val: &str) -> String {
    let s = val.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Window entry from `wmctrl -lx`
#[derive(Debug, Clone)]
struct WindowEntry {
    id: String,
    class: String,
    title: String,
}

fn get_active_windows() -> Vec<WindowEntry> {
    let output = match Command::new("wmctrl").arg("-lx").output() {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => return Vec::new(),
    };

    let mut windows = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let id = parts[0].to_string();
            let class = parts[2].to_string();
            // The title starts after id, desktop, class, client machine
            let title = line
                .splitn(5, char::is_whitespace)
                .nth(4)
                .unwrap_or("")
                .trim()
                .to_string();
            windows.push(WindowEntry { id, class, title });
        }
    }
    windows
}

/// Detect meeting status from active audio sources and open windows
pub fn detect_active_meeting() -> Option<MeetingInfo> {
    let pactl_out = match Command::new("pactl").args(["list", "source-outputs"]).output() {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => return None,
    };

    let sources = parse_source_outputs(&pactl_out);
    let windows = get_active_windows();

    // 1. Check Zoom
    for s in &sources {
        let bin = s.binary.to_lowercase();
        let app = s.app_name.to_lowercase();
        if bin.contains("zoom") || app.contains("zoom") {
            let zoom_win = windows.iter().find(|w| {
                let c = w.class.to_lowercase();
                let t = w.title.to_lowercase();
                c.contains("zoom") || t.contains("zoom meeting") || t.contains("zoom webinar")
            });

            let title = zoom_win
                .map(|w| w.title.clone())
                .unwrap_or_else(|| "Zoom Meeting".to_string());
            let win_id = zoom_win.map(|w| w.id.clone());

            return Some(MeetingInfo {
                is_active: true,
                platform: MeetingPlatform::Zoom,
                app_name: "Zoom".to_string(),
                title,
                mic_active: !s.is_corked && !s.is_muted,
                is_muted: s.is_muted,
                elapsed_seconds: 0,
                window_id: win_id,
            });
        }
    }

    // 2. Check Discord
    for s in &sources {
        let bin = s.binary.to_lowercase();
        let app = s.app_name.to_lowercase();
        if bin.contains("discord") || bin.contains("vesktop") || bin.contains("webcord") || app.contains("discord") {
            let discord_win = windows.iter().find(|w| {
                let c = w.class.to_lowercase();
                let t = w.title.to_lowercase();
                c.contains("discord") || c.contains("vesktop") || t.contains("discord")
            });

            let title = discord_win
                .map(|w| {
                    if let Some(ch) = w.title.split('|').next() {
                        let trimmed = ch.trim();
                        if !trimmed.is_empty() {
                            return format!("Discord: {}", trimmed);
                        }
                    }
                    "Discord Voice".to_string()
                })
                .unwrap_or_else(|| "Discord Voice".to_string());

            let win_id = discord_win.map(|w| w.id.clone());

            return Some(MeetingInfo {
                is_active: true,
                platform: MeetingPlatform::Discord,
                app_name: "Discord".to_string(),
                title,
                mic_active: !s.is_corked && !s.is_muted,
                is_muted: s.is_muted,
                elapsed_seconds: 0,
                window_id: win_id,
            });
        }
    }

    // 3. Check Slack Huddles / Calls
    for s in &sources {
        let bin = s.binary.to_lowercase();
        let app = s.app_name.to_lowercase();
        if bin.contains("slack") || app.contains("slack") {
            let slack_win = windows.iter().find(|w| {
                w.class.to_lowercase().contains("slack") || w.title.to_lowercase().contains("slack")
            });
            let title = slack_win
                .map(|w| format!("Slack: {}", w.title))
                .unwrap_or_else(|| "Slack Huddle".to_string());
            let win_id = slack_win.map(|w| w.id.clone());

            return Some(MeetingInfo {
                is_active: true,
                platform: MeetingPlatform::Slack,
                app_name: "Slack".to_string(),
                title,
                mic_active: !s.is_corked && !s.is_muted,
                is_muted: s.is_muted,
                elapsed_seconds: 0,
                window_id: win_id,
            });
        }
    }

    // 4. Check Microsoft Teams
    for s in &sources {
        let bin = s.binary.to_lowercase();
        let app = s.app_name.to_lowercase();
        if bin.contains("teams") || app.contains("teams") {
            let teams_win = windows.iter().find(|w| {
                w.class.to_lowercase().contains("teams") || w.title.to_lowercase().contains("teams")
            });
            let title = teams_win
                .map(|w| w.title.clone())
                .unwrap_or_else(|| "Microsoft Teams Meeting".to_string());
            let win_id = teams_win.map(|w| w.id.clone());

            return Some(MeetingInfo {
                is_active: true,
                platform: MeetingPlatform::Teams,
                app_name: "Microsoft Teams".to_string(),
                title,
                mic_active: !s.is_corked && !s.is_muted,
                is_muted: s.is_muted,
                elapsed_seconds: 0,
                window_id: win_id,
            });
        }
    }

    // 5. Check Google Meet / WebRTC Browser Calls
    // Any browser capturing microphone (chrome, chromium, firefox, brave, vivaldi, edge)
    for s in &sources {
        let bin = s.binary.to_lowercase();
        let app = s.app_name.to_lowercase();
        let is_browser = bin.contains("chrome")
            || bin.contains("chromium")
            || bin.contains("firefox")
            || bin.contains("brave")
            || bin.contains("vivaldi")
            || bin.contains("msedge")
            || app.contains("google chrome")
            || app.contains("chromium")
            || app.contains("firefox")
            || app.contains("brave");

        if is_browser {
            // Find if there is a window containing Meet or Webex
            let meet_win = windows.iter().find(|w| {
                let t = w.title.to_lowercase();
                t.contains("meet -") || t.contains("meet –") || t.contains("meet.google.com")
            });

            if let Some(win) = meet_win {
                let mut title = win.title.clone();
                // Clean browser title suffix (e.g. " - Google Chrome", " — Mozilla Firefox")
                if let Some(idx) = title.find(" - Google Chrome") {
                    title.truncate(idx);
                } else if let Some(idx) = title.find(" — Mozilla Firefox") {
                    title.truncate(idx);
                } else if let Some(idx) = title.find(" - Brave") {
                    title.truncate(idx);
                }

                return Some(MeetingInfo {
                    is_active: true,
                    platform: MeetingPlatform::GoogleMeet,
                    app_name: "Google Meet".to_string(),
                    title,
                    mic_active: !s.is_corked && !s.is_muted,
                    is_muted: s.is_muted,
                    elapsed_seconds: 0,
                    window_id: Some(win.id.clone()),
                });
            }

            // Webex via browser or other webrtc call
            let webex_win = windows.iter().find(|w| w.title.to_lowercase().contains("webex"));
            if let Some(win) = webex_win {
                return Some(MeetingInfo {
                    is_active: true,
                    platform: MeetingPlatform::Webex,
                    app_name: "Cisco Webex".to_string(),
                    title: win.title.clone(),
                    mic_active: !s.is_corked && !s.is_muted,
                    is_muted: s.is_muted,
                    elapsed_seconds: 0,
                    window_id: Some(win.id.clone()),
                });
            }

            // Fallback: WebRTC Browser Call (e.g. Teams web, Zoom web, Jitsi)
            let browser_win = windows.iter().find(|w| {
                let c = w.class.to_lowercase();
                c.contains("chrome") || c.contains("firefox") || c.contains("brave")
            });

            let win_id = browser_win.map(|w| w.id.clone());
            let title = browser_win
                .map(|w| format!("Call: {}", w.title))
                .unwrap_or_else(|| "Web Conference Call".to_string());

            return Some(MeetingInfo {
                is_active: true,
                platform: MeetingPlatform::Other,
                app_name: "Browser Call".to_string(),
                title,
                mic_active: !s.is_corked && !s.is_muted,
                is_muted: s.is_muted,
                elapsed_seconds: 0,
                window_id: win_id,
            });
        }
    }

    None
}

/// Bring the meeting application window to the foreground using `wmctrl -ia <window_id>` or `wmctrl -xa <class>`
pub fn focus_meeting_window(window_id: Option<&str>, platform: &MeetingPlatform) -> Result<(), String> {
    if let Some(id) = window_id {
        if !id.is_empty() {
            let res = Command::new("wmctrl").args(["-ia", id]).status();
            if let Ok(st) = res {
                if st.success() {
                    return Ok(());
                }
            }
        }
    }

    // Fallback: Target by class / name
    let class_name = match platform {
        MeetingPlatform::Zoom => "zoom",
        MeetingPlatform::Discord => "discord",
        MeetingPlatform::GoogleMeet => "google-chrome",
        MeetingPlatform::Slack => "slack",
        MeetingPlatform::Teams => "teams",
        MeetingPlatform::Webex => "webex",
        MeetingPlatform::Other => "chrome",
    };

    let res = Command::new("wmctrl").args(["-xa", class_name]).status();
    match res {
        Ok(st) if st.success() => Ok(()),
        Ok(_) => {
            // Also try by title substring
            let title_keyword = platform.display_name();
            let _ = Command::new("wmctrl").args(["-a", title_keyword]).status();
            Ok(())
        }
        Err(e) => Err(format!("Failed to activate window: {e}")),
    }
}

/// Periodic worker that monitors meeting streams every 1.5 seconds and emits Tauri events
pub fn start_meeting_monitor(app_handle: AppHandle, is_running: Arc<AtomicBool>) {
    std::thread::Builder::new()
        .name("meeting-monitor".to_string())
        .spawn(move || {
            let mut last_emitted_active = false;
            let mut last_emitted_mic = false;
            let mut last_emitted_platform: Option<MeetingPlatform> = None;

            while is_running.load(Ordering::Relaxed) {
                let detected = detect_active_meeting();
                let mut tracker = get_tracker().lock().unwrap();

                match detected {
                    Some(mut meeting) => {
                        // Track elapsed time
                        if tracker.last_platform != Some(meeting.platform.clone()) {
                            tracker.meeting_start_instant = Some(Instant::now());
                            tracker.last_platform = Some(meeting.platform.clone());
                        }

                        let elapsed = tracker
                            .meeting_start_instant
                            .map(|start| start.elapsed().as_secs())
                            .unwrap_or(0);
                        meeting.elapsed_seconds = elapsed;

                        let should_emit = !last_emitted_active
                            || last_emitted_mic != meeting.mic_active
                            || last_emitted_platform != Some(meeting.platform.clone())
                            || elapsed % 5 == 0; // heartbeats

                        if should_emit {
                            let _ = app_handle.emit("meeting-status-change", &meeting);
                            last_emitted_active = true;
                            last_emitted_mic = meeting.mic_active;
                            last_emitted_platform = Some(meeting.platform.clone());
                        }

                        tracker.current_meeting = Some(meeting);
                    }
                    None => {
                        if last_emitted_active {
                            let inactive = MeetingInfo {
                                is_active: false,
                                platform: MeetingPlatform::Other,
                                app_name: String::new(),
                                title: String::new(),
                                mic_active: false,
                                is_muted: false,
                                elapsed_seconds: 0,
                                window_id: None,
                            };
                            let _ = app_handle.emit("meeting-status-change", &inactive);
                            last_emitted_active = false;
                            last_emitted_platform = None;
                            tracker.current_meeting = None;
                            tracker.meeting_start_instant = None;
                            tracker.last_platform = None;
                        }
                    }
                }

                drop(tracker);
                std::thread::sleep(Duration::from_millis(1500));
            }
        })
        .expect("failed to spawn meeting-monitor thread");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_source_outputs_empty() {
        let list = parse_source_outputs("");
        assert!(list.is_empty());
    }

    #[test]
    fn test_parse_source_outputs_zoom_and_browser() {
        let sample = r#"
Source Output #101
	Driver: PipeWire
	Mute: no
	Corked: no
	Properties:
		application.name = "zoom"
		application.process.binary = "zoom"
		media.name = "Audio Callback"

Source Output #102
	Driver: PipeWire
	Mute: yes
	Corked: no
	Properties:
		application.name = "Google Chrome"
		application.process.binary = "chrome"
		media.name = "WebRTC Voice Engine"
"#;
        let list = parse_source_outputs(sample);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].binary, "zoom");
        assert!(!list[0].is_muted);
        assert!(!list[0].is_corked);
        assert_eq!(list[1].binary, "chrome");
        assert!(list[1].is_muted);
    }
}
