use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortInfo {
    pub port: u16,
    pub ip: String,
    pub process: Option<String>,
    pub pid: Option<u32>,
    pub container: Option<String>,
    pub category: String, // "web", "database", "service", "ide", "system"
    pub is_openable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DockerContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String, // "running", "exited", "created", etc.
    pub status: String,
    pub ports: String,
    pub is_running: bool,
}

#[derive(Debug, Deserialize)]
struct DockerJsonItem {
    #[serde(rename = "ID")]
    id: Option<String>,
    #[serde(rename = "Names")]
    names: Option<String>,
    #[serde(rename = "Image")]
    image: Option<String>,
    #[serde(rename = "State")]
    state: Option<String>,
    #[serde(rename = "Status")]
    status: Option<String>,
    #[serde(rename = "Ports")]
    ports: Option<String>,
}

/// Fetch Docker containers list via `docker ps -a`
pub fn get_docker_containers() -> Vec<DockerContainerInfo> {
    let output = match Command::new("docker")
        .args(["ps", "-a", "--format", "{{json .}}"])
        .output()
    {
        Ok(out) if out.status.success() => out.stdout,
        _ => return Vec::new(),
    };

    let text = String::from_utf8_lossy(&output);
    let mut containers = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Ok(item) = serde_json::from_str::<DockerJsonItem>(trimmed) {
            let id = item.id.unwrap_or_default();
            let name = item.names.unwrap_or_default();
            let image = item.image.unwrap_or_default();
            let state = item.state.unwrap_or_else(|| "unknown".to_string()).to_lowercase();
            let status = item.status.unwrap_or_default();
            let ports = item.ports.unwrap_or_default();
            let is_running = state == "running";

            containers.push(DockerContainerInfo {
                id,
                name,
                image,
                state,
                status,
                ports,
                is_running,
            });
        }
    }

    // Sort: running containers first, then alphabetically by name
    containers.sort_by(|a, b| {
        b.is_running
            .cmp(&a.is_running)
            .then_with(|| a.name.cmp(&b.name))
    });

    containers
}

/// Extract port mappings from running Docker containers -> { port: container_name }
pub fn get_docker_port_mappings() -> HashMap<u16, String> {
    let mut map = HashMap::new();
    let containers = get_docker_containers();

    for c in containers {
        if !c.is_running {
            continue;
        }
        // Ports look like: "0.0.0.0:6545->5432/tcp, :::6545->5432/tcp, 9443/tcp"
        for part in c.ports.split(',') {
            if let Some(arrow_idx) = part.find("->") {
                let left = &part[..arrow_idx];
                if let Some(colon_idx) = left.rfind(':') {
                    if let Ok(p) = left[colon_idx + 1..].trim().parse::<u16>() {
                        map.insert(p, c.name.clone());
                    }
                }
            }
        }
    }

    map
}

/// Query listening TCP ports on Linux using `ss -tlpn -H`
pub fn get_listening_ports() -> Vec<PortInfo> {
    let docker_map = get_docker_port_mappings();

    let output = match Command::new("ss")
        .args(["-tlpn", "-H"])
        .output()
    {
        Ok(out) if out.status.success() => out.stdout,
        _ => return Vec::new(),
    };

    let text = String::from_utf8_lossy(&output);
    let mut ports: Vec<PortInfo> = Vec::new();
    let mut seen_ports = std::collections::HashSet::new();

    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        // Local address is in column 3 (0-indexed)
        let local_addr = parts[3];
        let port_str = match local_addr.rfind(':') {
            Some(idx) => &local_addr[idx + 1..],
            None => continue,
        };

        let port: u16 = match port_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        if !seen_ports.insert(port) {
            continue;
        }

        let ip = if let Some(idx) = local_addr.rfind(':') {
            local_addr[..idx].replace('[', "").replace(']', "")
        } else {
            "0.0.0.0".to_string()
        };

        // Extract process name and PID from column 5+ (e.g. users:(("node",pid=2434002,fd=53)))
        let mut proc_name: Option<String> = None;
        let mut pid_val: Option<u32> = None;

        if parts.len() >= 6 {
            let proc_str = parts[5..].join(" ");
            if let Some(start_quote) = proc_str.find('"') {
                if let Some(end_quote) = proc_str[start_quote + 1..].find('"') {
                    let name = &proc_str[start_quote + 1..start_quote + 1 + end_quote];
                    proc_name = Some(name.to_string());
                }
            }

            if let Some(pid_idx) = proc_str.find("pid=") {
                let rest = &proc_str[pid_idx + 4..];
                let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(pid) = num_str.parse::<u32>() {
                    pid_val = Some(pid);
                }
            }
        }

        let container = docker_map.get(&port).cloned();

        // Categorize
        let category = classify_port(port, proc_name.as_deref(), container.as_deref());
        let is_openable = category == "web"
            || port == 80
            || port == 443
            || (port >= 3000 && port <= 3100)
            || (port >= 5000 && port <= 5200)
            || (port >= 8000 && port <= 9100)
            || port == 1420;

        ports.push(PortInfo {
            port,
            ip,
            process: proc_name,
            pid: pid_val,
            container,
            category,
            is_openable,
        });
    }

    // Sort order:
    // 1. web
    // 2. database
    // 3. service
    // 4. ide
    // 5. system
    ports.sort_by(|a, b| {
        let cat_score = |cat: &str| match cat {
            "web" => 1,
            "database" => 2,
            "service" => 3,
            "ide" => 4,
            _ => 5,
        };
        cat_score(&a.category)
            .cmp(&cat_score(&b.category))
            .then_with(|| a.port.cmp(&b.port))
    });

    ports
}

fn classify_port(port: u16, proc: Option<&str>, container: Option<&str>) -> String {
    let p_lower = proc.unwrap_or("").to_lowercase();
    let c_lower = container.unwrap_or("").to_lowercase();

    // Check database
    if p_lower.contains("postgres")
        || p_lower.contains("mysql")
        || p_lower.contains("mariadb")
        || p_lower.contains("redis")
        || p_lower.contains("mongo")
        || c_lower.contains("postgres")
        || c_lower.contains("mysql")
        || c_lower.contains("mariadb")
        || c_lower.contains("redis")
        || c_lower.contains("mongo")
        || matches!(port, 3306..=3309 | 5432 | 6379..=6400 | 6543..=6547 | 7654 | 27017..=27019)
    {
        return "database".to_string();
    }

    // Check web dev servers
    if p_lower.contains("node")
        || p_lower.contains("vite")
        || p_lower.contains("next")
        || p_lower.contains("caddy")
        || p_lower.contains("nginx")
        || p_lower.contains("python")
        || p_lower.contains("uvicorn")
        || p_lower.contains("flask")
        || p_lower.contains("deno")
        || p_lower.contains("bun")
        || c_lower.contains("portainer")
        || c_lower.contains("seatable")
        || c_lower.contains("web")
        || c_lower.contains("app")
        || matches!(port, 80 | 443 | 1420 | 3000..=3100 | 4200..=4300 | 5000..=5200 | 8000..=8100 | 8880..=9000)
    {
        return "web".to_string();
    }

    // Check IDE & language servers
    if p_lower.contains("cursor")
        || p_lower.contains("antigravity")
        || p_lower.contains("language_server")
        || p_lower.contains("code")
    {
        return "ide".to_string();
    }

    // Check other developer services
    if p_lower.contains("adb")
        || p_lower.contains("discord")
        || p_lower.contains("cloudflared")
        || c_lower.contains("bot")
        || c_lower.contains("floci")
        || c_lower.contains("sandbox")
        || port == 11434 // Ollama
    {
        return "service".to_string();
    }

    "system".to_string()
}

/// Kill a process by PID
pub fn kill_process_by_pid(pid: u32) -> Result<(), String> {
    if pid <= 1 {
        return Err("Cannot terminate root system process".to_string());
    }

    // Use SIGTERM (-15) then fallback to SIGKILL (-9)
    let output = Command::new("kill")
        .args(["-9", &pid.to_string()])
        .output()
        .map_err(|e| format!("Failed to execute kill: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Restart Docker container
pub fn docker_restart(id: &str) -> Result<(), String> {
    let output = Command::new("docker")
        .args(["restart", "-t", "3", id])
        .output()
        .map_err(|e| format!("Failed to restart Docker container: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Stop Docker container
pub fn docker_stop(id: &str) -> Result<(), String> {
    let output = Command::new("docker")
        .args(["stop", "-t", "3", id])
        .output()
        .map_err(|e| format!("Failed to stop Docker container: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Start Docker container
pub fn docker_start(id: &str) -> Result<(), String> {
    let output = Command::new("docker")
        .args(["start", id])
        .output()
        .map_err(|e| format!("Failed to start Docker container: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Open localhost URL in user's default browser
pub fn open_browser_url(url: &str) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map_err(|e| format!("Failed to launch browser: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_port() {
        assert_eq!(classify_port(5173, Some("vite"), None), "web");
        assert_eq!(classify_port(3000, Some("node"), None), "web");
        assert_eq!(classify_port(5432, None, Some("postgres_16")), "database");
        assert_eq!(classify_port(6379, Some("redis-server"), None), "database");
        assert_eq!(classify_port(11434, None, None), "service");
        assert_eq!(classify_port(9000, None, Some("portainer")), "web");
    }

    #[test]
    fn test_get_listening_ports_does_not_panic() {
        let ports = get_listening_ports();
        // Should successfully return vector of detected ports
        println!("Detected {} ports", ports.len());
    }

    #[test]
    fn test_get_docker_containers_does_not_panic() {
        let containers = get_docker_containers();
        println!("Detected {} containers", containers.len());
    }
}

