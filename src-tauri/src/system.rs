use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryTelemetry {
    pub percent: u8,
    pub status: String,
    pub is_charging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTelemetry {
    pub cpu_percent: f32,
    pub core_count: usize,
    pub mem_used_mb: u64,
    pub mem_total_mb: u64,
    pub mem_percent: f32,
    pub battery: Option<BatteryTelemetry>,
    pub net_rx_speed_kb: f64,
    pub net_tx_speed_kb: f64,
}

pub struct SystemTracker {
    last_cpu_idle: u64,
    last_cpu_total: u64,
    last_net_rx: u64,
    last_net_tx: u64,
    last_sample_time: Instant,
    core_count: usize,
}

impl SystemTracker {
    pub fn new() -> Self {
        let core_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        let (cpu_idle, cpu_total) = Self::read_cpu_ticks().unwrap_or((0, 0));
        let (net_rx, net_tx) = Self::read_network_bytes().unwrap_or((0, 0));

        Self {
            last_cpu_idle: cpu_idle,
            last_cpu_total: cpu_total,
            last_net_rx: net_rx,
            last_net_tx: net_tx,
            last_sample_time: Instant::now(),
            core_count,
        }
    }

    pub fn sample(&mut self) -> SystemTelemetry {
        let now = Instant::now();
        let elapsed_secs = now.duration_since(self.last_sample_time).as_secs_f64().max(0.1);

        // 1. CPU Usage
        let mut cpu_percent = 0.0f32;
        if let Ok((curr_idle, curr_total)) = Self::read_cpu_ticks() {
            let total_delta = curr_total.saturating_sub(self.last_cpu_total);
            let idle_delta = curr_idle.saturating_sub(self.last_cpu_idle);

            if total_delta > 0 {
                let usage = 1.0 - (idle_delta as f64 / total_delta as f64);
                cpu_percent = (usage * 100.0).clamp(0.0, 100.0) as f32;
            }

            self.last_cpu_idle = curr_idle;
            self.last_cpu_total = curr_total;
        }

        // 2. Memory Usage
        let (mem_used_mb, mem_total_mb, mem_percent) = Self::read_memory().unwrap_or((0, 0, 0.0));

        // 3. Battery
        let battery = Self::read_battery();

        // 4. Network Throughput
        let mut net_rx_speed_kb = 0.0;
        let mut net_tx_speed_kb = 0.0;
        if let Ok((curr_rx, curr_tx)) = Self::read_network_bytes() {
            let rx_delta = curr_rx.saturating_sub(self.last_net_rx);
            let tx_delta = curr_tx.saturating_sub(self.last_net_tx);

            net_rx_speed_kb = (rx_delta as f64 / 1024.0) / elapsed_secs;
            net_tx_speed_kb = (tx_delta as f64 / 1024.0) / elapsed_secs;

            self.last_net_rx = curr_rx;
            self.last_net_tx = curr_tx;
        }

        self.last_sample_time = now;

        SystemTelemetry {
            cpu_percent,
            core_count: self.core_count,
            mem_used_mb,
            mem_total_mb,
            mem_percent,
            battery,
            net_rx_speed_kb,
            net_tx_speed_kb,
        }
    }

    fn read_cpu_ticks() -> Result<(u64, u64), String> {
        let content = fs::read_to_string("/proc/stat").map_err(|e| e.to_string())?;
        for line in content.lines() {
            if line.starts_with("cpu ") {
                let parts: Vec<u64> = line
                    .split_whitespace()
                    .skip(1)
                    .filter_map(|s| s.parse::<u64>().ok())
                    .collect();

                if parts.len() >= 4 {
                    // user, nice, system, idle, iowait, irq, softirq, steal
                    let idle = parts[3] + parts.get(4).unwrap_or(&0);
                    let total: u64 = parts.iter().sum();
                    return Ok((idle, total));
                }
            }
        }
        Err("Failed to parse /proc/stat".to_string())
    }

    fn read_memory() -> Result<(u64, u64, f32), String> {
        let content = fs::read_to_string("/proc/meminfo").map_err(|e| e.to_string())?;
        let mut total_kb = 0u64;
        let mut avail_kb = 0u64;

        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                total_kb = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                avail_kb = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
            }
        }

        if total_kb > 0 {
            let used_kb = total_kb.saturating_sub(avail_kb);
            let total_mb = total_kb / 1024;
            let used_mb = used_kb / 1024;
            let percent = (used_kb as f32 / total_kb as f32) * 100.0;
            return Ok((used_mb, total_mb, percent));
        }

        Err("Failed to parse /proc/meminfo".to_string())
    }

    fn read_battery() -> Option<BatteryTelemetry> {
        let bat_dirs = ["/sys/class/power_supply/BAT0", "/sys/class/power_supply/BAT1"];
        for dir in bat_dirs {
            let p = Path::new(dir);
            if p.exists() {
                let cap_file = p.join("capacity");
                let status_file = p.join("status");

                let percent = fs::read_to_string(cap_file)
                    .ok()?
                    .trim()
                    .parse::<u8>()
                    .unwrap_or(0);

                let status = fs::read_to_string(status_file)
                    .ok()?
                    .trim()
                    .to_string();

                let is_charging = status.eq_ignore_ascii_case("charging")
                    || status.eq_ignore_ascii_case("full");

                return Some(BatteryTelemetry {
                    percent,
                    status,
                    is_charging,
                });
            }
        }
        None
    }

    fn read_network_bytes() -> Result<(u64, u64), String> {
        let content = fs::read_to_string("/proc/net/dev").map_err(|e| e.to_string())?;
        let mut total_rx = 0u64;
        let mut total_tx = 0u64;

        for line in content.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                let iface = parts[0].trim_end_matches(':');
                if iface == "lo" {
                    continue; // Skip loopback
                }

                let rx = parts[1].parse::<u64>().unwrap_or(0);
                let tx = parts[9].parse::<u64>().unwrap_or(0);
                total_rx += rx;
                total_tx += tx;
            }
        }

        Ok((total_rx, total_tx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_tracker_sample() {
        let mut tracker = SystemTracker::new();
        assert!(tracker.core_count > 0);

        let telemetry = tracker.sample();
        assert!(telemetry.core_count > 0);
        assert!(telemetry.cpu_percent >= 0.0 && telemetry.cpu_percent <= 100.0);
        assert!(telemetry.mem_total_mb > 0);
        assert!(telemetry.mem_percent >= 0.0 && telemetry.mem_percent <= 100.0);
    }
}
