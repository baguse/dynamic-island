import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
  DEFAULT_MODULE_ORDER,
  type IslandMode,
  type IslandDimensions,
  type MediaMetadata,
  type NotificationItem,
  type MonitorInfo,
  type SystemTelemetry,
  type AgentStatus,
  type BuildStatus,
  type DevStatusPayload,
  type IslandSettings,
  type IslandTheme,
  type GitStatus,
  type EditorPresence,
  type PortInfo,
  type DockerContainerInfo,
  type AudioStatus,
  type AudioOutputDevice,
  type ClipboardData,
  type ClipboardItem,
  type SnippetItem,
  type ShelfItem,
  type ModuleKey,
  type AudioSpectrumData,
  type WeatherData,
  type SkyTheme,
  type SkyIntensity,
} from '../types';

export const DEFAULT_SETTINGS: IslandSettings = {
  theme: 'dark',
  hover_expand_enabled: true,
  hover_delay_ms: 220,
  leave_delay_ms: 450,
  inactivity_timeout_s: 12,
  top_margin: 12.0,
  horizontal_position: 'center',
  x_offset: 0,
  show_system_in_pill: true,
  show_battery_alert: true,
  show_media_in_pill: true,
  show_build_in_pill: true,
  show_agent_in_pill: true,
  show_git_in_pill: true,
  show_editor_in_pill: true,
  show_ports_in_pill: true,
  show_docker_in_pill: true,
  show_mic_in_pill: true,
  show_shelf_in_pill: true,
  show_clipboard_in_pill: true,
  module_order: [...DEFAULT_MODULE_ORDER],
  watched_repo_path: null,
  show_media_companion: true,
  media_companion_path: null,
  media_companion_style: 'vinyl',
  equalizer_style: 'bars',
  ambient_sky_enabled: true,
  ambient_sky_theme: 'auto',
  ambient_sky_intensity: 'subtle',
  weather_city: null,
  mascot_enabled: true,
  mascot_type: 'cat',
  mascot_interactive: true,
  autostart_enabled: false,
};

export const DASHBOARD_WIDTH = 800;
export const DASHBOARD_HEIGHT = 500;

export const BASE_DIMENSIONS: Record<IslandMode, IslandDimensions> = {
  idle: { width: 340, height: 52 },
  expanded: { width: DASHBOARD_WIDTH, height: DASHBOARD_HEIGHT },
  agent: { width: DASHBOARD_WIDTH, height: DASHBOARD_HEIGHT },
  media: { width: DASHBOARD_WIDTH, height: DASHBOARD_HEIGHT },
  notification: { width: 420, height: 96 },
  system: { width: DASHBOARD_WIDTH, height: DASHBOARD_HEIGHT },
  settings: { width: DASHBOARD_WIDTH, height: DASHBOARD_HEIGHT },
  git: { width: DASHBOARD_WIDTH, height: DASHBOARD_HEIGHT },
  editor: { width: DASHBOARD_WIDTH, height: DASHBOARD_HEIGHT },
  ports: { width: DASHBOARD_WIDTH, height: DASHBOARD_HEIGHT },
  docker: { width: DASHBOARD_WIDTH, height: DASHBOARD_HEIGHT },
  hardware: { width: DASHBOARD_WIDTH, height: DASHBOARD_HEIGHT },
  clipboard: { width: DASHBOARD_WIDTH, height: DASHBOARD_HEIGHT },
  shelf: { width: DASHBOARD_WIDTH, height: DASHBOARD_HEIGHT },
};

class IslandStore {
  mode = $state<IslandMode>('idle');
  previousMode: IslandMode = 'idle';
  activePanel = $state<IslandMode | null>(null);
  lastActiveDashboardModule = $state<ModuleKey>('media');
  isSyncing = $state<boolean>(false);
  isExpanding = $state<boolean>(false);
  isCollapsing = $state<boolean>(false);
  private collapseAnimationTimer: ReturnType<typeof setTimeout> | null = null;
  private expandAnimationTimer: ReturnType<typeof setTimeout> | null = null;
  currentMedia = $state<MediaMetadata | null>(null);
  weather = $state<WeatherData | null>(null);
  audioSpectrum = $state<AudioSpectrumData>({
    bands: [0, 0, 0, 0, 0],
    energy: 0,
  });
  activeNotification = $state<NotificationItem | null>(null);
  monitors = $state<MonitorInfo[]>([]);
  telemetry = $state<SystemTelemetry | null>(null);
  settings = $state<IslandSettings>({ ...DEFAULT_SETTINGS });
  agent = $state<AgentStatus>({
    name: 'Antigravity Agent',
    state: 'idle',
    message: 'Ready for your next task',
    is_demo: false,
  });
  build = $state<BuildStatus>({
    command: '',
    state: 'idle',
  });
  git = $state<GitStatus>({
    is_repo: false,
    repo_name: null,
    repo_path: null,
    branch: null,
    head_commit: null,
    staged_count: 0,
    unstaged_count: 0,
    untracked_count: 0,
    ahead: 0,
    behind: 0,
    changed_files: [],
    recent_commits: [],
  });
  editor = $state<EditorPresence | null>(null);
  ports = $state<PortInfo[]>([]);
  dockerContainers = $state<DockerContainerInfo[]>([]);
  audio = $state<AudioStatus | null>(null);
  clipboard = $state<ClipboardData>({ history: [], snippets: [] });
  shelf = $state<ShelfItem[]>([]);
  isDragOverShelf = $state<boolean>(false);

  get webPortsCount(): number {
    return this.ports.filter((p) => p.category === 'web').length;
  }

  get runningDockerCount(): number {
    return this.dockerContainers.filter((c) => c.is_running).length;
  }

  get isMicMuted(): boolean {
    return this.audio?.mic_muted ?? false;
  }

  get shelfCount(): number {
    return this.shelf.length;
  }

  get clipboardHistoryCount(): number {
    return this.clipboard.history.length;
  }

  // Hover & Inactivity Settings
  hoverExpandDelayMs = 220;    // Hover intent delay to expand
  leaveCollapseDelayMs = 450;  // Grace delay after mouse leaves panel
  autoCollapseDelayMs = 12000; // Inactivity fallback (12s)

  private hoverExpandTimeout: ReturnType<typeof setTimeout> | null = null;
  private leaveCollapseTimeout: ReturnType<typeof setTimeout> | null = null;
  private autoCollapseTimeout: ReturnType<typeof setTimeout> | null = null;
  private isHovered = false;
  private lastActivityTime = 0;

  get isCursorOverSurface(): boolean {
    if (this.isHovered) {
      return true;
    }
    if (typeof document !== 'undefined') {
      const el = document.querySelector('.island-surface');
      if (el?.matches(':hover')) {
        this.isHovered = true;
        return true;
      }
    }
    return false;
  }

  private unlistenMedia: UnlistenFn | null = null;
  private unlistenAudioSpectrum: UnlistenFn | null = null;
  private unlistenWeather: UnlistenFn | null = null;
  private unlistenNotification: UnlistenFn | null = null;
  private unlistenTelemetry: UnlistenFn | null = null;
  private unlistenAgent: UnlistenFn | null = null;
  private unlistenBuild: UnlistenFn | null = null;
  private unlistenGit: UnlistenFn | null = null;
  private unlistenEditor: UnlistenFn | null = null;
  private unlistenTrayToggle: UnlistenFn | null = null;
  private unlistenTrayCycle: UnlistenFn | null = null;
  private unlistenTraySettings: UnlistenFn | null = null;
  private unlistenAudio: UnlistenFn | null = null;
  private unlistenClipboard: UnlistenFn | null = null;
  private unlistenSettings: UnlistenFn | null = null;
  private unlistenTrayReposition: UnlistenFn | null = null;
  private notificationTimeout: ReturnType<typeof setTimeout> | null = null;

  get isExpanded(): boolean {
    return this.mode !== 'idle' && !this.isCollapsing;
  }

  get hasActiveMedia(): boolean {
    return (
      this.currentMedia !== null &&
      (this.currentMedia.status === 'Playing' || this.currentMedia.status === 'Paused')
    );
  }

  get isPlaying(): boolean {
    return this.currentMedia?.status === 'Playing';
  }

  get hasActiveBuild(): boolean {
    return this.build.state === 'running';
  }

  get hasActiveAgent(): boolean {
    if (!this.settings.show_agent_in_pill) {
      return false;
    }
    return (
      this.agent.state === 'thinking' ||
      this.agent.state === 'working' ||
      this.agent.state === 'review'
    );
  }

  get hasActiveGit(): boolean {
    return this.git.is_repo && !!this.git.branch;
  }

  get gitDirtyCount(): number {
    return this.git.staged_count + this.git.unstaged_count + this.git.untracked_count;
  }

  get hasActiveEditor(): boolean {
    return (
      this.editor !== null &&
      this.editor.state !== 'idle' &&
      !!this.editor.file_name &&
      Date.now() - (this.editor.updated_at * 1000) < 300000
    );
  }

  get currentMonitorName(): string {
    return this.monitors.find((m) => m.is_current)?.name || 'Default';
  }

  get activeOutputDeviceName(): string {
    return this.audio?.outputs?.find((o) => o.is_active)?.name || 'Audio Output';
  }

  get isBatteryLow(): boolean {
    return !!(
      this.telemetry?.battery &&
      this.telemetry.battery.percent <= 20 &&
      !this.telemetry.battery.is_charging
    );
  }

  get dimensions(): IslandDimensions {
    return {
      width: this.targetWidth,
      height: this.targetHeight,
    };
  }

  get moduleOrder(): ModuleKey[] {
    const saved = this.settings.module_order || [];
    const knownKeys = DEFAULT_MODULE_ORDER as readonly string[];
    const result: ModuleKey[] = [];
    for (const key of saved) {
      if (knownKeys.includes(key) && !result.includes(key as ModuleKey)) {
        result.push(key as ModuleKey);
      }
    }
    for (const key of DEFAULT_MODULE_ORDER) {
      if (!result.includes(key)) {
        result.push(key);
      }
    }
    return result;
  }

  get activePillModule(): ModuleKey | null {
    for (const key of this.moduleOrder) {
      if (key === 'build' && this.hasActiveBuild && this.settings.show_build_in_pill) return 'build';
      if (key === 'agent' && this.hasActiveAgent) return 'agent';
      if (key === 'media' && this.hasActiveMedia && this.settings.show_media_in_pill) return 'media';
      if (key === 'git' && this.hasActiveGit && this.settings.show_git_in_pill) return 'git';
      if (key === 'editor' && this.hasActiveEditor && this.settings.show_editor_in_pill) return 'editor';
    }
    return null;
  }

  get targetWidth(): number {
    if (this.mode === 'idle' || this.isCollapsing) {
      const active = this.activePillModule;
      if (active === 'agent') return 340;
      if (active === 'editor') return 340;
      if (active === 'build') return 330;
      if (active === 'media') return 330;
      if (active === 'git') return 330;
      if (this.isMicMuted && this.settings.show_mic_in_pill) return 320;
      if (this.shelfCount > 0 && this.settings.show_shelf_in_pill) return 320;
      return 300;
    }
    if (this.mode === 'notification') {
      return 420;
    }
    return DASHBOARD_WIDTH;
  }

  get targetHeight(): number {
    if (this.mode === 'idle' || this.isCollapsing) {
      return 52;
    }
    if (this.mode === 'notification') {
      return 96;
    }
    return DASHBOARD_HEIGHT;
  }

  async setMode(newMode: IslandMode) {
    if (this.mode === newMode && !this.isCollapsing) return;

    if (newMode === 'idle') {
      await this.collapse();
      return;
    }

    if (newMode !== 'notification') {
      this.lastActiveDashboardModule = (newMode === 'expanded' ? 'system' : newMode) as ModuleKey;
    }

    if (this.mode === 'idle') {
      await this.expandToMode(newMode);
      return;
    }

    // Switching between open dashboard modules (e.g. media -> agent)
    this.previousMode = this.mode;
    this.mode = newMode;
    this.activePanel = newMode;
    this.resetInactivityTimer();
    const neededWidth = this.targetWidth;
    const neededHeight = this.targetHeight;
    await this.syncNativeWindow(neededWidth, neededHeight + 14);
  }

  async expandToMode(targetMode: IslandMode) {
    if (this.collapseAnimationTimer) {
      clearTimeout(this.collapseAnimationTimer);
      this.collapseAnimationTimer = null;
    }

    this.clearHoverTimers();
    await this.refreshMonitors();

    // Set mode and trigger expand animation
    this.previousMode = this.mode;
    this.mode = targetMode;
    this.activePanel = targetMode;
    if (targetMode !== 'notification') {
      this.lastActiveDashboardModule = (targetMode === 'expanded' ? 'system' : targetMode) as ModuleKey;
    }
    this.isExpanding = true;
    this.isCollapsing = false;
    this.isHovered = this.isCursorOverSurface;
    this.resetInactivityTimer();

    // 1. Immediately expand the native window height and width so canvas is ready
    const neededWidth = this.targetWidth;
    const neededHeight = this.targetHeight;
    await this.syncNativeWindow(neededWidth, neededHeight + 14);

    if (this.expandAnimationTimer) clearTimeout(this.expandAnimationTimer);
    this.expandAnimationTimer = setTimeout(() => {
      this.isExpanding = false;
      this.expandAnimationTimer = null;
    }, 340);
  }

  async expand() {
    let targetMode: IslandMode;
    if (this.hasActiveAgent) {
      targetMode = 'agent';
    } else if (this.hasActiveMedia) {
      targetMode = 'media';
    } else if (this.hasActiveBuild) {
      targetMode = 'agent';
    } else if (this.lastActiveDashboardModule && this.lastActiveDashboardModule !== 'build') {
      targetMode = this.lastActiveDashboardModule as IslandMode;
    } else {
      targetMode = 'media';
    }
    await this.expandToMode(targetMode);
  }

  async openEditor() {
    await this.setMode('editor');
  }

  async updateAgentStatus(status: Partial<AgentStatus>) {
    this.resetInactivityTimer();
    const updated: AgentStatus = {
      ...this.agent,
      ...status,
    };
    try {
      await invoke('set_agent_status', { status: updated });
    } catch {
      this.agent = updated;
    }
  }

  // Watchdog timer for background builds
  private buildWatchdogTimer: ReturnType<typeof setTimeout> | null = null;

  private checkBuildWatchdog() {
    if (this.buildWatchdogTimer) {
      clearTimeout(this.buildWatchdogTimer);
      this.buildWatchdogTimer = null;
    }
    if (this.build.state === 'running') {
      const MAX_BUILD_DURATION_MS = 180_000; // 3 minutes auto-clear timeout
      let remaining = MAX_BUILD_DURATION_MS;
      if (this.build.started_at) {
        const elapsed = Date.now() - this.build.started_at;
        remaining = Math.max(1000, MAX_BUILD_DURATION_MS - elapsed);
      }
      this.buildWatchdogTimer = setTimeout(async () => {
        if (this.build.state === 'running') {
          console.log('[Dynamic Island] Auto-clearing stuck build after 3m timeout');
          await this.clearBuild();
        }
      }, remaining);
    }
  }

  async clearBuild() {
    if (this.buildWatchdogTimer) {
      clearTimeout(this.buildWatchdogTimer);
      this.buildWatchdogTimer = null;
    }
    const cleared: BuildStatus = {
      command: '',
      state: 'idle',
      duration_ms: null,
      message: 'Cleared by user/timeout',
      exit_code: null,
      started_at: null,
    };
    this.build = cleared;
    try {
      await invoke('set_build_status', { status: cleared });
    } catch (e) {
      console.debug('Failed to set build status:', e);
    }
    if (this.mode === 'idle') {
      await this.syncNativeWindow();
    }
  }

  async updateBuildStatus(status: Partial<BuildStatus>) {
    this.resetInactivityTimer();
    const updated: BuildStatus = {
      ...this.build,
      ...status,
    };
    try {
      await invoke('set_build_status', { status: updated });
    } catch {
      this.build = updated;
    }
    this.checkBuildWatchdog();
  }

  async approveAgentReview() {
    this.resetInactivityTimer();
    this.agent = {
      ...this.agent,
      state: 'working',
      message: 'Changes approved! Proceeding with task...',
      progress: 90,
    };
    try {
      await invoke('submit_review_decision', { decision: true });
    } catch (err) {
      console.debug('submit_review_decision error:', err);
    }
  }

  async rejectAgentReview() {
    this.resetInactivityTimer();
    this.agent = {
      ...this.agent,
      state: 'idle',
      message: 'Changes discarded by user.',
      progress: null,
      changed_files: null,
    };
    try {
      await invoke('submit_review_decision', { decision: false });
    } catch (err) {
      console.debug('submit_review_decision error:', err);
    }
  }

  async collapse() {
    if (this.mode === 'idle' || this.isCollapsing) return;

    this.clearHoverTimers();
    this.clearInactivityTimer();

    if (this.expandAnimationTimer) {
      clearTimeout(this.expandAnimationTimer);
      this.expandAnimationTimer = null;
    }

    // 1. Mark collapsing to initiate the smooth spring-back
    this.isCollapsing = true;
    this.isExpanding = false;
    this.previousMode = this.mode;

    if (this.collapseAnimationTimer) clearTimeout(this.collapseAnimationTimer);
    // 2. Keep native window open until spring animation completes, then lock to compact bounds
    this.collapseAnimationTimer = setTimeout(async () => {
      this.mode = 'idle';
      this.activePanel = null;
      this.isCollapsing = false;
      this.collapseAnimationTimer = null;
      await this.syncNativeWindow();
    }, 340);
  }

  async toggle() {
    if (this.mode === 'idle' || this.isCollapsing) {
      await this.expand();
    } else {
      await this.collapse();
    }
  }

  async openAgent() {
    if (!this.settings.show_agent_in_pill) return;
    await this.setMode('agent');
  }

  async openMedia() {
    await this.setMode('media');
  }

  async openSystem() {
    await this.refreshTelemetry();
    await this.setMode('system');
  }

  async openSettings() {
    await this.setMode('settings');
  }

  async openGit() {
    await this.refreshGitStatus();
    await this.setMode('git');
  }

  async openPorts() {
    await this.refreshPorts();
    await this.setMode('ports');
  }

  async openDocker() {
    await this.refreshDocker();
    await this.setMode('docker');
  }

  async refreshPorts() {
    try {
      const data = await invoke<PortInfo[]>('get_listening_ports');
      if (data) {
        this.ports = data;
      }
    } catch (err) {
      console.debug('Failed to fetch listening ports:', err);
    }
  }

  async killPortProcess(pid: number) {
    try {
      await invoke('kill_port_process', { pid });
      await this.refreshPorts();
    } catch (err) {
      console.error('Failed to kill process:', err);
    }
  }

  async refreshDocker() {
    try {
      const data = await invoke<DockerContainerInfo[]>('get_docker_containers');
      if (data) {
        this.dockerContainers = data;
      }
    } catch (err) {
      console.debug('Failed to fetch docker containers:', err);
    }
  }

  async restartDocker(id: string) {
    try {
      await invoke('docker_restart_container', { id });
      await this.refreshDocker();
    } catch (err) {
      console.error('Failed to restart container:', err);
    }
  }

  async stopDocker(id: string) {
    try {
      await invoke('docker_stop_container', { id });
      await this.refreshDocker();
    } catch (err) {
      console.error('Failed to stop container:', err);
    }
  }

  async startDocker(id: string) {
    try {
      await invoke('docker_start_container', { id });
      await this.refreshDocker();
    } catch (err) {
      console.error('Failed to start container:', err);
    }
  }

  async openBrowser(url: string) {
    try {
      await invoke('open_url_in_browser', { url });
    } catch (err) {
      console.error('Failed to open browser:', err);
    }
  }

  async refreshGitStatus() {
    try {
      const data = await invoke<GitStatus>('refresh_git_status', {
        path: this.settings.watched_repo_path || null,
      });
      if (data) {
        this.git = data;
      }
    } catch (err) {
      console.debug('refreshGitStatus note:', err);
    }
  }

  // --- Hardware & Audio Controls ---

  async openHardware() {
    await this.refreshAudio();
    await this.setMode('hardware');
  }

  async refreshAudio() {
    try {
      const status = await invoke<AudioStatus>('get_audio_status');
      if (status) {
        this.audio = status;
      }
    } catch (err) {
      console.debug('Failed to get audio status:', err);
    }
  }

  async toggleMicMute() {
    try {
      const muted = await invoke<boolean>('toggle_mic_mute');
      if (this.audio) {
        this.audio.mic_muted = muted;
      } else {
        await this.refreshAudio();
      }
    } catch (err) {
      console.error('Failed to toggle mic mute:', err);
    }
  }

  async setMicVolume(volume: number) {
    try {
      await invoke('set_mic_volume', { volume: Math.round(volume) });
      if (this.audio) {
        this.audio.mic_volume = Math.round(volume);
      }
    } catch (err) {
      console.error('Failed to set mic volume:', err);
    }
  }

  async setOutputVolume(volume: number) {
    try {
      await invoke('set_output_volume', { volume: Math.round(volume) });
      if (this.audio) {
        this.audio.output_volume = Math.round(volume);
      }
    } catch (err) {
      console.error('Failed to set output volume:', err);
    }
  }

  async toggleOutputMute() {
    try {
      const muted = await invoke<boolean>('toggle_output_mute');
      if (this.audio) {
        this.audio.output_muted = muted;
      }
    } catch (err) {
      console.error('Failed to toggle output mute:', err);
    }
  }

  async setAudioOutput(id: string) {
    try {
      await invoke('set_audio_output', { id });
      await this.refreshAudio();
    } catch (err) {
      console.error('Failed to set audio output:', err);
    }
  }

  // --- Quick Clipboard & Snippets ---

  async openClipboard() {
    await this.refreshClipboard();
    await this.setMode('clipboard');
  }

  async refreshClipboard() {
    try {
      const data = await invoke<ClipboardData>('get_clipboard_data');
      if (data) {
        this.clipboard = data;
      }
    } catch (err) {
      console.debug('Failed to get clipboard data:', err);
    }
  }

  async copyToClipboard(text: string): Promise<boolean> {
    try {
      await invoke('copy_to_clipboard', { content: text });
      await this.refreshClipboard();
      return true;
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
      return false;
    }
  }

  async addSnippet(title: string, content: string, category: string = 'General') {
    try {
      await invoke('add_snippet', { title, content, category });
      await this.refreshClipboard();
    } catch (err) {
      console.error('Failed to add snippet:', err);
    }
  }

  async deleteSnippet(id: string) {
    try {
      await invoke('delete_snippet', { id });
      await this.refreshClipboard();
    } catch (err) {
      console.error('Failed to delete snippet:', err);
    }
  }

  async deleteClipboardItem(id: string) {
    try {
      await invoke('delete_clipboard_item', { id });
      await this.refreshClipboard();
    } catch (err) {
      console.error('Failed to delete clipboard item:', err);
    }
  }

  async clearClipboardHistory() {
    try {
      await invoke('clear_clipboard_history');
      await this.refreshClipboard();
    } catch (err) {
      console.error('Failed to clear clipboard history:', err);
    }
  }

  // --- Desktop Drop Shelf ---

  async openShelf() {
    await this.refreshShelf();
    await this.setMode('shelf');
  }

  async refreshShelf() {
    try {
      const items = await invoke<ShelfItem[]>('get_shelf_items');
      if (items) {
        this.shelf = items;
      }
    } catch (err) {
      console.debug('Failed to get shelf items:', err);
    }
  }

  async addShelfPaths(paths: string[]) {
    try {
      const items = await invoke<ShelfItem[]>('add_shelf_paths', { paths });
      if (items) {
        this.shelf = items;
      }
    } catch (err) {
      console.error('Failed to add shelf paths:', err);
    }
  }

  async addShelfText(content: string, title?: string) {
    try {
      const item = await invoke<ShelfItem>('add_shelf_text', {
        content,
        title: title || null,
      });
      if (item) {
        await this.refreshShelf();
      }
    } catch (err) {
      console.error('Failed to add shelf text:', err);
    }
  }

  async removeShelfItem(id: string) {
    try {
      await invoke('remove_shelf_item', { id });
      await this.refreshShelf();
    } catch (err) {
      console.error('Failed to remove shelf item:', err);
    }
  }

  async clearShelf() {
    try {
      await invoke('clear_shelf');
      this.shelf = [];
    } catch (err) {
      console.error('Failed to clear shelf:', err);
    }
  }

  async openShelfItemInFolder(path: string) {
    try {
      await invoke('open_shelf_item_in_file_manager', { path });
    } catch (err) {
      console.error('Failed to open shelf item:', err);
    }
  }

  async copyShelfFileToFileManager(path: string): Promise<boolean> {
    try {
      await invoke('copy_shelf_file_to_file_manager', { path });
      return true;
    } catch (err) {
      console.error('Failed to copy shelf file for file manager:', err);
      return false;
    }
  }

  async updateSettings(partial: Partial<IslandSettings>) {
    this.settings = {
      ...this.settings,
      ...partial,
    };
    try {
      await invoke('save_settings', { settings: this.settings });
    } catch (err) {
      console.debug('save_settings note:', err);
    }
    if ('show_agent_in_pill' in partial && !partial.show_agent_in_pill) {
      if (this.mode === 'agent') {
        await this.collapse();
      }
      this.agent = {
        name: 'Antigravity Agent',
        state: 'idle',
        message: 'Ready for your next task',
        is_demo: false,
      };
      await this.syncNativeWindow();
    }
    if ('top_margin' in partial || 'horizontal_position' in partial || 'x_offset' in partial) {
      await this.syncNativeWindow();
    }
  }

  async resetSettings() {
    this.settings = { ...DEFAULT_SETTINGS };
    try {
      await invoke('save_settings', { settings: this.settings });
    } catch (err) {
      console.debug('resetSettings note:', err);
    }
    await this.syncNativeWindow();
  }

  async setModuleOrder(order: string[]) {
    await this.updateSettings({ module_order: order });
  }

  async moveModule(index: number, direction: -1 | 1) {
    const current = [...this.moduleOrder];
    const targetIndex = index + direction;
    if (targetIndex < 0 || targetIndex >= current.length) return;
    const [moved] = current.splice(index, 1);
    current.splice(targetIndex, 0, moved);
    await this.setModuleOrder(current);
  }

  async resetModuleOrder() {
    await this.setModuleOrder([...DEFAULT_MODULE_ORDER]);
  }

  async refreshTelemetry() {
    try {
      const data = await invoke<SystemTelemetry>('get_system_telemetry');
      if (data) {
        this.telemetry = data;
      }
    } catch (err) {
      console.debug('refreshTelemetry note:', err);
    }
  }

  async openOverview() {
    await this.refreshMonitors();
    await this.setMode('expanded');
  }

  // --- Hover Expand & Lost Focus Minimization ---

  handleSurfacePointerEnter() {
    this.isHovered = true;
    if (this.leaveCollapseTimeout) {
      clearTimeout(this.leaveCollapseTimeout);
      this.leaveCollapseTimeout = null;
    }

    if (this.mode === 'idle') {
      if (this.settings.hover_expand_enabled) {
        if (this.hoverExpandTimeout) {
          clearTimeout(this.hoverExpandTimeout);
        }
        this.hoverExpandTimeout = setTimeout(() => {
          if (this.mode === 'idle' && this.isCursorOverSurface) {
            this.expand();
          }
        }, this.settings.hover_delay_ms);
      }
    } else {
      this.resetInactivityTimer();
    }
  }

  handleSurfacePointerMove() {
    this.isHovered = true;
    if (this.leaveCollapseTimeout) {
      clearTimeout(this.leaveCollapseTimeout);
      this.leaveCollapseTimeout = null;
    }
    const now = Date.now();
    if (now - this.lastActivityTime > 1000) {
      this.lastActivityTime = now;
      this.resetInactivityTimer();
    }
  }

  handleSurfacePointerLeave() {
    this.isHovered = false;
    if (this.hoverExpandTimeout) {
      clearTimeout(this.hoverExpandTimeout);
      this.hoverExpandTimeout = null;
    }

    if (this.mode === 'idle' || this.mode === 'notification') return;

    if (this.leaveCollapseTimeout) {
      clearTimeout(this.leaveCollapseTimeout);
    }

    this.leaveCollapseTimeout = setTimeout(() => {
      // Only collapse if cursor is truly not over the dynamic island area
      if (!this.isCursorOverSurface && this.mode !== 'idle' && this.mode !== 'notification') {
        this.collapse();
      }
    }, this.settings.leave_delay_ms);
  }

  handlePillHoverStart() {
    this.handleSurfacePointerEnter();
  }

  handlePillHoverEnd() {
    this.handleSurfacePointerLeave();
  }

  handlePanelPointerEnter() {
    this.handleSurfacePointerEnter();
  }

  handlePanelPointerLeave() {
    this.handleSurfacePointerLeave();
  }

  handleWindowBlur() {
    // When focus leaves the window, NEVER collapse if cursor is still over the island!
    if (this.isCursorOverSurface) {
      return;
    }
    if (this.mode !== 'idle' && this.mode !== 'notification') {
      this.collapse();
    }
  }

  handleUserActivity() {
    this.lastActivityTime = Date.now();
    this.resetInactivityTimer();
  }

  clearHoverTimers() {
    if (this.hoverExpandTimeout) {
      clearTimeout(this.hoverExpandTimeout);
      this.hoverExpandTimeout = null;
    }
    if (this.leaveCollapseTimeout) {
      clearTimeout(this.leaveCollapseTimeout);
      this.leaveCollapseTimeout = null;
    }
  }

  // --- Inactivity Timer ---

  resetInactivityTimer() {
    if (this.autoCollapseTimeout) {
      clearTimeout(this.autoCollapseTimeout);
      this.autoCollapseTimeout = null;
    }

    if (this.mode === 'idle' || this.mode === 'notification') {
      return;
    }

    this.autoCollapseTimeout = setTimeout(() => {
      if (this.mode === 'idle' || this.mode === 'notification') return;

      if (this.isCursorOverSurface) {
        // Cursor is still over the dynamic island area! Keep open and re-check after inactivity timeout
        this.resetInactivityTimer();
        return;
      }

      this.collapse();
    }, this.settings.inactivity_timeout_s * 1000);
  }

  clearInactivityTimer() {
    if (this.autoCollapseTimeout) {
      clearTimeout(this.autoCollapseTimeout);
      this.autoCollapseTimeout = null;
    }
  }

  // --- Notifications ---

  async showNotification(notif: NotificationItem) {
    if (this.notificationTimeout) {
      clearTimeout(this.notificationTimeout);
      this.notificationTimeout = null;
    }

    this.clearHoverTimers();
    this.clearInactivityTimer();

    if (this.mode !== 'notification') {
      this.previousMode = this.mode;
    }

    this.activeNotification = notif;
    await this.setMode('notification');

    const duration = notif.duration_ms || 4500;
    this.notificationTimeout = setTimeout(() => {
      this.dismissNotification();
    }, duration);
  }

  async dismissNotification() {
    if (this.notificationTimeout) {
      clearTimeout(this.notificationTimeout);
      this.notificationTimeout = null;
    }

    this.activeNotification = null;
    const targetMode = this.previousMode === 'notification' ? 'idle' : this.previousMode;
    await this.setMode(targetMode);
  }

  async triggerTestNotification(type: 'message' | 'build' | 'system' = 'message') {
    let payload = {
      title: 'Slack Message',
      message: 'Elena: Pull request #42 is ready for review.',
      appName: 'Slack',
      icon: 'message',
      durationMs: 4500,
    };

    if (type === 'build') {
      payload = {
        title: 'Cargo Build Success',
        message: 'dynamic-island target built in 0.50s [OK]',
        appName: 'Terminal',
        icon: 'terminal',
        durationMs: 4000,
      };
    } else if (type === 'system') {
      payload = {
        title: 'Battery Alert',
        message: 'Battery charged to 80% (Conservation Mode)',
        appName: 'Power Management',
        icon: 'system',
        durationMs: 4000,
      };
    }

    try {
      await invoke('trigger_notification', {
        title: payload.title,
        message: payload.message,
        appName: payload.appName,
        icon: payload.icon,
        durationMs: payload.durationMs,
      });
    } catch (err) {
      this.showNotification({
        id: `notif-${Date.now()}`,
        title: payload.title,
        message: payload.message,
        app_name: payload.appName,
        icon: payload.icon,
        duration_ms: payload.durationMs,
      });
    }
  }

  async togglePlayPause() {
    this.resetInactivityTimer();
    try {
      await invoke('media_play_pause');
      if (this.currentMedia) {
        this.currentMedia.status =
          this.currentMedia.status === 'Playing' ? 'Paused' : 'Playing';
      }
    } catch (err) {
      console.debug('media_play_pause failed:', err);
    }
  }

  async nextTrack() {
    this.resetInactivityTimer();
    try {
      await invoke('media_next');
    } catch (err) {
      console.debug('media_next failed:', err);
    }
  }

  async previousTrack() {
    this.resetInactivityTimer();
    try {
      await invoke('media_previous');
    } catch (err) {
      console.debug('media_previous failed:', err);
    }
  }

  // --- Multi-Monitor Methods ---

  async refreshMonitors() {
    try {
      const list = await invoke<MonitorInfo[]>('get_available_monitors');
      if (list) {
        this.monitors = list;
      }
    } catch (err) {
      console.debug('get_available_monitors note:', err);
    }
  }

  async moveToMonitor(targetName: string) {
    this.resetInactivityTimer();
    const dim = this.dimensions;
    try {
      await invoke('move_island_to_monitor_by_name', {
        targetName,
        width: dim.width,
        height: dim.height,
        topMargin: this.settings.top_margin,
      });
      await this.refreshMonitors();
    } catch (err) {
      console.debug('move_island_to_monitor_by_name failed:', err);
    }
  }

  async cycleMonitor() {
    this.resetInactivityTimer();
    const dim = this.dimensions;
    try {
      await invoke('cycle_next_monitor', {
        width: dim.width,
        height: dim.height,
        topMargin: this.settings.top_margin,
      });
      await this.refreshMonitors();
    } catch (err) {
      console.debug('cycle_next_monitor failed:', err);
    }
  }

  async cyclePosition() {
    this.resetInactivityTimer();
    const dim = this.dimensions;
    try {
      const nextPos = await invoke<string>('cycle_horizontal_position', {
        width: dim.width,
        height: dim.height,
      });
      if (nextPos) {
        this.settings.horizontal_position = nextPos as any;
        this.settings.x_offset = 0;
      }
    } catch (err) {
      console.debug('cyclePosition note:', err);
    }
  }

  async setHorizontalPosition(position: 'left' | 'center' | 'right' | 'custom', xOffset: number = 0) {
    this.resetInactivityTimer();
    const dim = this.dimensions;
    try {
      await invoke('set_horizontal_position', {
        position,
        xOffset,
        width: dim.width,
        height: dim.height,
      });
      this.settings.horizontal_position = position;
      this.settings.x_offset = xOffset;
    } catch (err) {
      console.debug('setHorizontalPosition note:', err);
    }
  }

  async refreshWeather(city?: string) {
    try {
      const data = await invoke<WeatherData>('fetch_weather', {
        city: city !== undefined ? city : (this.settings.weather_city ?? null),
      });
      if (data) {
        this.weather = data;
      }
    } catch (err) {
      console.debug('refreshWeather note:', err);
    }
  }

  async saveCustomPosition() {
    try {
      await invoke('save_custom_window_position');
    } catch (err) {
      console.debug('saveCustomPosition note:', err);
    }
  }

  async initServices() {
    // 0. Load persistent settings from disk
    try {
      const saved = await invoke<IslandSettings>('get_settings');
      if (saved) {
        this.settings = saved;
      }
    } catch (err) {
      console.debug('Initial get_settings note:', err);
    }

    // 1. Initial media status
    try {
      const initial = await invoke<MediaMetadata | null>('get_media_status');
      if (initial) {
        this.currentMedia = initial;
        if (this.mode === 'idle') {
          await this.syncNativeWindow();
        }
      }
    } catch (err) {
      console.debug('Initial get_media_status note:', err);
    }

    // 2. Discover connected monitors
    await this.refreshMonitors();

    // 3. Listen for MPRIS changes
    try {
      this.unlistenMedia = await listen<MediaMetadata | null>(
        'media-status-change',
        async (event) => {
          const wasActive = this.hasActiveMedia;
          this.currentMedia = event.payload;
          const nowActive = this.hasActiveMedia;

          if (this.mode === 'idle' && wasActive !== nowActive) {
            await this.syncNativeWindow();
          }
        }
      );
    } catch (err) {
      console.debug('Failed to register media listener:', err);
    }

    // 3b. Live PipeWire / PulseAudio spectrum listener
    try {
      this.unlistenAudioSpectrum = await listen<AudioSpectrumData>(
        'audio-spectrum',
        (event) => {
          if (event.payload) {
            this.audioSpectrum = event.payload;
          }
        }
      );
    } catch (err) {
      console.debug('Failed to register audio-spectrum listener:', err);
    }

    // 3c. Live Weather listener & initial load
    try {
      this.unlistenWeather = await listen<WeatherData>(
        'weather-update',
        (event) => {
          if (event.payload) {
            this.weather = event.payload;
          }
        }
      );
      this.refreshWeather();
    } catch (err) {
      console.debug('Failed to register weather listener:', err);
    }

    // 4. Listen for desktop notifications
    try {
      this.unlistenNotification = await listen<NotificationItem>(
        'island-notification',
        (event) => {
          if (event.payload) {
            this.showNotification(event.payload);
          }
        }
      );
    } catch (err) {
      console.debug('Failed to register notification listener:', err);
    }

    // 5. System Telemetry (initial sample + live event stream)
    try {
      const initialTelemetry = await invoke<SystemTelemetry>('get_system_telemetry');
      if (initialTelemetry) {
        this.telemetry = initialTelemetry;
      }
    } catch (err) {
      console.debug('Initial get_system_telemetry note:', err);
    }

    try {
      this.unlistenTelemetry = await listen<SystemTelemetry>(
        'system-telemetry',
        (event) => {
          if (event.payload) {
            this.telemetry = event.payload;
          }
        }
      );
    } catch (err) {
      console.debug('Failed to register system telemetry listener:', err);
    }

    // 6. Developer Integrations (Initial status + Agent & Build listeners)
    try {
      const dev = await invoke<DevStatusPayload>('get_dev_status');
      if (dev) {
        if (this.settings.show_agent_in_pill) {
          this.agent = dev.agent;
        }
        this.build = dev.build;
        this.checkBuildWatchdog();
        if (dev.editor) {
          this.editor = dev.editor;
        }
      }
    } catch (err) {
      console.debug('Initial get_dev_status note:', err);
    }

    try {
      this.unlistenEditor = await listen<EditorPresence>('editor-presence-change', async (event) => {
        const wasActive = this.hasActiveEditor;
        this.editor = event.payload;
        const nowActive = this.hasActiveEditor;

        if (this.mode === 'idle' && wasActive !== nowActive) {
          await this.syncNativeWindow();
        }
      });
    } catch (err) {
      console.debug('Failed to register editor listener:', err);
    }

    try {
      this.unlistenAgent = await listen<AgentStatus>('agent-status-change', async (event) => {
        if (!this.settings.show_agent_in_pill) {
          return;
        }
        const wasActive = this.hasActiveAgent;
        this.agent = event.payload;
        const nowActive = this.hasActiveAgent;

        if (this.mode === 'idle' && wasActive !== nowActive) {
          await this.syncNativeWindow();
        }
      });
    } catch (err) {
      console.debug('Failed to register agent listener:', err);
    }

    try {
      this.unlistenBuild = await listen<BuildStatus>('build-status-change', async (event) => {
        const wasActive = this.hasActiveBuild;
        this.build = event.payload;
        this.checkBuildWatchdog();
        const nowActive = this.hasActiveBuild;

        if (this.mode === 'idle' && wasActive !== nowActive) {
          await this.syncNativeWindow();
        }
      });
    } catch (err) {
      console.debug('Failed to register build listener:', err);
    }

    // 7. Git Status (Initial sample + live event stream)
    try {
      const initialGit = await invoke<GitStatus>('get_git_status', {
        path: this.settings.watched_repo_path || null,
      });
      if (initialGit) {
        this.git = initialGit;
      }
    } catch (err) {
      console.debug('Initial get_git_status note:', err);
    }

    try {
      this.unlistenGit = await listen<GitStatus>('git-status-change', async (event) => {
        const wasActive = this.hasActiveGit;
        this.git = event.payload;
        const nowActive = this.hasActiveGit;

        if (this.mode === 'idle' && wasActive !== nowActive) {
          await this.syncNativeWindow();
        }
      });
    } catch (err) {
      console.debug('Failed to register git listener:', err);
    }

    // 8. System Tray & Global Shortcuts
    try {
      this.unlistenTrayToggle = await listen('tray-toggle-island', async () => {
        await this.toggle();
      });
    } catch (err) {
      console.debug('Failed to register tray-toggle listener:', err);
    }

    try {
      this.unlistenTrayCycle = await listen('tray-cycle-monitor', async () => {
        await this.cycleMonitor();
      });
    } catch (err) {
      console.debug('Failed to register tray-cycle listener:', err);
    }

    try {
      this.unlistenTraySettings = await listen('tray-open-settings', async () => {
        await this.openSettings();
      });
    } catch (err) {
      console.debug('Failed to register tray-settings listener:', err);
    }

    // 9. Hardware audio listener
    try {
      this.unlistenAudio = await listen<AudioStatus>('audio-status-change', async (event) => {
        const wasMuted = this.isMicMuted;
        this.audio = event.payload;
        const nowMuted = this.isMicMuted;

        if (this.mode === 'idle' && wasMuted !== nowMuted) {
          await this.syncNativeWindow();
        }
      });
    } catch (err) {
      console.debug('Failed to register audio listener:', err);
    }

    // 10. Live Clipboard listener
    try {
      this.unlistenClipboard = await listen<ClipboardData>('clipboard-change', (event) => {
        this.clipboard = event.payload;
      });
    } catch (err) {
      console.debug('Failed to register clipboard listener:', err);
    }

    // 11. Initial samples
    this.refreshPorts();
    this.refreshDocker();
    this.refreshAudio();
    this.refreshClipboard();
    this.refreshShelf();

    // 12. Settings updated / Reposition events from IPC or Tray
    try {
      this.unlistenSettings = await listen<IslandSettings>('settings-updated', (event) => {
        this.settings = event.payload;
      });
    } catch (err) {
      console.debug('Failed to register settings-updated listener:', err);
    }

    try {
      this.unlistenTrayReposition = await listen('tray-reposition-island', async () => {
        await this.syncNativeWindow();
      });
    } catch (err) {
      console.debug('Failed to register tray-reposition listener:', err);
    }
  }

  destroyServices() {
    this.clearHoverTimers();
    this.clearInactivityTimer();
    if (this.unlistenSettings) {
      this.unlistenSettings();
      this.unlistenSettings = null;
    }
    if (this.unlistenTrayReposition) {
      this.unlistenTrayReposition();
      this.unlistenTrayReposition = null;
    }
    if (this.unlistenClipboard) {
      this.unlistenClipboard();
      this.unlistenClipboard = null;
    }
    if (this.unlistenAudio) {
      this.unlistenAudio();
      this.unlistenAudio = null;
    }
    if (this.unlistenMedia) {
      this.unlistenMedia();
      this.unlistenMedia = null;
    }
    if (this.unlistenAudioSpectrum) {
      this.unlistenAudioSpectrum();
      this.unlistenAudioSpectrum = null;
    }
    if (this.unlistenWeather) {
      this.unlistenWeather();
      this.unlistenWeather = null;
    }
    if (this.unlistenNotification) {
      this.unlistenNotification();
      this.unlistenNotification = null;
    }
    if (this.unlistenTelemetry) {
      this.unlistenTelemetry();
      this.unlistenTelemetry = null;
    }
    if (this.unlistenAgent) {
      this.unlistenAgent();
      this.unlistenAgent = null;
    }
    if (this.unlistenBuild) {
      this.unlistenBuild();
      this.unlistenBuild = null;
    }
    if (this.unlistenGit) {
      this.unlistenGit();
      this.unlistenGit = null;
    }
    if (this.unlistenEditor) {
      this.unlistenEditor();
      this.unlistenEditor = null;
    }
    if (this.unlistenTrayToggle) {
      this.unlistenTrayToggle();
      this.unlistenTrayToggle = null;
    }
    if (this.unlistenTrayCycle) {
      this.unlistenTrayCycle();
      this.unlistenTrayCycle = null;
    }
    if (this.unlistenTraySettings) {
      this.unlistenTraySettings();
      this.unlistenTraySettings = null;
    }
    if (this.notificationTimeout) {
      clearTimeout(this.notificationTimeout);
      this.notificationTimeout = null;
    }
  }

  async syncNativeWindow(overrideWidth?: number, overrideHeight?: number) {
    const dim = this.dimensions;
    const w = overrideWidth ?? dim.width;
    const h = overrideHeight ?? (dim.height + (this.mode === 'idle' ? 0 : 14));
    this.isSyncing = true;
    try {
      await invoke('resize_and_center_island', {
        width: w,
        height: h,
        topMargin: this.settings.top_margin,
      });
    } catch (err) {
      console.debug('Native resize_and_center_island note:', err);
    } finally {
      this.isSyncing = false;
    }
  }
}

export const islandState = new IslandStore();
