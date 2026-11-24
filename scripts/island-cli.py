#!/usr/bin/env python3
"""
Dynamic Island Developer CLI & Multi-Agent MCP Integration
Send live AI agent updates, track terminal build commands, trigger notifications,
and install integrations for Antigravity, Cursor, Claude Code, Windsurf, and OpenCode.
"""

import os
import sys
import json
import time
import shutil
import socket
import argparse
import tempfile
import subprocess

AGENT_GUIDELINES_MD = """# Linux Dynamic Island Integration

You have direct integration with the user's floating Linux Dynamic Island desktop companion via the `dynamic-island` MCP server and `island-cli`.

## Best Practices:
1. **Interactive Questions, Prompts & Permissions**:
   - Whenever you ask the user a question, prompt for clarification, or wait for human permission/action, you MUST notify the Dynamic Island:
     - Via MCP tool: `island_prompt_user(question="...")` or `island_request_review(message="...")`
     - Or via CLI: `island-cli agent --state review --message "Action required: <question>"`
     - This immediately highlights the amber action card and pops an alert banner on the user's desktop screen!
2. **Interactive Code Review Sign-off**:
   - When completing substantial refactors or edits and requesting user review, run with `--wait`:
     ```bash
     island-cli agent --state review --wait --message "Changes ready for approval" --files "<changed_files>"
     ```
   - When running via `run_command` in an AI agent, set `WaitMsBeforeAsync: 500`. The command will continue waiting in the background as a task. When the user clicks "Approve & Apply" or "Discard" on their Dynamic Island desktop, the task completes (exit code 0 on approve, exit code 1 on discard) and reactively wakes you up!
   - Note: The user may also choose to reply directly in the chat UI.
3. **Long-Running Builds & Tests**:
   - For commands taking more than a few seconds (e.g. `npm run build`, `cargo test`, `docker build`), prefer wrapping execution with `island-cli build --run "<command>"` or notify when finished:
     ```bash
     island-cli build --run "<command>"
     ```
4. **Live Status & Notifications**:
   - Keep the user aware of background activity via `island_update_status` or `island-cli agent --state working --message "..."`.
   - Send completion banners via `island_notify` or `island-cli notify "Title" "Message"`.
"""

ANTIGRAVITY_PLUGIN_JSON = """{
  "name": "dynamic-island",
  "version": "0.1.0",
  "description": "Real-time AI Agent status, build tracking, and notification integration for Linux Dynamic Island"
}
"""

ANTIGRAVITY_HOOKS_JSON = """{
  "dynamic-island": {
    "enabled": true,
    "PreInvocation": [
      {
        "type": "command",
        "command": "island-cli hook pre-invocation",
        "timeout": 5
      }
    ],
    "PreToolUse": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "island-cli hook pre-tool",
            "timeout": 5
          }
        ]
      }
    ],
    "Stop": [
      {
        "type": "command",
        "command": "island-cli hook stop",
        "timeout": 5
      }
    ]
  }
}
"""

def get_socket_path():
    if sys.platform == "win32":
        return os.path.join(tempfile.gettempdir(), "dynamic-island.sock")
    runtime_dir = os.environ.get("XDG_RUNTIME_DIR")
    if runtime_dir:
        return os.path.join(runtime_dir, "dynamic-island.sock")
    uid = os.getuid() if hasattr(os, "getuid") else "default"
    return f"/tmp/dynamic-island-{uid}.sock"

def is_agent_enabled():
    try:
        config_dir = os.environ.get("XDG_CONFIG_HOME") or os.path.expanduser("~/.config")
        settings_path = os.path.join(config_dir, "dynamic-island", "settings.json")
        if os.path.exists(settings_path):
            with open(settings_path, "r") as f:
                data = json.load(f)
                return data.get("show_agent_in_pill", True)
    except Exception:
        pass
    return True

def send_ipc(payload):
    if sys.platform == "win32":
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as client:
                client.settimeout(0.3)
                client.connect(("127.0.0.1", 49215))
                message = json.dumps(payload) + "\n"
                client.sendall(message.encode("utf-8"))
                resp = client.recv(1024).decode("utf-8").strip()
                if resp:
                    data = json.loads(resp)
                    return data.get("ok", False)
                return True
        except Exception:
            return False

    sock_path = get_socket_path()
    if not os.path.exists(sock_path):
        if os.path.exists("/tmp/dynamic-island.sock"):
            sock_path = "/tmp/dynamic-island.sock"
        else:
            return False

    try:
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
            client.settimeout(0.3)
            client.connect(sock_path)
            message = json.dumps(payload) + "\n"
            client.sendall(message.encode("utf-8"))
            resp = client.recv(1024).decode("utf-8").strip()
            if resp:
                data = json.loads(resp)
                return data.get("ok", False)
            return True
    except Exception:
        return False

def send_ipc_wait(payload, timeout=305):
    if sys.platform == "win32":
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as client:
                client.settimeout(float(timeout))
                client.connect(("127.0.0.1", 49215))
                message = json.dumps(payload) + "\n"
                client.sendall(message.encode("utf-8"))
                resp = client.recv(2048).decode("utf-8").strip()
                if resp:
                    return json.loads(resp)
                return {"ok": False, "error": "empty_response"}
        except socket.timeout:
            return {"ok": False, "decision": "timeout", "error": "timed out waiting for user response"}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    sock_path = get_socket_path()
    if not os.path.exists(sock_path):
        if os.path.exists("/tmp/dynamic-island.sock"):
            sock_path = "/tmp/dynamic-island.sock"
        else:
            print(f"[island-cli] Dynamic Island socket not found at {sock_path}. Is dynamic-island running?", file=sys.stderr)
            return {"ok": False, "error": "socket_not_found"}

    try:
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
            client.settimeout(float(timeout))
            client.connect(sock_path)
            message = json.dumps(payload) + "\n"
            client.sendall(message.encode("utf-8"))
            resp = client.recv(2048).decode("utf-8").strip()
            if resp:
                return json.loads(resp)
            return {"ok": False, "error": "empty_response"}
    except socket.timeout:
        return {"ok": False, "decision": "timeout", "error": "timed out waiting for user response"}
    except Exception as e:
        return {"ok": False, "error": str(e)}

def get_full_dev_status():
    if sys.platform == "win32":
        state_file = os.path.join(tempfile.gettempdir(), "dynamic-island-state.json")
    else:
        runtime_dir = os.environ.get("XDG_RUNTIME_DIR")
        uid = os.getuid() if hasattr(os, "getuid") else "default"
        state_file = os.path.join(runtime_dir, "dynamic-island-state.json") if runtime_dir else f"/tmp/dynamic-island-state-{uid}.json"
    if os.path.exists(state_file):
        try:
            with open(state_file, "r") as f:
                return json.load(f)
        except Exception:
            pass

    try:
        sock_path = get_socket_path()
        if os.path.exists(sock_path):
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
                client.settimeout(0.3)
                client.connect(sock_path)
                client.sendall(b'{"target":"status"}\n')
                resp = client.recv(4096).decode("utf-8").strip()
                if resp:
                    return json.loads(resp)
    except Exception:
        pass
    return None

def get_current_agent_state():
    data = get_full_dev_status()
    if data:
        return data.get("agent", {}).get("state")
    return None

def get_last_review_decision():
    runtime_dir = os.environ.get("XDG_RUNTIME_DIR")
    rev_file = os.path.join(runtime_dir, "dynamic-island-review.json") if runtime_dir else f"/tmp/dynamic-island-review-{os.getuid()}.json"
    if os.path.exists(rev_file):
        try:
            with open(rev_file, "r") as f:
                data = json.load(f)
                ts = data.get("timestamp", 0)
                if time.time() - ts < 300:
                    return data.get("decision", "none")
        except Exception:
            pass

    try:
        sock_path = get_socket_path()
        if os.path.exists(sock_path):
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
                client.settimeout(0.3)
                client.connect(sock_path)
                client.sendall(b'{"target":"decision"}\n')
                resp = client.recv(1024).decode("utf-8").strip()
                if resp:
                    data = json.loads(resp)
                    return data.get("decision", "none")
    except Exception:
        pass
    return "none"

def cmd_agent(args):
    if getattr(args, "approve", False):
        success = send_ipc({"target": "review_decision", "decision": True})
        if success:
            print("✓ Review APPROVED")
            return 0
        else:
            print("✗ Failed to send approval via IPC", file=sys.stderr)
            return 1

    if getattr(args, "reject", False):
        success = send_ipc({"target": "review_decision", "decision": False})
        if success:
            print("✓ Review DISCARDED / REJECTED")
            return 0
        else:
            print("✗ Failed to send rejection via IPC", file=sys.stderr)
            return 1

    if getattr(args, "decision", False):
        dec = get_last_review_decision()
        if dec == "approved":
            print("approved")
            return 0
        elif dec == "rejected":
            print("rejected")
            return 1
        else:
            print("none")
            return 2

    if not is_agent_enabled():
        if getattr(args, "wait", False):
            print("✓ Human-in-the-loop: Changes APPROVED on Dynamic Island (Agent module disabled).")
            return 0
        return 0

    payload = {
        "target": "agent",
        "state": args.state,
        "message": args.message or f"Agent is {args.state}",
        "name": args.name,
        "task": args.task,
        "progress": args.progress,
        "changed_files": [f.strip() for f in args.files.split(",")] if args.files else None,
        "wait": getattr(args, "wait", False),
        "timeout_sec": getattr(args, "timeout", 300)
    }

    if payload["wait"]:
        timeout_sec = payload["timeout_sec"]
        print(f"[island-cli] Presented review on Dynamic Island. Waiting for human sign-off (timeout: {timeout_sec}s)...")
        res = send_ipc_wait(payload, timeout=timeout_sec + 5)
        decision = res.get("decision")
        if decision == "approved":
            print("✓ Human-in-the-loop: Changes APPROVED on Dynamic Island.")
            return 0
        elif decision == "rejected":
            print("✗ Human-in-the-loop: Changes DISCARDED / REJECTED on Dynamic Island.")
            return 1
        elif decision == "timeout":
            print(f"✗ Review timed out after {timeout_sec}s.")
            return 2
        else:
            print(f"✗ Error waiting for review: {res.get('error', 'unknown error')}", file=sys.stderr)
            return 1

    success = send_ipc(payload)
    if success:
        print(f"✓ Dynamic Island: Agent updated to [{args.state}] - {payload['message']}")
    return 0 if success else 1

def cmd_git(args):
    try:
        target_dir = getattr(args, "path", None)
        if not target_dir:
            is_inside = subprocess.run(["git", "rev-parse", "--is-inside-work-tree"], capture_output=True, text=True)
            if is_inside.returncode == 0 and is_inside.stdout.strip() == "true":
                target_dir = os.getcwd()
            else:
                # Fallback to active editor workspace if available
                dev_status = get_full_dev_status() or {}
                editor_info = dev_status.get("editor", {})
                ws = editor_info.get("workspace_path")
                if ws and os.path.isdir(ws):
                    target_dir = ws
                    editor_name = editor_info.get("editor", "Editor")
                    print(f"[island-cli] Tracking active {editor_name} workspace: {ws}")

        if not target_dir:
            print("Not inside a Git repository, and no active editor workspace detected.")
            return 1

        is_inside = subprocess.run(["git", "rev-parse", "--is-inside-work-tree"], cwd=target_dir, capture_output=True, text=True)
        if is_inside.returncode != 0 or is_inside.stdout.strip() != "true":
            print(f"Directory is not a Git repository: {target_dir}")
            return 1

        toplevel = subprocess.run(["git", "rev-parse", "--show-toplevel"], cwd=target_dir, capture_output=True, text=True).stdout.strip()
        repo_name = os.path.basename(toplevel)

        branch = subprocess.run(["git", "branch", "--show-current"], cwd=target_dir, capture_output=True, text=True).stdout.strip()
        if not branch:
            branch = subprocess.run(["git", "symbolic-ref", "--short", "HEAD"], cwd=target_dir, capture_output=True, text=True).stdout.strip() or "HEAD"

        porcelain = [l for l in subprocess.run(["git", "status", "--porcelain"], cwd=target_dir, capture_output=True, text=True).stdout.splitlines() if l.strip()]
        dirty_count = len(porcelain)

        print(f"Git Repository: {repo_name} ({toplevel})")
        print(f"Branch:         ⎇ {branch}")
        print(f"Dirty Files:    {dirty_count}")
        if dirty_count > 0:
            for line in porcelain[:6]:
                print(f"  {line}")
            if dirty_count > 6:
                print(f"  ...and {dirty_count - 6} more")

        send_ipc({"target": "git_refresh"})
        print("✓ Dynamic Island Git HUD refreshed.")
        return 0
    except Exception as e:
        print(f"Error querying Git: {e}", file=sys.stderr)
        return 1

def cmd_status(args):
    data = get_full_dev_status()
    if not data:
        print("[island-cli] Dynamic Island is not currently active or socket unreachable.")
        return 1

    print("Dynamic Island Status:")
    agent = data.get("agent")
    if agent:
        state_icon = {
            "idle": "⚪",
            "thinking": "🟣",
            "working": "🟢",
            "review": "🟡",
            "error": "🔴",
        }.get(agent.get("state", "idle"), "⚪")
        print(f"  Agent:   {state_icon} {agent.get('name', 'AI Agent')} [{agent.get('state', 'idle')}]")
        if agent.get("message"):
            print(f"           \"{agent.get('message')}\"")
        if agent.get("task"):
            print(f"           Task: {agent.get('task')}")

    build = data.get("build")
    if build and build.get("state") != "idle":
        print(f"  Build:   [{build.get('state')}] {build.get('command')}")

    editor = data.get("editor")
    if editor and editor.get("editor"):
        loc = f":{editor.get('line')}" if editor.get('line') else ""
        col = f":{editor.get('column')}" if editor.get('column') else ""
        print(f"  Editor:  ⚡ {editor.get('editor')} - {editor.get('file_name', '')}{loc}{col}")
        if editor.get("workspace"):
            print(f"           Workspace: {editor.get('workspace')}")
        if editor.get("workspace_path"):
            print(f"           Path:      {editor.get('workspace_path')}")
        if editor.get("git_branch"):
            print(f"           Branch:    ⎇ {editor.get('git_branch')}")

    return 0

def cmd_ports(args):
    resp = send_ipc_wait({"target": "ports"}, timeout=2)
    ports = []
    if isinstance(resp, dict) and resp.get("ok") and "ports" in resp:
        ports = resp["ports"]
    else:
        try:
            import re
            out = subprocess.check_output(["ss", "-tlpn", "-H"], text=True)
            for line in out.splitlines():
                parts = line.split()
                if len(parts) < 4: continue
                local = parts[3]
                m = re.search(r':(\d+)$', local)
                if not m: continue
                port = int(m.group(1))
                proc = None
                pid = None
                if len(parts) >= 6:
                    pm = re.search(r'\"([^\"]+)\",pid=(\d+)', parts[5])
                    if pm:
                        proc = pm.group(1)
                        pid = int(pm.group(2))
                ports.append({"port": port, "process": proc, "pid": pid, "container": None, "category": "web" if port in (3000, 5173, 5174, 8080) else "service"})
        except Exception:
            pass

    if not ports:
        print("No active listening ports detected.")
        return 0

    print(f"Active Listening Ports ({len(ports)}):")
    for p in ports:
        port_num = p.get("port")
        proc = p.get("process")
        pid = p.get("pid")
        container = p.get("container")
        cat = p.get("category", "")
        desc = f"🐳 {container}" if container else (f"{proc} (PID {pid})" if proc else "System")
        tag = f"[{cat}]" if cat else ""
        print(f"  :{port_num:<6} {tag:<10} {desc}")
    return 0

def cmd_docker(args):
    resp = send_ipc_wait({"target": "docker"}, timeout=3)
    containers = []
    if isinstance(resp, dict) and resp.get("ok") and "containers" in resp:
        containers = resp["containers"]
    else:
        try:
            out = subprocess.check_output(["docker", "ps", "-a", "--format", "{{json .}}"], text=True)
            for l in out.splitlines():
                if l.strip():
                    c = json.loads(l)
                    containers.append({
                        "id": c.get("ID", ""),
                        "name": c.get("Names", ""),
                        "image": c.get("Image", ""),
                        "is_running": c.get("State", "").lower() == "running",
                        "status": c.get("Status", ""),
                        "ports": c.get("Ports", ""),
                    })
        except Exception as e:
            print(f"Error querying Docker: {e}")
            return 1

    if not containers:
        print("No Docker containers found.")
        return 0

    running = [c for c in containers if c.get("is_running")]
    print(f"Docker Containers ({len(running)} running / {len(containers)} total):")
    for c in containers:
        status_icon = "🟢" if c.get("is_running") else "⚪"
        name = c.get("name", "")
        img = c.get("image", "")
        stat = c.get("status", "")
        print(f"  {status_icon} {name:<22} {img:<24} ({stat})")
    return 0

def cmd_todo(args):
    action = args.todo_action
    payload = {"target": "todo", "action": action}

    if action == "add":
        if not args.title:
            print("Error: Task title is required. Usage: island-cli todo add 'Task title'")
            return 1
        payload["title"] = args.title
        if args.priority:
            payload["priority"] = args.priority.lower()
        if args.due:
            payload["due"] = args.due
    elif action in ("toggle", "delete"):
        if not args.id:
            print(f"Error: Task ID is required. Usage: island-cli todo {action} <id>")
            return 1
        payload["id"] = args.id
    elif action in ("export", "sync"):
        if args.path:
            payload["path"] = args.path

    resp = send_ipc_wait(payload, timeout=5)
    if not isinstance(resp, dict):
        print("Error: Dynamic Island is not currently running or IPC failed.")
        return 1

    if not resp.get("ok"):
        print(f"Error: {resp.get('error', 'Operation failed')}")
        return 1

    if action == "add":
        item = resp.get("todo", {})
        prio = f" [{item.get('priority', '').upper()}]" if item.get('priority') != 'medium' else ""
        print(f"✓ Added task: {item.get('title')}{prio} (ID: {item.get('id')})")
    elif action == "list":
        todos = resp.get("todos", [])
        if not todos:
            print("No tasks found in To-Do list.")
            return 0
        pending = [t for t in todos if not t.get("completed")]
        completed = [t for t in todos if t.get("completed")]
        print(f"To-Do List ({len(pending)} pending / {len(todos)} total):")
        for t in todos:
            mark = "✓" if t.get("completed") else " "
            prio = f" [{t.get('priority', '').upper()}]" if t.get("priority") != "medium" else ""
            due = f" (due: {t.get('due_date')})" if t.get("due_date") else ""
            print(f"  [{mark}] {t.get('title')}{prio}{due}  (ID: {t.get('id')})")
    elif action == "toggle":
        item = resp.get("todo", {})
        status = "Completed" if item.get("completed") else "Pending"
        print(f"✓ Task updated: {item.get('title')} -> {status}")
    elif action == "delete":
        print("✓ Task deleted successfully.")
    elif action == "clear_completed":
        print(f"✓ Cleared {resp.get('removed', 0)} completed tasks.")
    elif action == "export":
        print(f"✓ Tasks exported to markdown: {resp.get('path')}")
    elif action == "sync":
        print(f"✓ Synced {resp.get('count', 0)} tasks from markdown.")

    return 0

def cmd_focus(args):
    action = getattr(args, "focus_action", None) or "start"
    payload = {"target": "focus", "action": action}
    if action == "start":
        mins = getattr(args, "minutes", 25) or 25
        payload["minutes"] = mins
        label = getattr(args, "label", None)
        if label:
            payload["label"] = label
        print(f"🔥 Starting focus session for {mins} minutes ({label or 'Locking In'})...")
    elif action == "stop":
        print("💀 Stopping focus session...")
    elif action == "pause":
        print("⏸ Pausing focus session...")
    elif action == "resume":
        print("▶ Resuming focus session...")
    elif action == "open":
        print("🔥 Opening Focus Mode dashboard...")

    res = send_ipc(payload)
    if not res:
        print("Error: Could not connect to Dynamic Island. Is it running?")
        return 1
    print(f"✓ Dynamic Island: Focus mode {action} executed")
    return 0

def cmd_prayer(args):
    action = getattr(args, "prayer_action", None) or "status"
    payload = {"target": "prayer", "action": action}

    if action == "set_city":
        city_val = getattr(args, "city", None)
        if not city_val:
            print("Error: City name is required. Usage: island-cli prayer city <city_name>")
            return 1
        payload["city"] = city_val
    elif action == "test":
        prayer_name = getattr(args, "name", None)
        if prayer_name:
            payload["prayer_name"] = prayer_name

    resp = send_ipc_wait(payload, timeout=5)

    # Fallback to direct calculation or notification if daemon hasn't reloaded yet
    if not isinstance(resp, dict) or not resp.get("ok"):
        if action == "test":
            prayer_name = getattr(args, "name", None) or "Maghrib"
            send_ipc({
                "target": "notification",
                "title": f"🕌 Time for {prayer_name}",
                "message": f"Hayya 'ala-s-Salah. It is now time for {prayer_name} prayer.",
                "app_name": "Islamic Prayer Reminder",
                "icon": "terminal",
                "duration_ms": 8000,
            })
            print(f"✓ Triggered prayer alert for {prayer_name} on Dynamic Island.")
            return 0
        elif action in ("status", "next"):
            import urllib.request
            import urllib.parse
            import json as pyjson
            import datetime

            city = "Surabaya"
            cfg_path = os.path.expanduser("~/.config/dynamic-island/settings.json")
            if os.path.exists(cfg_path):
                try:
                    with open(cfg_path, "r", encoding="utf-8") as f:
                        cfg = pyjson.load(f)
                        city = cfg.get("prayer_city") or cfg.get("weather_city") or "Surabaya"
                except Exception:
                    pass

            try:
                url = f"https://api.aladhan.com/v1/timingsByCity?city={urllib.parse.quote(city)}&country=Indonesia&method=11"
                req = urllib.request.Request(url, headers={"User-Agent": "DynamicIsland/1.0"})
                with urllib.request.urlopen(req, timeout=4) as response:
                    raw = pyjson.loads(response.read().decode())
                    data = raw.get("data", {})
                    timings = data.get("timings", {})
                    hijri = data.get("date", {}).get("hijri", {})
                    hijri_str = f"{hijri.get('day')} {hijri.get('month', {}).get('en')} {hijri.get('year')} AH"

                    now = datetime.datetime.now()
                    now_min = now.hour * 60 + now.minute

                    prayers = [
                        ("Fajr", "Fajr (Subuh)", timings.get("Fajr", "04:15")),
                        ("Sunrise", "Sunrise (Syuruq)", timings.get("Sunrise", "05:30")),
                        ("Dhuhr", "Dhuhr (Dzuhur)", timings.get("Dhuhr", "11:45")),
                        ("Asr", "Asr (Ashar)", timings.get("Asr", "15:05")),
                        ("Maghrib", "Maghrib", timings.get("Maghrib", "17:50")),
                        ("Isha", "Isha (Isya)", timings.get("Isha", "19:05")),
                    ]

                    next_p = None
                    for name, label, t_str in prayers:
                        clean_t = t_str.split()[0]
                        hp, mp = map(int, clean_t.split(":"))
                        p_min = hp * 60 + mp
                        if p_min > now_min and not next_p and name in ("Fajr", "Dhuhr", "Asr", "Maghrib", "Isha"):
                            diff = p_min - now_min
                            dh = diff // 60
                            dm = diff % 60
                            cd = f"{dh}h {dm}m" if dh > 0 else f"{dm}m"
                            next_p = (name, label, clean_t, cd)

                    if not next_p:
                        next_p = ("Fajr", "Fajr (Subuh)", timings.get("Fajr", "04:15").split()[0], "Tomorrow")

                    if action == "next":
                        print(f"🕌 Next: {next_p[0]} at {next_p[2]} ({next_p[3]} remaining) in {city}")
                        return 0

                    print(f"🕌 Islamic Prayer Times — {city}")
                    print(f"   📅 {hijri_str} • {now.strftime('%Y-%m-%d')}")
                    print(f"   ★ Next Prayer: {next_p[0]} at {next_p[2]} ({next_p[3]} remaining)")
                    print("\n   Daily Schedule:")
                    for name, label, t_str in prayers:
                        clean_t = t_str.split()[0]
                        marker = "★ NEXT" if name == next_p[0] else " "
                        print(f"     [{marker:<6}] {label:<18}: {clean_t}")
                    return 0
            except Exception as e:
                print(f"Note: Prayer calculation fallback note: {e}")

        print(f"Error: {resp.get('error', 'Operation failed') if isinstance(resp, dict) else 'Dynamic Island IPC unavailable'}")
        return 1

    if action == "next":
        name = resp.get("name", "Prayer")
        time_str = resp.get("time", "--:--")
        countdown = resp.get("formatted_countdown", "")
        city = resp.get("city", "")
        print(f"🕌 Next: {name} at {time_str} ({countdown} remaining) in {city}")
    elif action == "test":
        print(f"✓ {resp.get('message', 'Triggered prayer alert')}")
    elif action == "set_city":
        print(f"✓ Prayer city updated to: {resp.get('city')}")
    else:
        # status
        sched = resp.get("schedule", {})
        city = sched.get("city", "Local")
        country = sched.get("country", "")
        hijri = sched.get("date_hijri", "")
        gregorian = sched.get("date_gregorian", "")
        qibla = sched.get("qibla_bearing", 0.0)
        method = sched.get("method_name", "")
        next_p = sched.get("next_prayer", {})

        print(f"🕌 Islamic Prayer Times — {city}, {country}")
        print(f"   📅 {hijri} • {gregorian}")
        print(f"   🕋 Qibla: {qibla}° from North • Method: {method}")
        if next_p:
            print(f"   ★ Next Prayer: {next_p.get('name')} at {next_p.get('time')} ({next_p.get('formatted_countdown')} remaining)")
        print("\n   Daily Schedule:")
        for item in sched.get("items", []):
            marker = "★ NEXT" if item.get("is_next") else ("▶ CURR" if item.get("is_current") else ("✓" if item.get("is_passed") else " "))
            label = item.get("label", item.get("name", ""))
            ar = item.get("arabic_name", "")
            t = item.get("time", "")
            print(f"     [{marker:<6}] {label:<18} ({ar:<12}): {t}")

    return 0

def cmd_meeting(args):
    action = getattr(args, "meeting_action", None) or "status"

    if action == "focus":
        import subprocess
        platform = getattr(args, "platform", None) or "zoom"
        cmd = ["wmctrl", "-xa", platform]
        try:
            subprocess.run(cmd, check=True)
            print(f"✓ Brought {platform} to foreground")
            return 0
        except Exception as e:
            print(f"Error bringing window to foreground: {e}")
            return 1

    # Check active meeting status
    import subprocess
    try:
        pactl = subprocess.check_output(["pactl", "list", "source-outputs"], text=True)
    except Exception as e:
        print(f"Error checking audio streams: {e}")
        return 1

    wmctrl_out = ""
    try:
        wmctrl_out = subprocess.check_output(["wmctrl", "-lx"], text=True)
    except Exception:
        pass

    meeting_detected = None
    lower_pa = pactl.lower()
    lower_wm = wmctrl_out.lower()

    if "zoom" in lower_pa or "zoom meeting" in lower_wm:
        meeting_detected = ("Zoom", "🟦", "Zoom Meeting")
    elif "discord" in lower_pa or "vesktop" in lower_pa:
        meeting_detected = ("Discord", "🎧", "Discord Voice Channel")
    elif "slack" in lower_pa:
        meeting_detected = ("Slack", "💬", "Slack Huddle")
    elif "teams" in lower_pa:
        meeting_detected = ("Microsoft Teams", "👥", "Teams Meeting")
    elif ("meet -" in lower_wm or "meet –" in lower_wm or "meet.google.com" in lower_wm) and any(b in lower_pa for b in ["chrome", "firefox", "brave"]):
        meeting_detected = ("Google Meet", "📹", "Google Meet Call")
    elif any(b in lower_pa for b in ["chrome", "firefox", "brave"]):
        meeting_detected = ("Browser Call", "📞", "Active WebRTC Call")

    if meeting_detected:
        name, icon, title = meeting_detected
        print(f"{icon} Active Call Detected: {name}")
        print(f"   Title: {title}")
        print(f"   Status: Microphone active in PipeWire/PulseAudio")
    else:
        print("⚪ No active meeting or call detected (Microphone is idle).")

    return 0

def cmd_build(args):
    if getattr(args, "clear", False) or args.state == "idle":
        payload = {
            "target": "build",
            "command": "",
            "state": "idle"
        }
        success = send_ipc(payload)
        if success:
            print("✓ Dynamic Island: Cleared build status")
        return 0 if success else 1

    if args.run:
        cmd_str = args.run
        print(f"[island-cli] Tracking build: '{cmd_str}'...")

        send_ipc({
            "target": "build",
            "command": cmd_str,
            "state": "running"
        })

        start_time = time.time()
        try:
            res = subprocess.run(cmd_str, shell=True)
            elapsed_ms = int((time.time() - start_time) * 1000)

            state = "success" if res.returncode == 0 else "failed"
            send_ipc({
                "target": "build",
                "command": cmd_str,
                "state": state,
                "duration_ms": elapsed_ms,
                "exit_code": res.returncode
            })

            duration_sec = elapsed_ms / 1000.0
            if res.returncode == 0:
                print(f"✓ Dynamic Island: Build PASSED ({duration_sec:.2f}s)")
            else:
                print(f"✗ Dynamic Island: Build FAILED with exit code {res.returncode} ({duration_sec:.2f}s)")
            return res.returncode
        except KeyboardInterrupt:
            elapsed_ms = int((time.time() - start_time) * 1000)
            send_ipc({
                "target": "build",
                "command": cmd_str,
                "state": "failed",
                "message": "Cancelled by user (SIGINT)",
                "duration_ms": elapsed_ms,
                "exit_code": 130
            })
            print(f"\n[island-cli] Build cancelled by user (Ctrl+C)")
            return 130
        except Exception as e:
            elapsed_ms = int((time.time() - start_time) * 1000)
            send_ipc({
                "target": "build",
                "command": cmd_str,
                "state": "failed",
                "message": str(e),
                "duration_ms": elapsed_ms,
                "exit_code": 1
            })
            print(f"[island-cli] Build error: {e}")
            return 1

    payload = {
        "target": "build",
        "command": args.command or "build",
        "state": args.state,
        "duration_ms": args.duration,
        "exit_code": args.code
    }
    success = send_ipc(payload)
    if success:
        print(f"✓ Dynamic Island: Build state updated to [{args.state}]")
    return 0 if success else 1

def cmd_notify(args):
    payload = {
        "target": "notification",
        "title": args.title,
        "message": args.message,
        "app_name": args.app or "Developer CLI",
        "icon": args.icon or "terminal",
        "duration_ms": args.duration or 4500
    }
    success = send_ipc(payload)
    if success:
        print(f"✓ Dynamic Island: Notification sent '{args.title}'")
    return 0 if success else 1

def cmd_toggle(args):
    success = send_ipc({"target": "toggle"})
    if success:
        print("✓ Dynamic Island: Toggled")
    return 0 if success else 1

def cmd_screen(args):
    success = send_ipc({"target": "screen"})
    if success:
        print("✓ Dynamic Island: Cycled to next monitor")
    return 0 if success else 1

def cmd_settings(args):
    success = send_ipc({"target": "settings"})
    if success:
        print("✓ Dynamic Island: Opened Settings panel")
    return 0 if success else 1

def cmd_position(args):
    payload = {
        "target": "position",
        "position": args.pos,
        "x_offset": args.offset
    }
    resp = send_ipc_wait(payload, timeout=5)
    if resp and resp.get("ok"):
        pos = resp.get("horizontal_position", args.pos)
        offset = resp.get("x_offset", 0.0)
        print(f"✓ Dynamic Island: Position updated to [{pos}] (x_offset: {offset}px)")
        return 0
    else:
        err = resp.get("error") if resp else "Failed to send IPC"
        print(f"✗ Dynamic Island: Could not set position: {err}")
        return 1

def cmd_hook(args):
    """
    Native Antigravity lifecycle hook handler.
    Called directly by Antigravity via `island-cli hook <event>`.
    """
    event = args.event
    if not is_agent_enabled():
        if event == "pre-tool":
            print(json.dumps({"decision": "allow"}))
        else:
            print(json.dumps({}))
        return 0

    raw_input = ""
    try:
        raw_input = sys.stdin.read()
        data = json.loads(raw_input) if raw_input.strip() else {}
    except Exception:
        data = {}

    if event == "pre-invocation":
        send_ipc({
            "target": "agent",
            "name": "Antigravity Agent",
            "state": "thinking",
            "message": "Analyzing prompt & planning next steps...",
        })
        print(json.dumps({}))

    elif event == "pre-tool":
        tool_call = data.get("toolCall", {})
        tool_name = tool_call.get("name", "tool")
        tool_args = tool_call.get("args", {})

        message = f"Executing {tool_name}"
        changed_files = None
        state = "working"

        if tool_name == "ask_question":
            questions = tool_args.get("questions", [])
            q_text = questions[0].get("question", "Clarification requested") if questions else "Clarification requested"
            message = f"Action needed: {q_text[:40]}"
            state = "review"
            send_ipc({
                "target": "notification",
                "title": "Antigravity Action Needed",
                "message": q_text[:90],
                "icon": "message",
                "duration_ms": 6000
            })
        elif tool_name == "run_command":
            cmd = tool_args.get("CommandLine", "").strip()
            if "island-cli agent" in cmd or "island-cli hook" in cmd:
                print(json.dumps({"decision": "allow"}))
                return 0
            first_line = cmd.split("\n")[0][:45]
            if tool_args.get("BypassSandbox"):
                message = f"Permission required: {first_line[:30]}"
                state = "review"
                send_ipc({
                    "target": "notification",
                    "title": "Permission Requested",
                    "message": f"Requires manual approval: {first_line[:60]}",
                    "icon": "terminal",
                    "duration_ms": 6000
                })
            else:
                message = f"Shell: {first_line}"
        elif tool_name in ("edit_file", "write_file"):
            path = tool_args.get("Path", "")
            basename = os.path.basename(path) if path else "file"
            action = "Editing" if tool_name == "edit_file" else "Writing"
            message = f"{action} {basename}"
            if basename:
                changed_files = [basename]
        elif tool_name == "read_file":
            path = tool_args.get("Path", "")
            basename = os.path.basename(path) if path else "file"
            message = f"Reading {basename}"
        elif tool_name == "search_web":
            q = tool_args.get("query", "")[:35]
            message = f"Web search: {q}"

        payload = {
            "target": "agent",
            "name": "Antigravity Agent",
            "state": state,
            "message": message,
        }
        if changed_files:
            payload["changed_files"] = changed_files

        send_ipc(payload)
        print(json.dumps({"decision": "allow"}))

    elif event == "post-tool":
        print(json.dumps({}))

    elif event == "stop":
        term_reason = data.get("terminationReason", "NO_TOOL_CALL")
        if term_reason == "NO_TOOL_CALL":
            current_state = get_current_agent_state()
            if current_state != "review":
                send_ipc({
                    "target": "agent",
                    "name": "Antigravity Agent",
                    "state": "idle",
                    "message": "Awaiting your reply...",
                })
                send_ipc({
                    "target": "notification",
                    "title": "Antigravity Agent",
                    "message": "Response ready - awaiting your input",
                    "app_name": "Antigravity",
                    "icon": "terminal",
                    "duration_ms": 3500
                })
        print(json.dumps({"decision": "allow"}))

    else:
        print(json.dumps({}))

    return 0

def install_antigravity(scope):
    if scope == "global":
        target_dir = os.path.expanduser("~/.gemini/config/plugins/dynamic-island")
        rules_dir = os.path.join(target_dir, "rules")
        os.makedirs(rules_dir, exist_ok=True)

        with open(os.path.join(target_dir, "plugin.json"), "w") as f:
            f.write(ANTIGRAVITY_PLUGIN_JSON)

        with open(os.path.join(target_dir, "hooks.json"), "w") as f:
            f.write(ANTIGRAVITY_HOOKS_JSON)

        with open(os.path.join(rules_dir, "AGENTS.md"), "w") as f:
            f.write(AGENT_GUIDELINES_MD)

        print(f"✓ Antigravity: Installed globally in {target_dir}")
        print(f"  • hooks.json configured to use 'island-cli hook'")
        return True
    else:
        target_dir = os.path.abspath(".agents")
        rules_dir = os.path.join(target_dir, "rules")
        os.makedirs(rules_dir, exist_ok=True)

        with open(os.path.join(target_dir, "hooks.json"), "w") as f:
            f.write(ANTIGRAVITY_HOOKS_JSON)

        with open(os.path.join(rules_dir, "dynamic-island.md"), "w") as f:
            f.write(AGENT_GUIDELINES_MD)

        print(f"✓ Antigravity: Installed locally for workspace in {target_dir}")
        return True

def install_cursor(scope):
    mcp_config = {
        "command": "island-cli",
        "args": ["mcp"],
        "type": "stdio"
    }

    if scope == "global":
        cursor_dir = os.path.expanduser("~/.cursor")
        os.makedirs(cursor_dir, exist_ok=True)
        mcp_path = os.path.join(cursor_dir, "mcp.json")
        data = {}
        if os.path.exists(mcp_path):
            try:
                with open(mcp_path, "r") as f:
                    data = json.load(f)
            except Exception:
                data = {}

        if "mcpServers" not in data or not isinstance(data["mcpServers"], dict):
            data["mcpServers"] = {}

        data["mcpServers"]["dynamic-island"] = mcp_config

        with open(mcp_path, "w") as f:
            json.dump(data, f, indent=2)

        rules_file = os.path.expanduser("~/.cursorrules")
        with open(rules_file, "w") as f:
            f.write(AGENT_GUIDELINES_MD)

        print(f"✓ Cursor: Registered MCP server in {mcp_path}")
        print(f"✓ Cursor: Added rules to {rules_file}")
        return True
    else:
        rules_dir = os.path.abspath(".cursor/rules")
        os.makedirs(rules_dir, exist_ok=True)
        rules_file = os.path.join(rules_dir, "dynamic-island.mdc")
        with open(rules_file, "w") as f:
            f.write(AGENT_GUIDELINES_MD)
        print(f"✓ Cursor: Added workspace rules to {rules_file}")
        return True

def install_claude(scope):
    if scope == "global":
        claude_dir = os.path.expanduser("~/.claude")
        os.makedirs(claude_dir, exist_ok=True)

        claude_json = os.path.expanduser("~/.claude.json")
        if os.path.exists(claude_json):
            try:
                with open(claude_json, "r") as f:
                    cdata = json.load(f)
                if "mcpServers" not in cdata or not isinstance(cdata["mcpServers"], dict):
                    cdata["mcpServers"] = {}
                cdata["mcpServers"]["dynamic-island"] = {
                    "command": "island-cli",
                    "args": ["mcp"]
                }
                with open(claude_json, "w") as f:
                    json.dump(cdata, f, indent=2)
                print(f"✓ Claude Code: Registered MCP server in {claude_json}")
            except Exception as e:
                print(f"  Note: Could not update ~/.claude.json: {e}")

        rules_file = os.path.join(claude_dir, "CLAUDE.md")
        with open(rules_file, "w") as f:
            f.write(AGENT_GUIDELINES_MD)
        print(f"✓ Claude Code: Added global guidelines to {rules_file}")
        return True
    else:
        rules_file = os.path.abspath("CLAUDE.md")
        with open(rules_file, "w") as f:
            f.write(AGENT_GUIDELINES_MD)
        print(f"✓ Claude Code: Added project guidelines to {rules_file}")
        return True

def install_windsurf(scope):
    if scope == "global":
        windsurf_dir = os.path.expanduser("~/.codeium/windsurf")
        os.makedirs(windsurf_dir, exist_ok=True)
        mcp_path = os.path.join(windsurf_dir, "mcp_config.json")

        data = {}
        if os.path.exists(mcp_path):
            try:
                with open(mcp_path, "r") as f:
                    data = json.load(f)
            except Exception:
                data = {}

        if "mcpServers" not in data or not isinstance(data["mcpServers"], dict):
            data["mcpServers"] = {}

        data["mcpServers"]["dynamic-island"] = {
            "command": "island-cli",
            "args": ["mcp"]
        }

        with open(mcp_path, "w") as f:
            json.dump(data, f, indent=2)

        memories_dir = os.path.join(windsurf_dir, "memories")
        os.makedirs(memories_dir, exist_ok=True)
        rules_file = os.path.join(memories_dir, "global_rules.md")
        with open(rules_file, "w") as f:
            f.write(AGENT_GUIDELINES_MD)

        print(f"✓ Windsurf: Configured MCP in {mcp_path}")
        print(f"✓ Windsurf: Added global memory rules in {rules_file}")
        return True
    else:
        rules_file = os.path.abspath(".windsurfrules")
        with open(rules_file, "w") as f:
            f.write(AGENT_GUIDELINES_MD)
        print(f"✓ Windsurf: Added workspace rules to {rules_file}")
        return True

def install_opencode(scope):
    rules_file = os.path.expanduser("~/.config/opencode/rules.md") if scope == "global" else os.path.abspath("OPENCODE.md")
    os.makedirs(os.path.dirname(rules_file), exist_ok=True)
    with open(rules_file, "w") as f:
        f.write(AGENT_GUIDELINES_MD)
    print(f"✓ OpenCode: Added instructions to {rules_file}")
    return True

def cmd_install(args):
    target = (args.target or "antigravity").lower()
    scope = (args.scope or ("local" if getattr(args, "local", False) else "global")).lower()

    if target in ("antigravity", "gemini"):
        install_antigravity(scope)
    elif target == "cursor":
        install_cursor(scope)
    elif target in ("claude", "claudecode"):
        install_claude(scope)
    elif target in ("windsurf", "codeium"):
        install_windsurf(scope)
    elif target in ("opencode", "openhands", "aider"):
        install_opencode(scope)
    elif target in ("all", "*"):
        print(f"Installing Dynamic Island integrations across all supported agents ({scope}):\n")
        install_antigravity(scope)
        install_cursor(scope)
        install_claude(scope)
        install_windsurf(scope)
        install_opencode(scope)
        print("\nAll AI coding agents configured! They will now automatically stream status to your Dynamic Island.")
    else:
        print(f"[island-cli] Unknown target: '{target}'. Available: antigravity, cursor, claude, windsurf, opencode, all", file=sys.stderr)
        return 1

    return 0

def cmd_uninstall(args):
    target = (args.target or "antigravity").lower()
    scope = (args.scope or ("local" if getattr(args, "local", False) else "global")).lower()

    if target in ("antigravity", "gemini"):
        if scope == "global":
            p = os.path.expanduser("~/.gemini/config/plugins/dynamic-island")
            if os.path.exists(p):
                shutil.rmtree(p)
                print(f"✓ Removed {p}")
        else:
            p = os.path.abspath(".agents")
            if os.path.exists(p):
                shutil.rmtree(p)
                print(f"✓ Removed {p}")
    elif target == "cursor":
        mcp_path = os.path.expanduser("~/.cursor/mcp.json")
        if os.path.exists(mcp_path):
            try:
                with open(mcp_path, "r") as f:
                    d = json.load(f)
                d.get("mcpServers", {}).pop("dynamic-island", None)
                with open(mcp_path, "w") as f:
                    json.dump(d, f, indent=2)
                print(f"✓ Removed dynamic-island from {mcp_path}")
            except Exception:
                pass
    elif target in ("claude", "claudecode"):
        claude_json = os.path.expanduser("~/.claude.json")
        if os.path.exists(claude_json):
            try:
                with open(claude_json, "r") as f:
                    d = json.load(f)
                d.get("mcpServers", {}).pop("dynamic-island", None)
                with open(claude_json, "w") as f:
                    json.dump(d, f, indent=2)
                print(f"✓ Removed dynamic-island from {claude_json}")
            except Exception:
                pass
    elif target in ("windsurf", "codeium"):
        p = os.path.expanduser("~/.codeium/windsurf/mcp_config.json")
        if os.path.exists(p):
            try:
                with open(p, "r") as f:
                    d = json.load(f)
                d.get("mcpServers", {}).pop("dynamic-island", None)
                with open(p, "w") as f:
                    json.dump(d, f, indent=2)
                print(f"✓ Removed dynamic-island from {p}")
            except Exception:
                pass
    elif target in ("all", "*"):
        args.target = "antigravity"
        cmd_uninstall(args)
        args.target = "cursor"
        cmd_uninstall(args)
        args.target = "claude"
        cmd_uninstall(args)
        args.target = "windsurf"
        cmd_uninstall(args)
    return 0

def run_mcp_server():
    """
    Standard Model Context Protocol (MCP) stdio server.
    Allows Cursor, Claude Code, Windsurf, OpenCode, and Cline to interact with the Dynamic Island natively.
    """
    while True:
        line = sys.stdin.readline()
        if not line:
            break
        try:
            req = json.loads(line)
        except Exception:
            continue

        req_id = req.get("id")
        method = req.get("method")
        params = req.get("params", {})

        if method == "initialize":
            resp = {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "dynamic-island-mcp",
                        "version": "0.1.0"
                    }
                }
            }
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()

        elif method == "notifications/initialized":
            pass

        elif method == "tools/list":
            resp = {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "tools": [
                        {
                            "name": "island_update_status",
                            "description": "Update the AI Agent status pill & panel on the Linux Dynamic Island (thinking, working, review, idle)",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "state": {
                                        "type": "string",
                                        "enum": ["idle", "thinking", "working", "review", "error"],
                                        "description": "Agent state"
                                    },
                                    "message": {
                                        "type": "string",
                                        "description": "Status description displayed on the island"
                                    },
                                    "name": {
                                        "type": "string",
                                        "description": "Name of the AI assistant (e.g. Cursor, Claude, Windsurf)"
                                    },
                                    "task": {
                                        "type": "string",
                                        "description": "Active task or goal title"
                                    },
                                    "progress": {
                                        "type": "integer",
                                        "description": "Progress percentage (0-100)"
                                    },
                                    "changed_files": {
                                        "type": "array",
                                        "items": {"type": "string"},
                                        "description": "List of filenames currently modified"
                                    }
                                },
                                "required": ["state", "message"]
                            }
                        },
                        {
                            "name": "island_prompt_user",
                            "description": "Notify Dynamic Island when asking the user a question, awaiting clarification, or requesting permission",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "question": {"type": "string", "description": "The question or action needed from the user"},
                                    "options": {"type": "array", "items": {"type": "string"}, "description": "Selectable choices if applicable"}
                                },
                                "required": ["question"]
                            }
                        },
                        {
                            "name": "island_request_review",
                            "description": "Present an interactive desktop human-in-the-loop review modal with 'Approve & Apply' and 'Discard' buttons on the Dynamic Island",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "message": {
                                        "type": "string",
                                        "description": "Review summary message"
                                    },
                                    "changed_files": {
                                        "type": "array",
                                        "items": {"type": "string"},
                                        "description": "Files to present as changed chips for approval"
                                    }
                                },
                                "required": ["message"]
                            }
                        },
                        {
                            "name": "island_notify",
                            "description": "Display a floating alert notification banner on the Linux Dynamic Island",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "title": {"type": "string", "description": "Notification title"},
                                    "message": {"type": "string", "description": "Notification message body"},
                                    "icon": {"type": "string", "enum": ["terminal", "message", "system"], "description": "Icon type"}
                                },
                                "required": ["title", "message"]
                            }
                        }
                    ]
                }
            }
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()

        elif method == "tools/call":
            tool_name = params.get("name")
            arguments = params.get("arguments", {})

            if not is_agent_enabled() and tool_name in ("island_update_status", "island_prompt_user", "island_request_review"):
                text = "Dynamic Island AI Agent module is disabled in settings; request suppressed"
            elif tool_name == "island_update_status":
                send_ipc({
                    "target": "agent",
                    "state": arguments.get("state", "working"),
                    "message": arguments.get("message", "Working..."),
                    "name": arguments.get("name", "AI Assistant"),
                    "task": arguments.get("task"),
                    "progress": arguments.get("progress"),
                    "changed_files": arguments.get("changed_files")
                })
                text = f"Dynamic Island updated to [{arguments.get('state')}]"
            elif tool_name == "island_prompt_user":
                question = arguments.get("question", "Action required")
                send_ipc({
                    "target": "agent",
                    "state": "review",
                    "message": f"Action: {question[:45]}",
                })
                send_ipc({
                    "target": "notification",
                    "title": "Action Required",
                    "message": question[:100],
                    "icon": "message",
                    "duration_ms": 6000
                })
                text = "Dynamic Island prompt displayed to user"
            elif tool_name == "island_request_review":
                message = arguments.get("message", "Review requested")
                changed_files = arguments.get("changed_files")
                wait = arguments.get("wait", True)

                if wait:
                    res = send_ipc_wait({
                        "target": "agent",
                        "state": "review",
                        "message": message,
                        "changed_files": changed_files,
                        "wait": True,
                        "timeout_sec": 300
                    }, timeout=305)
                    decision = res.get("decision")
                    if decision == "approved":
                        text = "Decision: APPROVED by user on Dynamic Island. Proceed to apply/commit changes."
                    elif decision == "rejected":
                        text = "Decision: DISCARDED by user on Dynamic Island. Do not apply changes; ask user how to proceed."
                    elif decision == "timeout":
                        text = "Decision: TIMED OUT. User did not respond in 5 minutes."
                    else:
                        text = f"Decision: Error ({res.get('error', 'unknown')})"
                else:
                    send_ipc({
                        "target": "agent",
                        "state": "review",
                        "message": message,
                        "changed_files": changed_files
                    })
                    text = "Dynamic Island review card presented for human approval"
            elif tool_name == "island_notify":
                send_ipc({
                    "target": "notification",
                    "title": arguments.get("title", "Notice"),
                    "message": arguments.get("message", ""),
                    "icon": arguments.get("icon", "terminal")
                })
                text = "Dynamic Island notification displayed"
            else:
                text = f"Unknown tool: {tool_name}"

            resp = {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "content": [{"type": "text", "text": text}]
                }
            }
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()

def main():
    parser = argparse.ArgumentParser(description="Dynamic Island Developer CLI & Multi-Agent MCP Integration")
    subparsers = parser.add_subparsers(dest="subcommand", required=True)

    # Toggle sub-command
    subparsers.add_parser("toggle", help="Toggle Dynamic Island expanded/collapsed")

    # Screen sub-command
    subparsers.add_parser("screen", help="Cycle Dynamic Island to next monitor")

    # Position sub-command
    pos_parser = subparsers.add_parser("position", help="Set or cycle horizontal screen position (left, center, right, cycle)")
    pos_parser.add_argument("pos", nargs="?", default="cycle", choices=["left", "center", "right", "cycle"], help="Target position (default: cycle)")
    pos_parser.add_argument("--offset", "-o", type=float, default=None, help="Horizontal shift offset in pixels")

    # Settings sub-command
    subparsers.add_parser("settings", help="Open Settings & Preferences panel")

    # Hook sub-command (used internally by Antigravity lifecycle hooks)
    hook_parser = subparsers.add_parser("hook", help="Antigravity lifecycle hook handler")
    hook_parser.add_argument("event", choices=["pre-invocation", "pre-tool", "post-tool", "stop"], help="Hook event type")

    # MCP server sub-command
    subparsers.add_parser("mcp", help="Run Model Context Protocol (MCP) stdio server for AI agents")

    # Install sub-command
    install_parser = subparsers.add_parser("install", help="Install integrations (Antigravity, Cursor, Claude, Windsurf, OpenCode)")
    install_parser.add_argument("target", choices=["antigravity", "cursor", "claude", "windsurf", "opencode", "all"], help="Integration target to install")
    install_parser.add_argument("scope", nargs="?", default="global", choices=["global", "local", "workspace"], help="Install scope (default: global)")
    install_parser.add_argument("--global", "-g", dest="scope_global", action="store_true", help="Install globally")
    install_parser.add_argument("--local", "-l", dest="local", action="store_true", help="Install locally in current workspace")

    # Uninstall sub-command
    uninstall_parser = subparsers.add_parser("uninstall", help="Uninstall integrations")
    uninstall_parser.add_argument("target", choices=["antigravity", "cursor", "claude", "windsurf", "opencode", "all"], help="Integration target to uninstall")
    uninstall_parser.add_argument("scope", nargs="?", default="global", choices=["global", "local", "workspace"], help="Uninstall scope (default: global)")

    # Agent sub-command
    agent_parser = subparsers.add_parser("agent", help="Update AI Agent status")
    agent_parser.add_argument("--state", choices=["idle", "thinking", "working", "review", "error"], default="working")
    agent_parser.add_argument("--message", "-m", help="Status description")
    agent_parser.add_argument("--name", default="AI Assistant", help="Agent name")
    agent_parser.add_argument("--task", "-t", help="Active task title")
    agent_parser.add_argument("--progress", "-p", type=int, help="Progress percent (0-100)")
    agent_parser.add_argument("--files", "-f", help="Comma-separated modified files")
    agent_parser.add_argument("--wait", "-w", action="store_true", help="Block until human approves or discards on Dynamic Island")
    agent_parser.add_argument("--timeout", type=int, default=300, help="Timeout in seconds when waiting for approval (default: 300)")
    agent_parser.add_argument("--decision", action="store_true", help="Print latest review decision (approved/rejected/none)")
    agent_parser.add_argument("--approve", action="store_true", help="Approve active review")
    agent_parser.add_argument("--reject", action="store_true", help="Reject/discard active review")

    # Build sub-command
    build_parser = subparsers.add_parser("build", help="Update or track build status")
    build_parser.add_argument("--run", "-r", help="Run command, monitor its duration and exit code")
    build_parser.add_argument("--clear", action="store_true", help="Clear active build status back to idle")
    build_parser.add_argument("--state", choices=["idle", "running", "success", "failed"], default="running")
    build_parser.add_argument("--command", "-c", help="Command description")
    build_parser.add_argument("--duration", "-d", type=int, help="Duration in milliseconds")
    build_parser.add_argument("--code", type=int, help="Exit code")

    # Git sub-command
    git_parser = subparsers.add_parser("git", help="Query or refresh Git repository status HUD")
    git_parser.add_argument("path", nargs="?", default=None, help="Optional git repository directory")

    # Status sub-command
    subparsers.add_parser("status", help="Show current AI agent, build, editor, and git status")

    # Ports sub-command
    subparsers.add_parser("ports", help="List active listening ports and dev servers")

    # Docker sub-command
    subparsers.add_parser("docker", help="List and inspect Docker containers")

    # To-Do sub-command
    todo_parser = subparsers.add_parser("todo", help="Manage Dynamic Island tasks and to-do list")
    todo_sub = todo_parser.add_subparsers(dest="todo_action", required=True)

    # todo add
    todo_add = todo_sub.add_parser("add", help="Add a new task")
    todo_add.add_argument("title", help="Task title / description")
    todo_add.add_argument("--priority", "-p", choices=["high", "medium", "low"], default="medium", help="Priority level")
    todo_add.add_argument("--due", "-d", help="Due date or reminder (e.g. 'Tomorrow', '2026-11-01')")

    # todo list
    todo_sub.add_parser("list", help="List all tasks (pending and completed)")

    # todo toggle
    todo_toggle = todo_sub.add_parser("toggle", help="Toggle completion state of a task")
    todo_toggle.add_argument("id", help="Task ID (e.g. todo-1730000000)")

    # todo delete
    todo_del = todo_sub.add_parser("delete", help="Delete a task by ID")
    todo_del.add_argument("id", help="Task ID to delete")

    # todo clear-completed
    todo_sub.add_parser("clear-completed", help="Remove all finished tasks")

    # todo export
    todo_exp = todo_sub.add_parser("export", help="Export to markdown file")
    todo_exp.add_argument("--path", help="Custom markdown path (defaults to ~/todo.md)")

    # todo sync
    todo_sync = todo_sub.add_parser("sync", help="Sync tasks from a markdown file")
    todo_sync.add_argument("--path", help="Markdown file path (defaults to ~/todo.md)")

    # Notify sub-command
    notify_parser = subparsers.add_parser("notify", help="Send alert notification")
    notify_parser.add_argument("title", help="Notification title")
    notify_parser.add_argument("message", help="Notification message")
    notify_parser.add_argument("--app", default="CLI", help="Origin app name")
    notify_parser.add_argument("--icon", choices=["message", "terminal", "system"], default="terminal")
    notify_parser.add_argument("--duration", type=int, default=4500, help="Display duration in ms")

    # Focus sub-command
    p_focus = subparsers.add_parser("focus", help="Control Focus / Pomodoro Mode")
    focus_subs = p_focus.add_subparsers(dest="focus_action")

    p_focus_start = focus_subs.add_parser("start", help="Start a focus session")
    p_focus_start.add_argument("minutes", type=int, nargs="?", default=25, help="Duration in minutes (default: 25)")
    p_focus_start.add_argument("--label", "-l", help="Custom label or tag (e.g. 'Deep Work')")

    p_focus_stop = focus_subs.add_parser("stop", help="Stop / abort focus session")
    p_focus_pause = focus_subs.add_parser("pause", help="Pause focus timer")
    p_focus_resume = focus_subs.add_parser("resume", help="Resume focus timer")
    p_focus_open = focus_subs.add_parser("open", help="Open Focus Mode dashboard")

    # Prayer sub-command
    p_prayer = subparsers.add_parser("prayer", help="Islamic Prayer Times & Adhan Reminder")
    prayer_subs = p_prayer.add_subparsers(dest="prayer_action")

    prayer_subs.add_parser("status", help="Show full prayer schedule for today")
    prayer_subs.add_parser("next", help="Display next upcoming prayer and countdown")

    p_prayer_city = prayer_subs.add_parser("city", help="Set prayer calculation city")
    p_prayer_city.add_argument("city", help="City name (e.g. 'Surabaya', 'Jakarta', 'London')")

    p_prayer_test = prayer_subs.add_parser("test", help="Test trigger a prayer notification")
    p_prayer_test.add_argument("name", nargs="?", default="Maghrib", help="Prayer name to test (default: Maghrib)")

    # Meeting sub-command
    p_meeting = subparsers.add_parser("meeting", help="Active Meeting / Call status (Zoom, Discord, Meet, Slack, Teams)")
    meeting_subs = p_meeting.add_subparsers(dest="meeting_action")
    meeting_subs.add_parser("status", help="Detect active meeting & mic streams")
    p_meet_focus = meeting_subs.add_parser("focus", help="Bring meeting application to foreground")
    p_meet_focus.add_argument("platform", nargs="?", default="zoom", help="Platform name (zoom, discord, slack, teams, chrome)")

    args = parser.parse_args()

    if args.subcommand == "toggle":
        sys.exit(cmd_toggle(args))
    elif args.subcommand == "screen":
        sys.exit(cmd_screen(args))
    elif args.subcommand == "position":
        sys.exit(cmd_position(args))
    elif args.subcommand == "settings":
        sys.exit(cmd_settings(args))
    elif args.subcommand == "hook":
        sys.exit(cmd_hook(args))
    elif args.subcommand == "mcp":
        run_mcp_server()
    elif args.subcommand == "install":
        sys.exit(cmd_install(args))
    elif args.subcommand == "uninstall":
        sys.exit(cmd_uninstall(args))
    elif args.subcommand == "agent":
        sys.exit(cmd_agent(args))
    elif args.subcommand == "git":
        sys.exit(cmd_git(args))
    elif args.subcommand == "status":
        sys.exit(cmd_status(args))
    elif args.subcommand == "ports":
        sys.exit(cmd_ports(args))
    elif args.subcommand == "docker":
        sys.exit(cmd_docker(args))
    elif args.subcommand == "todo":
        sys.exit(cmd_todo(args))
    elif args.subcommand == "focus":
        sys.exit(cmd_focus(args))
    elif args.subcommand == "prayer":
        sys.exit(cmd_prayer(args))
    elif args.subcommand == "meeting":
        sys.exit(cmd_meeting(args))
    elif args.subcommand == "build":
        sys.exit(cmd_build(args))
    elif args.subcommand == "notify":
        sys.exit(cmd_notify(args))

if __name__ == "__main__":
    main()
