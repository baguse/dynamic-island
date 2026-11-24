use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const KAABA_LAT: f64 = 21.422487;
const KAABA_LON: f64 = 39.826206;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrayerTimeItem {
    pub name: String,
    pub arabic_name: String,
    pub label: String,
    pub time: String,
    pub timestamp: u64,
    pub is_next: bool,
    pub is_current: bool,
    pub is_passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextPrayerInfo {
    pub name: String,
    pub arabic_name: String,
    pub label: String,
    pub time: String,
    pub remaining_seconds: i64,
    pub formatted_countdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentPrayerInfo {
    pub name: String,
    pub label: String,
    pub time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrayerSchedule {
    pub city: String,
    pub country: String,
    pub date_gregorian: String,
    pub date_hijri: String,
    pub hijri_day: String,
    pub hijri_month: String,
    pub hijri_year: String,
    pub method_name: String,
    pub asr_school: String,
    pub times: HashMap<String, String>,
    pub items: Vec<PrayerTimeItem>,
    pub next_prayer: Option<NextPrayerInfo>,
    pub current_prayer: Option<CurrentPrayerInfo>,
    pub qibla_bearing: f64,
    pub updated_at: u64,
}

#[derive(Debug, Clone)]
pub struct CityGeo {
    pub city: &'static str,
    pub country: &'static str,
    pub lat: f64,
    pub lon: f64,
    pub tz: f64,
}

static KNOWN_CITIES: &[CityGeo] = &[
    CityGeo { city: "Jakarta", country: "Indonesia", lat: -6.2088, lon: 106.8456, tz: 7.0 },
    CityGeo { city: "Surabaya", country: "Indonesia", lat: -7.2575, lon: 112.7521, tz: 7.0 },
    CityGeo { city: "Bandung", country: "Indonesia", lat: -6.9175, lon: 107.6191, tz: 7.0 },
    CityGeo { city: "Medan", country: "Indonesia", lat: 3.5952, lon: 98.6722, tz: 7.0 },
    CityGeo { city: "Semarang", country: "Indonesia", lat: -6.9667, lon: 110.4167, tz: 7.0 },
    CityGeo { city: "Makassar", country: "Indonesia", lat: -5.1477, lon: 119.4327, tz: 8.0 },
    CityGeo { city: "Yogyakarta", country: "Indonesia", lat: -7.7956, lon: 110.3695, tz: 7.0 },
    CityGeo { city: "Makkah", country: "Saudi Arabia", lat: 21.4225, lon: 39.8262, tz: 3.0 },
    CityGeo { city: "Madinah", country: "Saudi Arabia", lat: 24.4672, lon: 39.6112, tz: 3.0 },
    CityGeo { city: "Riyadh", country: "Saudi Arabia", lat: 24.7136, lon: 46.6753, tz: 3.0 },
    CityGeo { city: "Dubai", country: "United Arab Emirates", lat: 25.2048, lon: 55.2708, tz: 4.0 },
    CityGeo { city: "Cairo", country: "Egypt", lat: 30.0444, lon: 31.2357, tz: 2.0 },
    CityGeo { city: "Istanbul", country: "Turkey", lat: 41.0082, lon: 28.9784, tz: 3.0 },
    CityGeo { city: "Kuala Lumpur", country: "Malaysia", lat: 3.1390, lon: 101.6869, tz: 8.0 },
    CityGeo { city: "Singapore", country: "Singapore", lat: 1.3521, lon: 103.8198, tz: 8.0 },
    CityGeo { city: "London", country: "United Kingdom", lat: 51.5074, lon: -0.1278, tz: 0.0 },
    CityGeo { city: "New York", country: "United States", lat: 40.7128, lon: -74.0060, tz: -5.0 },
    CityGeo { city: "Tokyo", country: "Japan", lat: 35.6762, lon: 139.6503, tz: 9.0 },
];

pub fn find_city_geo(city_name: &str) -> CityGeo {
    let lower = city_name.trim().to_lowercase();
    for c in KNOWN_CITIES {
        if lower.contains(&c.city.to_lowercase()) {
            return c.clone();
        }
    }
    // Default to Surabaya/Indonesia or Jakarta if matching Indonesian cities
    if lower.contains("indonesia") || lower.contains("jawa") {
        return CityGeo { city: "Jakarta", country: "Indonesia", lat: -6.2088, lon: 106.8456, tz: 7.0 };
    }
    CityGeo { city: "Jakarta", country: "Indonesia", lat: -6.2088, lon: 106.8456, tz: 7.0 }
}

#[derive(Debug, Clone)]
pub struct MethodParams {
    pub name: &'static str,
    pub fajr_angle: f64,
    pub isha_angle: f64,
    pub isha_fixed_minutes: Option<f64>,
    pub aladhan_id: u32,
}

pub fn get_method_params(method: &str) -> MethodParams {
    match method.to_uppercase().as_str() {
        "MWL" | "MUSLIM_WORLD_LEAGUE" => MethodParams {
            name: "Muslim World League",
            fajr_angle: 18.0,
            isha_angle: 17.0,
            isha_fixed_minutes: None,
            aladhan_id: 3,
        },
        "ISNA" | "NORTH_AMERICA" => MethodParams {
            name: "Islamic Society of North America (ISNA)",
            fajr_angle: 15.0,
            isha_angle: 15.0,
            isha_fixed_minutes: None,
            aladhan_id: 2,
        },
        "EGYPT" | "EGYPTIAN" => MethodParams {
            name: "Egyptian General Authority of Survey",
            fajr_angle: 19.5,
            isha_angle: 17.5,
            isha_fixed_minutes: None,
            aladhan_id: 5,
        },
        "MAKKAH" | "UMM_AL_QURA" => MethodParams {
            name: "Umm Al-Qura University, Makkah",
            fajr_angle: 18.5,
            isha_angle: 0.0,
            isha_fixed_minutes: Some(90.0),
            aladhan_id: 4,
        },
        "KARACHI" => MethodParams {
            name: "University of Islamic Sciences, Karachi",
            fajr_angle: 18.0,
            isha_angle: 18.0,
            isha_fixed_minutes: None,
            aladhan_id: 1,
        },
        "TEHRAN" => MethodParams {
            name: "Institute of Geophysics, University of Tehran",
            fajr_angle: 17.7,
            isha_angle: 14.0,
            isha_fixed_minutes: None,
            aladhan_id: 7,
        },
        "GULF" | "DUBAI" => MethodParams {
            name: "Gulf / Dubai",
            fajr_angle: 18.2,
            isha_angle: 0.0,
            isha_fixed_minutes: Some(90.0),
            aladhan_id: 8,
        },
        _ => MethodParams {
            name: "Kementerian Agama Republik Indonesia (KEMENAG)",
            fajr_angle: 20.0,
            isha_angle: 18.0,
            isha_fixed_minutes: None,
            aladhan_id: 11, // Majlis Ugama Islam Singapura / Kemenag style
        },
    }
}

/// Calculate Great Circle Qibla direction in degrees from true North
pub fn calculate_qibla_bearing(lat: f64, lon: f64) -> f64 {
    let lat1 = lat.to_radians();
    let lat2 = KAABA_LAT.to_radians();
    let dlon = (KAABA_LON - lon).to_radians();

    let y = dlon.sin();
    let x = lat1.cos() * lat2.tan() - lat1.sin() * dlon.cos();
    let mut bearing = y.atan2(x).to_degrees();
    if bearing < 0.0 {
        bearing += 360.0;
    }
    (bearing * 10.0).round() / 10.0
}

/// Convert Julian Day to Hijri (Day, Month, Year)
pub fn julian_to_hijri(jd: f64) -> (u32, &'static str, &'static str, u32) {
    let l = jd as i64 - 1948440 + 10632;
    let n = (l - 1) / 10631;
    let l = l - 10631 * n + 354;
    let j = ((10985 - l) / 5316) * ((50 * l) / 17719) + (l / 5670) * ((43 * l) / 15238);
    let l = l - ((30 - j) / 15) * ((17719 * j) / 50) - (j / 16) * ((15238 * j) / 43) + 29;
    let m = (24 * l) / 709;
    let d = l - (709 * m) / 24;
    let y = 30 * n + j - 30;

    let (en_month, ar_month) = match m {
        1 => ("Muharram", "مُحَرَّم"),
        2 => ("Safar", "صَفَر"),
        3 => ("Rabi' al-Awwal", "رَبِيع ٱلْأَوَّل"),
        4 => ("Rabi' al-Thani", "رَبِيع ٱلْآخِر"),
        5 => ("Jumada al-Ula", "جُمَادَىٰ ٱلْأُولَىٰ"),
        6 => ("Jumada al-Akhirah", "جُمَادَىٰ ٱلْآخِرَة"),
        7 => ("Rajab", "رَجَب"),
        8 => ("Sha'ban", "شَعْبَان"),
        9 => ("Ramadan", "رَمَضَان"),
        10 => ("Shawwal", "شَوَّال"),
        11 => ("Dhu al-Qi'dah", "ذُو ٱلْقَعْدَة"),
        _ => ("Dhu al-Hijjah", "ذُو ٱلْحِجَّة"),
    };

    (d.max(1) as u32, en_month, ar_month, y.max(1) as u32)
}

/// Astronomical Solar Calculations
struct SolarTimes {
    fajr: f64,
    sunrise: f64,
    dhuhr: f64,
    asr: f64,
    maghrib: f64,
    isha: f64,
    imsak: f64,
}

fn calculate_astronomical_times(
    year: i32,
    month: u32,
    day: u32,
    lat: f64,
    lon: f64,
    tz: f64,
    method: &MethodParams,
    asr_hanafi: bool,
) -> SolarTimes {
    let a = (14 - month as i32) / 12;
    let y = year + 4800 - a;
    let m = month as i32 + 12 * a - 3;
    let jd = day as f64 + ((153 * m + 2) / 5) as f64 + (365 * y) as f64 + (y / 4) as f64 - (y / 100) as f64
        + (y / 400) as f64
        - 32045.0
        - 0.5;

    let d = jd - 2451545.0;
    let g = (357.529 + 0.98560028 * d) % 360.0;
    let q = (280.459 + 0.98564736 * d) % 360.0;
    let l = (q + 1.915 * g.to_radians().sin() + 0.020 * (2.0 * g).to_radians().sin()) % 360.0;
    let e = 23.439 - 0.00000036 * d;

    let delta = (e.to_radians().sin() * l.to_radians().sin()).asin();
    let ra = (e.to_radians().cos() * l.to_radians().sin()).atan2(l.to_radians().cos());
    let ra_deg = (ra.to_degrees() + 360.0) % 360.0;

    let eq_t = (q / 15.0 - ra_deg / 15.0) * 60.0; // minutes
    let noon = 12.0 + tz - (lon / 15.0) - (eq_t / 60.0);

    let lat_rad = lat.to_radians();

    let hour_angle = |alt_deg: f64| -> f64 {
        let alt_rad = alt_deg.to_radians();
        let cos_h = (alt_rad.sin() - lat_rad.sin() * delta.sin()) / (lat_rad.cos() * delta.cos());
        let clamped = cos_h.clamp(-1.0, 1.0);
        clamped.acos().to_degrees() / 15.0
    };

    // Sun altitude angles
    let sunrise_ha = hour_angle(-0.833);
    let sunrise = noon - sunrise_ha;
    let sunset = noon + sunrise_ha;

    let fajr_ha = hour_angle(-method.fajr_angle);
    let fajr = noon - fajr_ha;

    let isha = if let Some(mins) = method.isha_fixed_minutes {
        sunset + (mins / 60.0)
    } else {
        let isha_ha = hour_angle(-method.isha_angle);
        noon + isha_ha
    };

    // Asr calculation
    let shadow_factor = if asr_hanafi { 2.0 } else { 1.0 };
    let asr_alt = (1.0 / (shadow_factor + (lat_rad - delta).abs().tan())).atan().to_degrees();
    let asr_ha = hour_angle(asr_alt);
    let asr = noon + asr_ha;

    let imsak = fajr - (10.0 / 60.0); // 10 minutes before fajr

    SolarTimes {
        imsak,
        fajr,
        sunrise,
        dhuhr: noon,
        asr,
        maghrib: sunset,
        isha,
    }
}

fn format_hour_minute(h_float: f64) -> String {
    let mut total_mins = (h_float * 60.0).round() as i64;
    while total_mins < 0 {
        total_mins += 1440;
    }
    total_mins %= 1440;
    let hour = total_mins / 60;
    let min = total_mins % 60;
    format!("{:02}:{:02}", hour, min)
}

fn get_today_ymd() -> (i32, u32, u32) {
    if let Ok(out) = Command::new("date").arg("+%Y-%m-%d").output() {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() == 3 {
            if let (Ok(y), Ok(m), Ok(d)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                return (y, m, d);
            }
        }
    }
    (2026, 11, 24)
}

fn get_current_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn get_today_start_epoch(y: i32, m: u32, d: u32) -> u64 {
    let date_str = format!("{:04}-{:02}-{:02} 00:00:00", y, m, d);
    if let Ok(out) = Command::new("date").args(["-d", &date_str, "+%s"]).output() {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if let Ok(epoch_s) = s.parse::<u64>() {
            return epoch_s * 1000;
        }
    }
    // Fallback approximation
    (get_current_epoch_ms() / 86_400_000) * 86_400_000
}

fn parse_time_to_epoch(base_start_ms: u64, time_str: &str) -> u64 {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() >= 2 {
        let h = parts[0].trim().parse::<u64>().unwrap_or(0);
        let m = parts[1].trim().parse::<u64>().unwrap_or(0);
        return base_start_ms + (h * 3600 + m * 60) * 1000;
    }
    base_start_ms
}

static PRAYER_CACHE: Mutex<Option<(PrayerSchedule, Instant)>> = Mutex::new(None);
const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour memory TTL

/// Calculate full prayer schedule with offline calculation and optional Aladhan sync
pub fn get_prayer_schedule(
    city_param: Option<String>,
    method_param: Option<String>,
    asr_school_param: Option<String>,
    force: bool,
) -> Result<PrayerSchedule, String> {
    let city = city_param.unwrap_or_else(|| "Surabaya".to_string());
    let method_key = method_param.unwrap_or_else(|| "KEMENAG".to_string());
    let asr_school = asr_school_param.unwrap_or_else(|| "standard".to_string());
    let asr_hanafi = asr_school.to_lowercase() == "hanafi";

    let (year, month, day) = get_today_ymd();
    let geo = find_city_geo(&city);
    let method = get_method_params(&method_key);
    let qibla_bearing = calculate_qibla_bearing(geo.lat, geo.lon);

    let base_start_ms = get_today_start_epoch(year, month, day);
    let now_ms = get_current_epoch_ms();

    // Check memory cache
    if !force {
        if let Ok(guard) = PRAYER_CACHE.lock() {
            if let Some((ref cached, ref instant)) = *guard {
                if instant.elapsed() < CACHE_TTL && cached.city == geo.city {
                    return Ok(update_next_prayer(cached.clone(), now_ms));
                }
            }
        }
    }

    // Try fetching from Aladhan API if online (with 3-second timeout)
    let mut api_times: Option<HashMap<String, String>> = None;
    let mut api_hijri: Option<(String, String, String, String)> = None;

    let api_url = format!(
        "https://api.aladhan.com/v1/timingsByCity?city={}&country={}&method={}",
        geo.city.replace(' ', "%20"),
        geo.country.replace(' ', "%20"),
        method.aladhan_id
    );

    if let Ok(output) = Command::new("curl")
        .args(["-s", "--max-time", "3", &api_url])
        .output()
    {
        if output.status.success() {
            let body = String::from_utf8_lossy(&output.stdout);
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(data) = v.get("data") {
                    if let Some(timings) = data.get("timings").and_then(|t| t.as_object()) {
                        let mut map = HashMap::new();
                        for (k, val) in timings {
                            if let Some(s) = val.as_str() {
                                // Strip timezone suffix if present (e.g. "04:15 (WIB)" -> "04:15")
                                let clean = s.split(' ').next().unwrap_or(s).to_string();
                                map.insert(k.clone(), clean);
                            }
                        }
                        api_times = Some(map);
                    }

                    if let Some(hijri) = data.get("date").and_then(|d| d.get("hijri")) {
                        let h_day = hijri.get("day").and_then(|d| d.as_str()).unwrap_or("14").to_string();
                        let h_year = hijri.get("year").and_then(|y| y.as_str()).unwrap_or("1448").to_string();
                        let en_month = hijri
                            .get("month")
                            .and_then(|m| m.get("en"))
                            .and_then(|s| s.as_str())
                            .unwrap_or("Sha'ban")
                            .to_string();
                        let ar_month = hijri
                            .get("month")
                            .and_then(|m| m.get("ar"))
                            .and_then(|s| s.as_str())
                            .unwrap_or("شَعْبَان")
                            .to_string();
                        api_hijri = Some((h_day, en_month, ar_month, h_year));
                    }
                }
            }
        }
    }

    // Fallback: Local Astronomical Calculation
    let solar = calculate_astronomical_times(year, month, day, geo.lat, geo.lon, geo.tz, &method, asr_hanafi);

    let (times_map, hijri_day, hijri_month, hijri_year) = match (api_times, api_hijri) {
        (Some(times), Some((h_d, h_m, _h_ar, h_y))) => (times, h_d, h_m, h_y),
        _ => {
            let mut map = HashMap::new();
            map.insert("Imsak".to_string(), format_hour_minute(solar.imsak));
            map.insert("Fajr".to_string(), format_hour_minute(solar.fajr));
            map.insert("Sunrise".to_string(), format_hour_minute(solar.sunrise));
            map.insert("Dhuhr".to_string(), format_hour_minute(solar.dhuhr));
            map.insert("Asr".to_string(), format_hour_minute(solar.asr));
            map.insert("Maghrib".to_string(), format_hour_minute(solar.maghrib));
            map.insert("Isha".to_string(), format_hour_minute(solar.isha));

            let (h_d, h_m, _, h_y) = julian_to_hijri(2451545.0 + (get_current_epoch_ms() as f64 / 86400000.0));
            (map, h_d.to_string(), h_m.to_string(), h_y.to_string())
        }
    };

    let prayer_definitions = [
        ("Imsak", "الإمساك", "Imsak", "Imsak"),
        ("Fajr", "الفجر", "Fajr (Subuh)", "Fajr"),
        ("Sunrise", "الشروق", "Sunrise (Syuruq)", "Sunrise"),
        ("Dhuhr", "الظهر", "Dhuhr (Dzuhur)", "Dhuhr"),
        ("Asr", "العصر", "Asr (Ashar)", "Asr"),
        ("Maghrib", "المغرب", "Maghrib", "Maghrib"),
        ("Isha", "العشاء", "Isha (Isya)", "Isha"),
    ];

    let mut items = Vec::new();
    for (name, ar, label, key) in prayer_definitions {
        let time_str = times_map
            .get(key)
            .cloned()
            .unwrap_or_else(|| "12:00".to_string());
        let epoch = parse_time_to_epoch(base_start_ms, &time_str);

        items.push(PrayerTimeItem {
            name: name.to_string(),
            arabic_name: ar.to_string(),
            label: label.to_string(),
            time: time_str,
            timestamp: epoch,
            is_next: false,
            is_current: false,
            is_passed: epoch < now_ms,
        });
    }

    let date_gregorian = format!("{:04}-{:02}-{:02}", year, month, day);
    let date_hijri = format!("{} {} {} AH", hijri_day, hijri_month, hijri_year);

    let schedule = PrayerSchedule {
        city: geo.city.to_string(),
        country: geo.country.to_string(),
        date_gregorian,
        date_hijri,
        hijri_day,
        hijri_month,
        hijri_year,
        method_name: method.name.to_string(),
        asr_school: if asr_hanafi { "Hanafi".to_string() } else { "Standard (Shafi'i/Maliki/Hanbali)".to_string() },
        times: times_map,
        items,
        next_prayer: None,
        current_prayer: None,
        qibla_bearing,
        updated_at: now_ms,
    };

    let updated = update_next_prayer(schedule, now_ms);

    if let Ok(mut guard) = PRAYER_CACHE.lock() {
        *guard = Some((updated.clone(), Instant::now()));
    }

    Ok(updated)
}

fn update_next_prayer(mut schedule: PrayerSchedule, now_ms: u64) -> PrayerSchedule {
    // Exclude Sunrise and Imsak from major obligatory next prayer notification, but keep in timeline
    let obligatory_prayers = ["Fajr", "Dhuhr", "Asr", "Maghrib", "Isha"];

    let mut next_item: Option<PrayerTimeItem> = None;
    let mut current_item: Option<PrayerTimeItem> = None;

    for item in &mut schedule.items {
        item.is_passed = item.timestamp <= now_ms;
        if item.timestamp <= now_ms && obligatory_prayers.contains(&item.name.as_str()) {
            current_item = Some(item.clone());
        }
        if item.timestamp > now_ms && obligatory_prayers.contains(&item.name.as_str()) && next_item.is_none() {
            next_item = Some(item.clone());
        }
    }

    // Set is_next and is_current flags
    if let Some(ref n) = next_item {
        for item in &mut schedule.items {
            if item.name == n.name {
                item.is_next = true;
            }
        }
    }

    if let Some(ref c) = current_item {
        for item in &mut schedule.items {
            if item.name == c.name {
                item.is_current = true;
            }
        }
        schedule.current_prayer = Some(CurrentPrayerInfo {
            name: c.name.clone(),
            label: c.label.clone(),
            time: c.time.clone(),
        });
    }

    if let Some(ref n) = next_item {
        let diff_ms = (n.timestamp as i64) - (now_ms as i64);
        let remaining_secs = (diff_ms / 1000).max(0);
        let hours = remaining_secs / 3600;
        let mins = (remaining_secs % 3600) / 60;
        let secs = remaining_secs % 60;

        let formatted_countdown = if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, mins, secs)
        } else {
            format!("{:02}:{:02}", mins, secs)
        };

        schedule.next_prayer = Some(NextPrayerInfo {
            name: n.name.clone(),
            arabic_name: n.arabic_name.clone(),
            label: n.label.clone(),
            time: n.time.clone(),
            remaining_seconds: remaining_secs,
            formatted_countdown,
        });
    } else {
        // All prayers passed for today -> Next is tomorrow's Fajr
        schedule.next_prayer = Some(NextPrayerInfo {
            name: "Fajr".to_string(),
            arabic_name: "الفجر".to_string(),
            label: "Fajr (Subuh)".to_string(),
            time: schedule.times.get("Fajr").cloned().unwrap_or_else(|| "04:15".to_string()),
            remaining_seconds: 0,
            formatted_countdown: "Tomorrow".to_string(),
        });
    }

    schedule
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qibla_bearing_jakarta() {
        let bearing = calculate_qibla_bearing(-6.2088, 106.8456);
        // Jakarta to Kaaba is ~295° NW
        assert!((bearing - 295.0).abs() < 2.0, "Expected ~295, got {}", bearing);
    }

    #[test]
    fn test_astronomical_calculation() {
        let method = get_method_params("KEMENAG");
        let times = calculate_astronomical_times(2026, 11, 24, -7.2575, 112.7521, 7.0, &method, false);

        assert!(times.fajr < times.sunrise);
        assert!(times.sunrise < times.dhuhr);
        assert!(times.dhuhr < times.asr);
        assert!(times.asr < times.maghrib);
        assert!(times.maghrib < times.isha);
    }

    #[test]
    fn test_get_prayer_schedule() {
        let sched = get_prayer_schedule(Some("Surabaya".to_string()), Some("KEMENAG".to_string()), None, true)
            .expect("Prayer schedule should generate");

        assert_eq!(sched.city, "Surabaya");
        assert!(sched.times.contains_key("Fajr"));
        assert!(sched.times.contains_key("Maghrib"));
        assert!(sched.items.len() >= 5);
        assert!(sched.qibla_bearing > 0.0);
    }
}
