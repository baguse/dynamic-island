#!/usr/bin/env python3
"""
Antigravity Agent Lifecycle Hook for Linux Dynamic Island
Reads lifecycle events from stdin and broadcasts agent telemetry to the Dynamic Island
Unix Domain Socket without blocking or throwing errors if the island is offline.
"""

import os
import sys
import json
import socket

def get_socket_path():
    runtime_dir = os.environ.get("XDG_RUNTIME_DIR")
    if runtime_dir:
        return os.path.join(runtime_dir, "dynamic-island.sock")
    return f"/tmp/dynamic-island-{os.getuid()}.sock"

def send_ipc(payload):
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
            resp = client.recv(512).decode("utf-8").strip()
            return True
    except Exception:
        return False

def main():
    hook_type = sys.argv[1] if len(sys.argv) > 1 else "unknown"

    raw_input = ""
    try:
        raw_input = sys.stdin.read()
        data = json.loads(raw_input) if raw_input.strip() else {}
    except Exception:
        data = {}

    if hook_type == "pre-invocation":
        # Agent is beginning to think / plan
        send_ipc({
            "target": "agent",
            "name": "Antigravity Agent",
            "state": "thinking",
            "message": "Analyzing prompt & planning next steps...",
        })
        print(json.dumps({}))

    elif hook_type == "pre-tool":
        # Agent is executing a tool
        tool_call = data.get("toolCall", {})
        tool_name = tool_call.get("name", "tool")
        args = tool_call.get("args", {})

        message = f"Executing {tool_name}"
        changed_files = None

        if tool_name == "run_command":
            cmd = args.get("CommandLine", "").strip()
            first_line = cmd.split("\n")[0][:45]
            message = f"Terminal: {first_line}"
        elif tool_name in ("edit_file", "write_file"):
            path = args.get("Path", "")
            basename = os.path.basename(path) if path else "file"
            action = "Editing" if tool_name == "edit_file" else "Writing"
            message = f"{action} {basename}"
            if basename:
                changed_files = [basename]
        elif tool_name == "read_file":
            path = args.get("Path", "")
            basename = os.path.basename(path) if path else "file"
            message = f"Reading {basename}"
        elif tool_name == "search_web":
            q = args.get("query", "")[:35]
            message = f"Searching web: {q}"
        elif tool_name == "ask_question":
            message = "Waiting for user clarification"

        payload = {
            "target": "agent",
            "name": "Antigravity Agent",
            "state": "working",
            "message": message,
        }
        if changed_files:
            payload["changed_files"] = changed_files

        send_ipc(payload)
        # PreToolUse contract: must output {"decision": "allow"}
        print(json.dumps({"decision": "allow"}))

    elif hook_type == "post-tool":
        # PostToolUse contract: expect empty object
        print(json.dumps({}))

    elif hook_type == "stop":
        # Execution loop finished for this turn
        term_reason = data.get("terminationReason", "NO_TOOL_CALL")
        if term_reason == "NO_TOOL_CALL":
            send_ipc({
                "target": "agent",
                "name": "Antigravity Agent",
                "state": "idle",
                "message": "Ready for your next instruction",
            })
            # Also notify on completion
            send_ipc({
                "target": "notification",
                "title": "Antigravity Agent",
                "message": "Task step completed and ready for review",
                "app_name": "Antigravity",
                "icon": "terminal",
                "duration_ms": 4000
            })
        print(json.dumps({"decision": "allow"}))

    else:
        print(json.dumps({}))

if __name__ == "__main__":
    main()
