use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherData {
    pub temp_c: i32,
    pub temp_f: i32,
    pub condition: String,       // "clear", "clouds", "rain", "thunderstorm", "snow", "fog"
    pub description: String,     // e.g. "Light rain shower"
    pub location: String,        // e.g. "Surabaya" or "Tokyo"
    pub is_day: bool,
    pub time_period: String,     // "dawn", "day", "sunset", "night"
    pub weather_code: i32,
    pub humidity: i32,
    pub cloud_cover: i32,
    pub sunrise: String,
    pub sunset: String,
    pub updated_at: u64,
}

static WEATHER_CACHE: Mutex<Option<(WeatherData, Instant)>> = Mutex::new(None);
const CACHE_TTL: Duration = Duration::from_secs(900); // 15 minutes cache

/// Determine time period from local hour (0-23)
pub fn determine_time_period(hour: u32) -> (&'static str, bool) {
    match hour {
        5..=6 => ("dawn", true),
        7..=16 => ("day", true),
        17..=18 => ("sunset", true),
        _ => ("night", false),
    }
}

/// Map weather code and description to canonical condition
pub fn map_weather_condition(code: i32, desc: &str) -> &'static str {
    let desc_lower = desc.to_lowercase();
    if desc_lower.contains("thunder") || (386..=395).contains(&code) || code == 200 {
        "thunderstorm"
    } else if desc_lower.contains("snow")
        || desc_lower.contains("blizzard")
        || desc_lower.contains("sleet")
        || desc_lower.contains("ice")
        || (323..=338).contains(&code)
        || code == 227
    {
        "snow"
    } else if desc_lower.contains("rain")
        || desc_lower.contains("drizzle")
        || desc_lower.contains("shower")
        || (293..=314).contains(&code)
        || (353..=377).contains(&code)
        || code == 176
    {
        "rain"
    } else if desc_lower.contains("fog") || desc_lower.contains("mist") || code == 143 || code == 248 {
        "fog"
    } else if desc_lower.contains("cloud") || desc_lower.contains("overcast") || code == 116 || code == 119 || code == 122 {
        "clouds"
    } else {
        "clear"
    }
}

/// Get local hour using Linux `date +%H`
pub fn get_local_hour() -> u32 {
    Command::new("date")
        .arg("+%H")
        .output()
        .ok()
        .and_then(|out| {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            s.parse::<u32>().ok()
        })
        .unwrap_or(12)
}

/// Parse wttr.in JSON payload into WeatherData
pub fn parse_wttr_json(json_str: &str, local_hour: u32) -> Result<WeatherData, String> {
    let v: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid weather JSON: {}", e))?;

    let current = v
        .get("current_condition")
        .and_then(|cc| cc.get(0))
        .ok_or_else(|| "Missing current_condition in weather response".to_string())?;

    let temp_c = current
        .get("temp_C")
        .and_then(|t| t.as_str())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(22);

    let temp_f = current
        .get("temp_F")
        .and_then(|t| t.as_str())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or((temp_c as f32 * 9.0 / 5.0 + 32.0).round() as i32);

    let weather_code = current
        .get("weatherCode")
        .and_then(|c| c.as_str())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(113);

    let description = current
        .get("weatherDesc")
        .and_then(|wd| wd.get(0))
        .and_then(|item| item.get("value"))
        .and_then(|v| v.as_str())
        .unwrap_or("Clear")
        .to_string();

    let humidity = current
        .get("humidity")
        .and_then(|h| h.as_str())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(50);

    let cloud_cover = current
        .get("cloudcover")
        .and_then(|c| c.as_str())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    // Location name
    let location = v
        .get("nearest_area")
        .and_then(|na| na.get(0))
        .and_then(|area| area.get("areaName"))
        .and_then(|an| an.get(0))
        .and_then(|item| item.get("value"))
        .and_then(|s| s.as_str())
        .unwrap_or("Local")
        .to_string();

    // Astronomy (sunrise / sunset)
    let (sunrise, sunset) = v
        .get("weather")
        .and_then(|w| w.get(0))
        .and_then(|d| d.get("astronomy"))
        .and_then(|a| a.get(0))
        .map(|astro| {
            let sr = astro
                .get("sunrise")
                .and_then(|s| s.as_str())
                .unwrap_or("06:00 AM")
                .to_string();
            let ss = astro
                .get("sunset")
                .and_then(|s| s.as_str())
                .unwrap_or("06:00 PM")
                .to_string();
            (sr, ss)
        })
        .unwrap_or_else(|| ("06:00 AM".to_string(), "06:00 PM".to_string()));

    let (time_period, is_day) = determine_time_period(local_hour);
    let condition = map_weather_condition(weather_code, &description).to_string();
    let updated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Ok(WeatherData {
        temp_c,
        temp_f,
        condition,
        description,
        location,
        is_day,
        time_period: time_period.to_string(),
        weather_code,
        humidity,
        cloud_cover,
        sunrise,
        sunset,
        updated_at,
    })
}

/// Fetch live weather from wttr.in with memory caching
pub fn get_weather(city: Option<String>, force: bool) -> Result<WeatherData, String> {
    // Check cache first if no specific city requested and not explicitly forced
    if city.is_none() && !force {
        if let Ok(guard) = WEATHER_CACHE.lock() {
            if let Some((ref cached, ref timestamp)) = *guard {
                if timestamp.elapsed() < CACHE_TTL {
                    return Ok(cached.clone());
                }
            }
        }
    }

    let url = match &city {
        Some(c) if !c.trim().is_empty() => {
            let encoded = c.trim().replace(' ', "+");
            format!("https://wttr.in/{}?format=j1", encoded)
        }
        _ => "https://wttr.in/?format=j1".to_string(),
    };

    let output = Command::new("curl")
        .args(["-s", "--max-time", "5", &url])
        .output()
        .map_err(|e| format!("Failed to execute curl: {}", e))?;

    if !output.status.success() {
        return Err("Curl request to weather service failed".to_string());
    }

    let body = String::from_utf8_lossy(&output.stdout);
    if body.trim().is_empty() {
        return Err("Weather service returned empty response".to_string());
    }
    if body.contains("location not found") || body.contains("Unknown location") {
        return Err(format!(
            "Location '{}' not found",
            city.as_deref().unwrap_or("specified")
        ));
    }

    let local_hour = get_local_hour();
    let data = parse_wttr_json(&body, local_hour)?;

    // Update cache
    if city.is_none() {
        if let Ok(mut guard) = WEATHER_CACHE.lock() {
            *guard = Some((data.clone(), Instant::now()));
        }
    }

    Ok(data)
}

#[tauri::command]
pub fn fetch_weather(city: Option<String>, force: Option<bool>) -> Result<WeatherData, String> {
    get_weather(city, force.unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_time_period() {
        assert_eq!(determine_time_period(6), ("dawn", true));
        assert_eq!(determine_time_period(12), ("day", true));
        assert_eq!(determine_time_period(18), ("sunset", true));
        assert_eq!(determine_time_period(23), ("night", false));
        assert_eq!(determine_time_period(2), ("night", false));
    }

    #[test]
    fn test_map_weather_condition() {
        assert_eq!(map_weather_condition(353, "Light rain shower"), "rain");
        assert_eq!(map_weather_condition(200, "Thundery outbreaks"), "thunderstorm");
        assert_eq!(map_weather_condition(326, "Light snow"), "snow");
        assert_eq!(map_weather_condition(119, "Overcast"), "clouds");
        assert_eq!(map_weather_condition(113, "Sunny"), "clear");
        assert_eq!(map_weather_condition(143, "Mist"), "fog");
    }

    #[test]
    fn test_parse_wttr_json_mock() {
        let json = r#"{
            "current_condition": [{
                "temp_C": "28",
                "temp_F": "82",
                "weatherCode": "353",
                "weatherDesc": [{"value": "Light rain shower"}],
                "humidity": "72",
                "cloudcover": "93"
            }],
            "nearest_area": [{
                "areaName": [{"value": "Surabaya"}]
            }],
            "weather": [{
                "astronomy": [{
                    "sunrise": "05:03 AM",
                    "sunset": "05:23 PM"
                }]
            }]
        }"#;

        let w = parse_wttr_json(json, 21).unwrap();
        assert_eq!(w.temp_c, 28);
        assert_eq!(w.location, "Surabaya");
        assert_eq!(w.condition, "rain");
        assert_eq!(w.description, "Light rain shower");
        assert_eq!(w.time_period, "night");
        assert!(!w.is_day);
        assert_eq!(w.sunrise, "05:03 AM");
    }
}
