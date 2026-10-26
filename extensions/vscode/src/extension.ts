import * as vscode from 'vscode';
import * as net from 'net';
import * as path from 'path';
import * as fs from 'fs';

interface EditorTelemetry {
  target: 'editor';
  editor: string;
  workspace?: string | null;
  workspace_path?: string | null;
  file_name?: string | null;
  file_path?: string | null;
  line?: number | null;
  column?: number | null;
  language?: string | null;
  state: 'editing' | 'viewing' | 'debugging' | 'idle';
  git_branch?: string | null;
}

class DynamicIslandBridge {
  private client: net.Socket | null = null;
  private isConnected = false;
  private reconnectTimer: NodeJS.Timeout | null = null;
  private sendDebounceTimer: NodeJS.Timeout | null = null;
  private statusBarItem: vscode.StatusBarItem;
  private isDisposed = false;

  constructor() {
    this.statusBarItem = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Right,
      100
    );
    this.statusBarItem.command = 'dynamicIsland.toggle';
    this.updateStatusBar(false);
    this.statusBarItem.show();
    this.connect();
  }

  private getSocketPath(): string {
    const runtimeDir = process.env.XDG_RUNTIME_DIR;
    if (runtimeDir) {
      const p = path.join(runtimeDir, 'dynamic-island.sock');
      if (fs.existsSync(p)) return p;
    }

    const uid = process.getuid ? process.getuid() : 1000;
    const uidSock = `/tmp/dynamic-island-${uid}.sock`;
    if (fs.existsSync(uidSock)) return uidSock;

    return '/tmp/dynamic-island.sock';
  }

  public connect() {
    if (this.isDisposed) return;
    if (this.client) {
      try {
        this.client.destroy();
      } catch {
        // ignore
      }
      this.client = null;
    }

    const sockPath = this.getSocketPath();
    if (!fs.existsSync(sockPath)) {
      this.updateStatusBar(false);
      this.scheduleReconnect(3000);
      return;
    }

    try {
      this.client = net.createConnection(sockPath);
      this.client.setNoDelay(true);

      this.client.on('connect', () => {
        this.isConnected = true;
        this.updateStatusBar(true);
        this.broadcastPresence();
      });

      this.client.on('data', () => {
        // responses from dynamic island
      });

      this.client.on('error', () => {
        this.isConnected = false;
        this.updateStatusBar(false);
      });

      this.client.on('close', () => {
        this.isConnected = false;
        this.updateStatusBar(false);
        this.scheduleReconnect(2500);
      });
    } catch {
      this.isConnected = false;
      this.updateStatusBar(false);
      this.scheduleReconnect(3000);
    }
  }

  private scheduleReconnect(delayMs: number) {
    if (this.isDisposed) return;
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.reconnectTimer = setTimeout(() => {
      this.connect();
    }, delayMs);
  }

  private updateStatusBar(connected: boolean) {
    if (connected) {
      this.statusBarItem.text = `$(circle-filled) Island`;
      this.statusBarItem.tooltip = `Dynamic Island: Connected (Click to toggle)`;
      this.statusBarItem.color = new vscode.ThemeColor('testing.iconPassed');
    } else {
      this.statusBarItem.text = `$(circle-outline) Island`;
      this.statusBarItem.tooltip = `Dynamic Island: Disconnected (Click to reconnect)`;
      this.statusBarItem.color = undefined;
    }
  }

  public send(payload: Record<string, unknown>) {
    if (!this.isConnected || !this.client) {
      // If disconnected, try a quick reconnect
      this.connect();
      return;
    }

    try {
      const line = JSON.stringify(payload) + '\n';
      this.client.write(line);
    } catch {
      this.isConnected = false;
      this.updateStatusBar(false);
      this.scheduleReconnect(1500);
    }
  }

  public broadcastPresence() {
    if (this.sendDebounceTimer) {
      clearTimeout(this.sendDebounceTimer);
    }

    this.sendDebounceTimer = setTimeout(() => {
      this.dispatchPresenceNow();
    }, 200);
  }

  private getGitBranch(): string | null {
    try {
      const gitExtension = vscode.extensions.getExtension('vscode.git');
      if (gitExtension && gitExtension.isActive) {
        const git = gitExtension.exports.getAPI(1);
        if (git && git.repositories && git.repositories.length > 0) {
          const repo = git.repositories[0];
          return repo.state.HEAD?.name || null;
        }
      }
    } catch {
      // ignore
    }
    return null;
  }

  private dispatchPresenceNow() {
    const config = vscode.workspace.getConfiguration('dynamicIsland');
    if (!config.get<boolean>('enabled', true)) {
      return;
    }

    const editor = vscode.window.activeTextEditor;
    const isDebugging = !!vscode.debug.activeDebugSession;
    const isFocused = vscode.window.state.focused;

    let state: 'editing' | 'viewing' | 'debugging' | 'idle' = 'editing';
    if (!isFocused) {
      state = 'idle';
    } else if (isDebugging) {
      state = 'debugging';
    } else if (!editor) {
      state = 'idle';
    }

    const editorName = vscode.env.appName.includes('Cursor')
      ? 'Cursor'
      : vscode.env.appName.includes('Code')
      ? 'VS Code'
      : vscode.env.appName;

    const workspaceName = config.get<boolean>('showWorkspace', true)
      ? vscode.workspace.name || null
      : null;

    let workspacePath: string | null = null;
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (workspaceFolders && workspaceFolders.length > 0) {
      const f = workspaceFolders[0];
      workspacePath = f.uri.fsPath || f.uri.path || null;
    }
    if (!workspacePath && (vscode.workspace as any).rootPath) {
      workspacePath = (vscode.workspace as any).rootPath;
    }

    let fileName: string | null = null;
    let filePath: string | null = null;
    let line: number | null = null;
    let column: number | null = null;
    let language: string | null = null;

    if (editor) {
      const doc = editor.document;
      fileName = path.basename(doc.fileName);
      filePath = vscode.workspace.asRelativePath(doc.uri, false);
      language = doc.languageId;

      if (!workspacePath) {
        const folder = vscode.workspace.getWorkspaceFolder(doc.uri);
        if (folder) {
          workspacePath = folder.uri.fsPath || folder.uri.path || null;
        }
      }

      if (!workspacePath && doc.fileName && filePath && doc.fileName.endsWith(filePath)) {
        workspacePath = doc.fileName.slice(0, -filePath.length).replace(/\/$/, '') || null;
      }

      if (!workspacePath && doc.fileName) {
        workspacePath = path.dirname(doc.fileName);
      }

      if (config.get<boolean>('showLineNumber', true) && editor.selection) {
        line = editor.selection.active.line + 1;
        column = editor.selection.active.character + 1;
      }
    }

    const payload: EditorTelemetry = {
      target: 'editor',
      editor: editorName,
      workspace: workspaceName,
      workspace_path: workspacePath,
      file_name: fileName,
      file_path: filePath,
      line,
      column,
      language,
      state,
      git_branch: this.getGitBranch(),
    };

    this.send(payload as unknown as Record<string, unknown>);
  }

  public dispose() {
    this.isDisposed = true;
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    if (this.sendDebounceTimer) clearTimeout(this.sendDebounceTimer);

    // Send idle on dispose
    try {
      if (this.isConnected && this.client) {
        const payload = {
          target: 'editor',
          editor: vscode.env.appName,
          state: 'idle',
        };
        this.client.write(JSON.stringify(payload) + '\n');
        this.client.destroy();
      }
    } catch {
      // ignore
    }

    this.statusBarItem.dispose();
  }
}

let bridge: DynamicIslandBridge | null = null;

export function activate(context: vscode.ExtensionContext) {
  bridge = new DynamicIslandBridge();

  context.subscriptions.push(
    vscode.commands.registerCommand('dynamicIsland.toggle', () => {
      if (bridge) {
        bridge.send({ target: 'toggle' });
      }
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('dynamicIsland.reconnect', () => {
      if (bridge) {
        bridge.connect();
      }
    })
  );

  // Monitor text editor and cursor changes
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor(() => {
      bridge?.broadcastPresence();
    })
  );

  context.subscriptions.push(
    vscode.window.onDidChangeTextEditorSelection(() => {
      bridge?.broadcastPresence();
    })
  );

  context.subscriptions.push(
    vscode.workspace.onDidChangeTextDocument((e) => {
      if (vscode.window.activeTextEditor?.document === e.document) {
        bridge?.broadcastPresence();
      }
    })
  );

  // Debug session lifecycle
  context.subscriptions.push(
    vscode.debug.onDidStartDebugSession(() => {
      bridge?.broadcastPresence();
    })
  );

  context.subscriptions.push(
    vscode.debug.onDidTerminateDebugSession(() => {
      bridge?.broadcastPresence();
    })
  );

  // Window focus state
  context.subscriptions.push(
    vscode.window.onDidChangeWindowState(() => {
      bridge?.broadcastPresence();
    })
  );

  // Config changes
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration('dynamicIsland')) {
        bridge?.broadcastPresence();
      }
    })
  );
}

export function deactivate() {
  if (bridge) {
    bridge.dispose();
    bridge = null;
  }
}
