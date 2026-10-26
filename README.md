# Linux Dynamic Island

A lightweight, floating desktop widget for Linux inspired by Apple's Dynamic Island, engineered with **Tauri 2**, **Svelte 5**, **TypeScript**, and **Rust**. The initial target is **Ubuntu Cinnamon running on X11**.

---

## 1. Architecture & Features

### Desktop Shell & Native Windowing (Tauri 2 + Rust)
- **Isolated Window Coordinator:** Dynamic window sizing and positioning via `resize_and_center_island` in Rust ([`src-tauri/src/lib.rs`](file:///myproject/project/personal/dynamic-island/src-tauri/src/lib.rs)).
- **Persistent Multi-Monitor Support:**
  - Enumerates all connected displays and their geometries via `get_available_monitors`.
  - Persists the active target monitor in Rust (`AppState.current_monitor_name`), preventing the window from snapping back to the primary display when expanding/collapsing.
  - Supports one-click cycling to the next connected display (`cycle_next_monitor`) or direct switching by screen name (`move_island_to_monitor_by_name`).
  - Correctly calculates physical offsets for multi-monitor setups with non-`(0,0)` origins and HiDPI scaling.
- **Window Flags:** Frameless (`decorations: false`), transparent background (`transparent: true`), `alwaysOnTop: true`, `skipTaskbar: true`, and `visibleOnAllWorkspaces: true`.

### Hover-to-Expand & Focus-Lost Auto-Minimize
- **Hover to Expand:** Hovering your mouse cursor over the compact pill for **220ms** (hover intent delay) automatically expands the island without needing a click.
- **Leave to Collapse:** When your mouse pointer leaves the expanded island, a **450ms grace timer** starts. If you don't return, it automatically minimizes back into the compact pill.
- **Focus Lost / Blur Minimization:** Clicking on any other desktop window or application immediately minimizes the island back to the compact pill.
- **Inactivity Fallback:** If left open without mouse movement or interaction, it auto-minimizes after **12 seconds**.

### MPRIS Linux Media Player Integration (D-Bus)
- **Automatic Player Discovery:** Discovers running media players (Spotify, VLC, Chromium, Firefox, etc.) over standard Linux D-Bus `org.mpris.MediaPlayer2.*` via the `mpris` crate.
- **Real-Time Status Synchronization:** Background worker monitors player status and track changes without polling overhead, emitting `media-status-change` events only on state changes.
- **Playback Controls:** Exposes commands to `play_pause`, skip `next`, and go to `previous` track.
- **Compact Pill Equalizer:** When audio is playing, the compact pill automatically widens to display an animated 3-bar audio equalizer, track title, artist, and a quick play/pause button.
- **Dedicated Media Player Panel:** Clicking the media pill expands into a full player card with album art, track details, elapsed/total time progress bar, and playback controls.

### Desktop Notifications (Alert Banners)
- **Transient Alert Pill:** Incoming notifications expand the island into a sleek, dark notification banner (`380px × 96px`).
- **Visual Categorization:** Themed icons and color accents for different notification types (e.g. Chat/Message, Terminal/Build, and System alerts).
- **Auto-Dismiss & Countdown:** Displays a smooth progress countdown bar and automatically returns the island to its prior state (`idle` or `media`) after 4–5 seconds.
- **Manual Dismissal:** Click the notification or the close button (`×`) to dismiss immediately.
- **Testing & Extensibility:** Interactive test chips in the expanded panel let you trigger sample notifications on demand; external scripts can trigger notifications through the `trigger_notification` Tauri command.

### System Resource Monitoring (Linux Telemetry)
- **Zero-Dependency Native Sampling:** Reads directly from the Linux virtual filesystems (`/proc` and `/sys`) with sub-millisecond execution overhead (`< 0.05ms`) and zero external C libraries:
  - `/proc/stat`: Delta tick calculation for true multi-core CPU usage % and active core count.
  - `/proc/meminfo`: Exact physical memory utilization (used MB, total MB, memory %).
  - `/proc/net/dev`: Live network throughput calculation (`net_rx_speed_kb` and `net_tx_speed_kb`).
  - `/sys/class/power_supply/BAT*`: Hardware battery percentage, charging state (`⚡`), and AC detection.
- **Real-Time Live Event Stream:** Emits `system-telemetry` events every **2.4 seconds** to update the UI without excessive CPU wakeups.
- **Compact Pill Integration:**
  - Real-time CPU & RAM metric chip (e.g. `12% · 38%`) right in the idle pill.
  - Automatic low battery alert pill badge (`🪫 18%`) when battery drops below 20% on battery power.
- **Dedicated System Performance Panel:** Full modal view (`400px × 350px`) accessible from the overview panel with:
  - CPU usage gauge with core count and color-coded status (<60% green, 60-85% amber, >85% red).
  - RAM used vs. total GB progress bar.
  - Network download (`↓ DL`) and upload (`↑ UL`) throughput in KB/s or MB/s.
  - Battery charge level, charging bolt, or AC desktop power indicator.
  - Instant on-demand sampling refresh button.

### Developer Integrations & Live AI Agent (Unix Domain Socket IPC)
- **Local Unix Domain Socket:** Dynamic Island listens at `$XDG_RUNTIME_DIR/dynamic-island.sock` (or `/tmp/dynamic-island-$UID.sock`). Any local script, terminal process, or AI coding assistant can stream live status over a standard Unix socket:
  ```bash
  # Send raw JSON line via netcat or Python
  echo '{"target":"agent","state":"working","message":"Refactoring types.ts","progress":60}' | nc -U /tmp/dynamic-island-$UID.sock
  ```
- **CLI Utility (`island-cli`):**
  > **Note on Installation:** When installed via the Debian package (`.deb`), `island-cli` is automatically installed to `/usr/bin/island-cli`, so you can invoke `island-cli` anywhere from any terminal. When running from source, use `./scripts/island-cli` or symlink it to `~/.local/bin/island-cli`.

  - **Track Build Commands:** Wrap any long-running shell command to track elapsed time, display an active build pill, and trigger desktop alert banners upon completion:
    ```bash
    island-cli build --run "cargo test"
    island-cli build --run "npm run build"
    ```
  - **Live AI Agent Status:** Update what your AI coding assistant is doing:
    ```bash
    island-cli agent --state thinking --message "Analyzing repository AST"
    island-cli agent --state working --message "Refactoring src-tauri/src/lib.rs" --progress 75 --files "lib.rs,types.ts"
    island-cli agent --state review --message "Changes ready for approval"
    island-cli agent --state idle
    ```
  - **Trigger Custom Notifications:**
    ```bash
    island-cli notify "Deployment Ready" "Artifacts uploaded to staging" --icon terminal
    ```
  - **Install AI Agent Integrations (Antigravity, Cursor, Claude Code, Windsurf, OpenCode):**
    ```bash
    # Install for all detected agents on your system at once:
    island-cli install all

    # Or install for specific agents:
    island-cli install antigravity   # Antigravity (global plugin with lifecycle hooks)
    island-cli install cursor        # Cursor IDE (registers MCP server in ~/.cursor/mcp.json)
    island-cli install claude        # Claude Code (registers MCP in ~/.claude.json & CLAUDE.md)
    island-cli install windsurf      # Windsurf (registers MCP in ~/.codeium/windsurf/mcp_config.json)
    island-cli install opencode      # OpenCode (creates rules in ~/.config/opencode/rules.md)

    # Local project scope (.agents/, .cursor/rules/, or project CLAUDE.md):
    island-cli install cursor local

    # Model Context Protocol (MCP) stdio server:
    island-cli mcp                   # Runs native MCP server for any agent (Cline, Roo-Code, etc.)

    # Uninstall:
    island-cli uninstall all
    island-cli uninstall cursor
    ```
  - **Desktop Controls & Shortcuts:**
    ```bash
    island-cli toggle     # Toggle island expanded/collapsed
    island-cli screen     # Cycle island to next monitor
    island-cli settings   # Open Settings & Preferences panel
    ```
- **Dynamic Compact Pill Priority:**
  - `⚡ Build Running`: Active compilation displays a spinning badge in the pill.
  - `🟣 Agent Active`: Displays pulsing status dot and progress % in the pill (`thinking`, `working`, or `review`).
  - `🎵 Now Playing`: Displays audio equalizer, song name, and quick play/pause.
  - `💻 Idle Telemetry`: Displays live CPU/RAM usage or low battery alert.
- **Dedicated AI Agent Companion Panel:**
  - Interactive state simulator for rapid testing (Thinking, Working 65%, Review, Idle).
  - Changed files chip preview.
  - "Approve & Apply" and "Discard" actions for the review state.

### Linux System Tray & Global Desktop Hotkeys
- **Native Cinnamon Panel Tray Icon:**
  - Displays a clean system tray icon in the Cinnamon taskbar.
  - **Left-Click:** Instantly toggles the Dynamic Island expanded or collapsed.
  - **Right-Click Context Menu:**
    - *Toggle Dynamic Island*
    - *Cycle Screen ⮂*
    - *Settings & Preferences*
    - *Quit Dynamic Island*
- **Global Keyboard Shortcut Integration:**
  - Control the island from anywhere on your desktop (even inside full-screen IDEs or games) via the CLI toggle hook:
    ```bash
    island-cli toggle
    ```
  - **Bind in Ubuntu Cinnamon:** Open **System Settings → Keyboard → Shortcuts → Custom Shortcuts**, click **Add custom shortcut**, set command to `island-cli toggle`, and assign a key combination (e.g. `Ctrl+Alt+Space` or `Super+I`).
  - **Cycle Screens via Shortcut:** Bind `island-cli screen` to `Super+Shift+I` to jump monitors without reaching for the mouse.

### Settings & Personalization Panel
- **Dedicated Settings View (`400px × 460px`):** Accessible via the gear icon in the header or the **Settings & Preferences** action card on the overview panel.
- **Interaction & Timing Sliders:**
  - *Hover to Expand:* Toggle hover behavior on/off (allowing click-only if preferred).
  - *Hover Intent Delay:* Adjust between `100ms` and `600ms` (default: `220ms`).
  - *Leave Grace Period:* Adjust between `200ms` and `1000ms` (default: `450ms`).
  - *Inactivity Auto-Collapse:* Adjust between `5s` and `30s` (default: `12s`).
- **Placement & Screen Top Margin:**
  - Live margin slider (`4px` to `36px`) that immediately updates native window positioning to accommodate various desktop panel heights.
- **Compact Pill Module Toggles:**
  - Selectively enable or disable modules in the compact pill: CPU/RAM chip, low battery warning, active media equalizer, terminal build spinner, and AI agent aura.
- **System Integration & Autostart:**
  - Toggle **Launch on System Login** to generate standard XDG autostart files at `~/.config/autostart/dynamic-island.desktop`.
- **Zero-Friction Disk Persistence:**
  - Preferences automatically serialize to `~/.config/dynamic-island/settings.json`.

### Frontend UI & State Model (Svelte 5 + TypeScript)
- Built with modern Svelte 5 runes (`$state`, `$derived`) in [`src/lib/state/island.svelte.ts`](file:///myproject/project/personal/dynamic-island/src/lib/state/island.svelte.ts).
- Supported modes:
  - `idle`: Compact floating black pill (adapts automatically between standard status, system telemetry, and active media).
  - `expanded`: General overview panel (`400px × 420px`) with status tags, multi-monitor controls, media shortcuts, system performance card, settings card, and notification test controls.
  - `system`: Dedicated system resource and hardware performance view.
  - `settings`: Dedicated settings and personalization panel.
  - `notification`: Transient alert banner with countdown timer.
  - `media`: Rich MPRIS media player controller.
  - `agent`: Demo AI Agent view with interactive simulation.
- Accessible semantic HTML structure with keyboard focus, window blur listeners, and `Escape` to collapse.

---

## 2. Prerequisites & Linux System Dependencies

On Ubuntu / Debian, install the required development libraries:

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

Ensure **Node.js** (v20+ or v22+) and **Rust / Cargo** (1.80+) are available:
```bash
node -v
cargo -v
```

---

## 3. Development & Build Commands

### Install Frontend Dependencies
```bash
npm install
```

### Type & Svelte Checks
```bash
npm run check
```

### Build Frontend
```bash
npm run build
```

### Run Web Preview (Browser Only)
```bash
npm run dev
# Open http://localhost:1420
```

### Run Tauri Desktop Application
```bash
npm run tauri dev
```

### Build Production Binary & Debian Package
```bash
npm run tauri build
```

---

## 4. Multi-Monitor Usage

When working with dual or triple displays:
1. Open the Dynamic Island overview panel (click or hover over the pill).
2. If multiple monitors are detected:
   - Click the **`Screen: <name> ⮂`** badge to instantly cycle the Dynamic Island to your next monitor.
   - Or click any specific display chip in the **Switch Screen** row (e.g. `HDMI-1`, `DP-1`, `eDP-1`) to place the island on that monitor.
3. The selected monitor is preserved in the backend state, so subsequent expansions, collapses, media updates, and notifications stay on that monitor.

---

## 5. Real-World Daily Usage & Workflows

Here is how you can integrate the Linux Dynamic Island into your daily desktop experience:

### 1. "Fire-and-Forget" Terminal Builds & Tests
When executing compilation or test suites that take 30 seconds to several minutes, you normally alt-tab to documentation or Slack and lose track of when it completes.

**Setup a shell alias:**
Add this to your `~/.bashrc` or `~/.zshrc`:
```bash
alias di-run="island-cli build --run"
```

**Usage:**
```bash
di-run "cargo test"
di-run "npm run build"
di-run "pytest -v"
di-run "docker compose up -d --build"
```
* **While Running:** The Dynamic Island shrinks to an active pill showing `[ ⚡ cargo test • Running ]` with a spinner. You can read or browse full-screen without checking terminal.
* **On Completion:** The island immediately drops a banner:
  - If successful: `✓ 'cargo test' passed in 3.4s` (green checkmark).
  - If failed: `✗ 'cargo test' failed (exit code 1)` (red warning banner with exit code).

---

### 2. Autonomous AI Coding Agents (Claude Code, Aider, Antigravity)
Autonomous agents frequently perform multi-step planning, file editing, and test generation in the background. Rather than keeping a massive terminal window visible, stream the agent's live state to the Dynamic Island.

#### The Communication Pipeline
```text
[AI Agent / Script] ──(JSON line)──> [Unix Socket: /run/user/$UID/dynamic-island.sock]
                                                    │
                                                    ▼
                                          [Rust Core dev_ipc.rs]
                                                    │
                                                    ▼
                                          [Svelte 5 Island UI]
```

#### Supported States & UI Behavior
| State | Pill Appearance | Island Card Content |
| :--- | :--- | :--- |
| `thinking` | `[ 🟣 Agent: Analyzing AST... ]` (purple pulsing aura) | Task description, glowing avatar |
| `working` | `[ ⚡ Agent: Refactor lib.rs • 65% ]` (blue spinning gear) | Live progress bar, step details |
| `review` | `[ 👁️ Agent: Review Required ]` + Alert banner | Modified file chips, **Approve & Apply** & **Discard** buttons |
| `idle` | Standard pill (returns to CPU/RAM or media) | "Ready for your next task" |

#### Python Agent Integration Example
You can embed this in any Python agent loop or LangChain tool:
```python
import socket, json, os

def notify_island(state, message, task=None, progress=None, files=None):
    socket_path = f"/run/user/{os.getuid()}/dynamic-island.sock"
    payload = {
        "target": "agent",
        "state": state,
        "message": message,
        "task": task,
        "progress": progress,
        "changed_files": files,
    }
    try:
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
            client.connect(socket_path)
            client.sendall(json.dumps(payload).encode("utf-8") + b"\n")
    except Exception:
        pass  # Continue silently if Dynamic Island is closed

# 1. When agent begins planning:
notify_island("thinking", "Analyzing project dependency tree", task="Refactor Auth")

# 2. While agent is modifying files:
notify_island("working", "Writing unit tests (+60 lines)", task="Refactor Auth", progress=75, files=["auth.ts", "auth.test.ts"])

# 3. When agent finishes and requires human sign-off:
notify_island("review", "Review pull request diff before commit", files=["src/auth.ts", "tests/auth.test.ts"])
```

#### Bash / Shell Script Integration
Using `island-cli` (installed in `/usr/bin/island-cli`):
```bash
# Signal that the agent is thinking:
island-cli agent --state thinking --message "Scanning repository AST"

# Signal progress:
island-cli agent --state working --message "Refactoring database models" --progress 50

# Request human confirmation and wait for user's desktop decision:
island-cli agent --state review --message "Changes ready for human review" --files "models.py,schema.sql" --wait
```

#### Human-In-The-Loop Approval & Blocking `--wait`
When in `review` state, the Dynamic Island presents an interactive review card:
- **✓ Approve & Apply Changes:** Unblocks `island-cli` and exits with code `0`. The script or AI agent proceeds to commit and push.
- **✕ Discard:** Unblocks `island-cli` and exits with code `1`. The script or AI agent aborts and reverts changes.
- **Timeout (`--timeout <seconds>`):** Defaults to 300 seconds (5 minutes); exits with code `2` if no desktop action is taken.

```bash
# Example CI/CD or Agent workflow in shell:
if island-cli agent --state review -m "Apply database migration?" --wait; then
  echo "Approved! Executing migration..."
  npm run migrate
else
  echo "Rejected or canceled by user."
  exit 1
fi
```

---

### 3. Background Music & Podcast Control
When listening to music on Spotify, YouTube Music, VLC, or Firefox:
- **Zero Alt-Tabbing:** The island detects the player automatically over standard Linux D-Bus MPRIS.
- **Equalizer Pill:** Displays animated equalizer bars, track title, artist, and play/pause button.
- **Hover Intent:** Move your mouse over the pill for **220ms** to expand into the full album art card with previous/next controls; move your mouse away and it collapses automatically in **450ms**.

---

### 4. Zero-Distraction System Telemetry & Low Battery Warning
- **Real-Time Load:** Live CPU % and RAM % displayed directly on the compact pill without opening `htop` or heavy desktop monitors.
- **Overheating / Spike Detection:** Instant visual feedback if a rogue compile or container pegs your CPU at 100%.
- **Low-Battery Alert:** If your laptop battery drops below 20% while on battery power, the pill automatically displays an amber alert `🪫 18%` so you plug in before hibernation.

