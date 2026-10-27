export type IslandMode = 'idle' | 'expanded' | 'agent' | 'media' | 'notification' | 'system' | 'settings' | 'git' | 'editor' | 'ports' | 'docker';

export interface IslandDimensions {
  width: number;
  height: number;
}

export type AgentState = 'idle' | 'thinking' | 'working' | 'review' | 'error';

export interface AgentStatus {
  name: string;
  state: AgentState;
  message: string;
  task?: string | null;
  progress?: number | null;
  changed_files?: string[] | null;
  is_demo?: boolean;
}

export type BuildState = 'idle' | 'running' | 'success' | 'failed';

export interface BuildStatus {
  command: string;
  state: BuildState;
  duration_ms?: number | null;
  message?: string | null;
  exit_code?: number | null;
  started_at?: number | null;
}

export interface EditorPresence {
  editor: string;
  workspace?: string | null;
  workspace_path?: string | null;
  file_name?: string | null;
  file_path?: string | null;
  line?: number | null;
  column?: number | null;
  language?: string | null;
  state: string;
  git_branch?: string | null;
  updated_at: number;
}

export interface DevStatusPayload {
  agent: AgentStatus;
  build: BuildStatus;
  editor?: EditorPresence | null;
}

export interface MediaMetadata {
  player_name: string;
  title: string;
  artist: string;
  album: string;
  art_url: string | null;
  status: 'Playing' | 'Paused' | 'Stopped';
  position_ms: number | null;
  length_ms: number | null;
}

export interface NotificationItem {
  id: string;
  title: string;
  message: string;
  app_name?: string;
  icon?: string;
  duration_ms?: number;
}

export interface MonitorInfo {
  name: string;
  is_current: boolean;
  width: number;
  height: number;
  scale_factor: number;
}

export interface BatteryTelemetry {
  percent: number;
  status: string;
  is_charging: boolean;
}

export interface SystemTelemetry {
  cpu_percent: number;
  core_count: number;
  mem_used_mb: number;
  mem_total_mb: number;
  mem_percent: number;
  battery: BatteryTelemetry | null;
  net_rx_speed_kb: number;
  net_tx_speed_kb: number;
}

export interface GitCommitInfo {
  hash: string;
  short_hash: string;
  message: string;
  author: string;
  relative_time: string;
}

export interface GitFileChange {
  path: string;
  status: string;
  is_staged: boolean;
}

export interface GitStatus {
  is_repo: boolean;
  repo_name: string | null;
  repo_path: string | null;
  branch: string | null;
  head_commit: GitCommitInfo | null;
  staged_count: number;
  unstaged_count: number;
  untracked_count: number;
  ahead: number;
  behind: number;
  changed_files: GitFileChange[];
  recent_commits: GitCommitInfo[];
}

export type IslandTheme = 'dark' | 'light' | 'neon' | 'anime';

export interface IslandSettings {
  theme: IslandTheme;
  hover_expand_enabled: boolean;
  hover_delay_ms: number;
  leave_delay_ms: number;
  inactivity_timeout_s: number;
  top_margin: number;
  show_system_in_pill: boolean;
  show_battery_alert: boolean;
  show_media_in_pill: boolean;
  show_build_in_pill: boolean;
  show_agent_in_pill: boolean;
  show_git_in_pill: boolean;
  show_editor_in_pill: boolean;
  show_ports_in_pill: boolean;
  show_docker_in_pill: boolean;
  watched_repo_path?: string | null;
  autostart_enabled: boolean;
}

export interface PortInfo {
  port: number;
  ip: string;
  process?: string | null;
  pid?: number | null;
  container?: string | null;
  category: 'web' | 'database' | 'service' | 'ide' | 'system';
  is_openable: boolean;
}

export interface DockerContainerInfo {
  id: string;
  name: string;
  image: string;
  state: string;
  status: string;
  ports: string;
  is_running: boolean;
}
