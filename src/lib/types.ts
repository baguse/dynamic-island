export type IslandMode = 'idle' | 'expanded' | 'agent' | 'media' | 'notification' | 'system' | 'settings' | 'git' | 'editor' | 'ports' | 'docker' | 'hardware' | 'clipboard' | 'shelf';

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

export interface AudioSpectrumData {
  bands: [number, number, number, number, number];
  energy: number;
}

export type SkyTheme = 'auto' | 'stars' | 'rain' | 'sakura' | 'clouds' | 'snow';
export type SkyIntensity = 'subtle' | 'medium' | 'vivid';

export interface WeatherData {
  temp_c: number;
  temp_f: number;
  condition: string;
  description: string;
  location: string;
  is_day: boolean;
  time_period: 'dawn' | 'day' | 'sunset' | 'night' | string;
  weather_code: number;
  humidity: number;
  cloud_cover: number;
  sunrise: string;
  sunset: string;
  updated_at: number;
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

export type IslandTheme =
  | 'dark'
  | 'light'
  | 'neon'
  | 'anime'
  | 'catppuccin'
  | 'nord'
  | 'dracula'
  | 'tokyo-night'
  | 'gruvbox'
  | 'matrix';
export type HorizontalPosition = 'center' | 'left' | 'right' | 'custom';

export const DEFAULT_MODULE_ORDER = [
  'media',
  'agent',
  'build',
  'system',
  'git',
  'editor',
  'ports',
  'docker',
  'mic',
  'clipboard',
  'shelf',
] as const;

export type ModuleKey = (typeof DEFAULT_MODULE_ORDER)[number];

export interface IslandSettings {
  theme: IslandTheme;
  hover_expand_enabled: boolean;
  hover_delay_ms: number;
  leave_delay_ms: number;
  inactivity_timeout_s: number;
  top_margin: number;
  horizontal_position: HorizontalPosition;
  x_offset: number;
  show_system_in_pill: boolean;
  show_battery_alert: boolean;
  show_media_in_pill: boolean;
  show_build_in_pill: boolean;
  show_agent_in_pill: boolean;
  show_git_in_pill: boolean;
  show_editor_in_pill: boolean;
  show_ports_in_pill: boolean;
  show_docker_in_pill: boolean;
  show_mic_in_pill: boolean;
  show_shelf_in_pill: boolean;
  show_clipboard_in_pill: boolean;
  module_order: string[];
  watched_repo_path?: string | null;
  show_media_companion: boolean;
  media_companion_path?: string | null;
  media_companion_style?: 'vinyl' | 'cassette';
  equalizer_style?: 'bars' | 'wave' | 'dots';
  ambient_sky_enabled?: boolean;
  ambient_sky_theme?: SkyTheme;
  ambient_sky_intensity?: SkyIntensity;
  weather_city?: string | null;
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

export interface AudioOutputDevice {
  id: string;
  name: string;
  description: string;
  device_type: 'speakers' | 'headphones' | 'other';
  is_active: boolean;
}

export interface AudioStatus {
  mic_muted: boolean;
  mic_volume: number;
  output_muted: boolean;
  output_volume: number;
  active_output_name: string;
  active_output_type: 'speakers' | 'headphones' | 'other';
  outputs: AudioOutputDevice[];
}

export interface ClipboardItem {
  id: string;
  content: string;
  preview: string;
  timestamp: number;
  char_count: number;
  line_count: number;
  is_snippet?: boolean;
}

export interface SnippetItem {
  id: string;
  title: string;
  content: string;
  category: string;
  created_at: number;
}

export interface ClipboardData {
  history: ClipboardItem[];
  snippets: SnippetItem[];
}

export interface ShelfItem {
  id: string;
  name: string;
  path?: string | null;
  text_content?: string | null;
  item_type: 'image' | 'code' | 'archive' | 'doc' | 'file' | 'url' | 'text';
  size_bytes?: number | null;
  size_formatted?: string | null;
  extension?: string | null;
  timestamp: number;
}
