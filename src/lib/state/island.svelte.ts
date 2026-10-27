import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  IslandMode,
  IslandDimensions,
  MediaMetadata,
  NotificationItem,
  MonitorInfo,
  SystemTelemetry,
  AgentStatus,
  BuildStatus,
  DevStatusPayload,
  IslandSettings,
  IslandTheme,
  GitStatus,
  EditorPresence,
  PortInfo,
  DockerContainerInfo,
} from '../types';

export const DEFAULT_SETTINGS: IslandSettings = {
  theme: 'dark',
  hover_expand_enabled: true,
  hover_delay_ms: 220,
  leave_delay_ms: 450,
  inactivity_timeout_s: 12,
  top_margin: 12.0,
  show_system_in_pill: true,
  show_battery_alert: true,
  show_media_in_pill: true,
  show_build_in_pill: true,
  show_agent_in_pill: true,
  show_git_in_pill: true,
  show_editor_in_pill: true,
  show_ports_in_pill: true,
  show_docker_in_pill: true,
  watched_repo_path: null,
  autostart_enabled: false,
};

export const BASE_DIMENSIONS: Record<IslandMode, IslandDimensions> = {
  idle: { width: 280, height: 52 },
  expanded: { width: 400, height: 430 },
  agent: { width: 400, height: 340 },
  media: { width: 400, height: 280 },
  notification: { width: 380, height: 96 },
  system: { width: 400, height: 350 },
  settings: { width: 400, height: 480 },
  git: { width: 400, height: 380 },
  editor: { width: 400, height: 320 },
  ports: { width: 400, height: 400 },
  docker: { width: 400, height: 420 },
};

class IslandStore {
  mode = $state<IslandMode>('idle');
  previousMode: IslandMode = 'idle';
  activePanel = $state<IslandMode | null>(null);
  isSyncing = $state<boolean>(false);
  isExpanding = $state<boolean>(false);
  isCollapsing = $state<boolean>(false);
  private collapseAnimationTimer: ReturnType<typeof setTimeout> | null = null;
  private expandAnimationTimer: ReturnType<typeof setTimeout> | null = null;
  currentMedia = $state<MediaMetadata | null>(null);
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

  get webPortsCount(): number {
    return this.ports.filter((p) => p.category === 'web').length;
  }

  get runningDockerCount(): number {
    return this.dockerContainers.filter((c) => c.is_running).length;
  }

  // Hover & Inactivity Settings
  hoverExpandDelayMs = 220;    // Hover intent delay to expand
  leaveCollapseDelayMs = 450;  // Grace delay after mouse leaves panel
  autoCollapseDelayMs = 12000; // Inactivity fallback (12s)

  private hoverExpandTimeout: ReturnType<typeof setTimeout> | null = null;
  private leaveCollapseTimeout: ReturnType<typeof setTimeout> | null = null;
  private autoCollapseTimeout: ReturnType<typeof setTimeout> | null = null;
  private isHovered = false;

  private unlistenMedia: UnlistenFn | null = null;
  private unlistenNotification: UnlistenFn | null = null;
  private unlistenTelemetry: UnlistenFn | null = null;
  private unlistenAgent: UnlistenFn | null = null;
  private unlistenBuild: UnlistenFn | null = null;
  private unlistenGit: UnlistenFn | null = null;
  private unlistenEditor: UnlistenFn | null = null;
  private unlistenTrayToggle: UnlistenFn | null = null;
  private unlistenTrayCycle: UnlistenFn | null = null;
  private unlistenTraySettings: UnlistenFn | null = null;
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

  get targetWidth(): number {
    if (this.mode === 'idle' || this.isCollapsing) {
      if (this.hasActiveBuild) return 330;
      if (this.hasActiveAgent) return 340;
      if (this.hasActiveMedia) return 330;
      if (this.hasActiveGit && this.settings.show_git_in_pill) return 330;
      if (this.hasActiveEditor && this.settings.show_editor_in_pill) return 340;
      return 280;
    }
    return BASE_DIMENSIONS[this.mode].width;
  }

  get targetHeight(): number {
    if (this.mode === 'idle' || this.isCollapsing) {
      return 52;
    }
    if (this.mode === 'expanded') {
      let h = 395;
      if (this.monitors.length > 1) h += 44;
      if (this.build.state !== 'idle') h += 40;
      return h;
    }
    if (this.mode === 'agent') {
      if (this.agent.state === 'review' || (this.agent.changed_files && this.agent.changed_files.length > 0)) {
        return 340;
      }
      return 260;
    }
    if (this.mode === 'git') {
      let h = 330;
      if (this.git.changed_files.length > 0) {
        h += Math.min(this.git.changed_files.length * 26, 110);
      }
      return h;
    }
    if (this.mode === 'editor') {
      return 320;
    }
    if (this.mode === 'ports') {
      return 400;
    }
    if (this.mode === 'docker') {
      return 420;
    }
    return BASE_DIMENSIONS[this.mode].height;
  }

  async setMode(newMode: IslandMode) {
    if (this.mode === newMode && !this.isCollapsing) return;

    if (newMode === 'idle') {
      await this.collapse();
      return;
    }

    if (this.mode === 'idle') {
      await this.expandToMode(newMode);
      return;
    }

    // Switching between open panel modes (e.g. overview -> agent)
    this.previousMode = this.mode;
    this.mode = newMode;
    this.activePanel = newMode;
    this.resetInactivityTimer();
    const neededHeight = this.targetHeight;
    await this.syncNativeWindow(400, neededHeight + 14);
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
    this.isExpanding = true;
    this.isCollapsing = false;
    this.resetInactivityTimer();

    // 1. Immediately expand the native window height so canvas is ready
    const neededHeight = this.targetHeight;
    await this.syncNativeWindow(400, neededHeight + 14);

    if (this.expandAnimationTimer) clearTimeout(this.expandAnimationTimer);
    this.expandAnimationTimer = setTimeout(() => {
      this.isExpanding = false;
      this.expandAnimationTimer = null;
    }, 340);
  }

  async expand() {
    const targetMode: IslandMode = this.hasActiveAgent
      ? 'agent'
      : this.hasActiveMedia
      ? 'media'
      : 'expanded';
    await this.expandToMode(targetMode);
  }

  async openEditor() {
    await this.expandToMode('editor');
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
    if ('top_margin' in partial) {
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

  handlePillHoverStart() {
    if (this.mode !== 'idle' || !this.settings.hover_expand_enabled) return;
    this.clearHoverTimers();

    this.hoverExpandTimeout = setTimeout(() => {
      if (this.mode === 'idle') {
        this.expand();
      }
    }, this.settings.hover_delay_ms);
  }

  handlePillHoverEnd() {
    if (this.hoverExpandTimeout) {
      clearTimeout(this.hoverExpandTimeout);
      this.hoverExpandTimeout = null;
    }
  }

  handlePanelPointerEnter() {
    this.isHovered = true;
    if (this.leaveCollapseTimeout) {
      clearTimeout(this.leaveCollapseTimeout);
      this.leaveCollapseTimeout = null;
    }
    this.resetInactivityTimer();
  }

  handlePanelPointerLeave() {
    this.isHovered = false;
    if (this.mode === 'idle' || this.mode === 'notification') return;

    if (this.leaveCollapseTimeout) {
      clearTimeout(this.leaveCollapseTimeout);
    }

    this.leaveCollapseTimeout = setTimeout(() => {
      if (!this.isHovered && this.mode !== 'idle' && this.mode !== 'notification') {
        this.collapse();
      }
    }, this.settings.leave_delay_ms);
  }

  handleWindowBlur() {
    // When focus leaves the window (user clicks on another app), minimize back
    if (this.mode !== 'idle' && this.mode !== 'notification') {
      this.collapse();
    }
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
      if (!this.isHovered && this.mode !== 'idle' && this.mode !== 'notification') {
        this.collapse();
      }
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
        this.agent = dev.agent;
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

    // 9. Initial Developer Hub sample
    this.refreshPorts();
    this.refreshDocker();
  }

  destroyServices() {
    this.clearHoverTimers();
    this.clearInactivityTimer();
    if (this.unlistenMedia) {
      this.unlistenMedia();
      this.unlistenMedia = null;
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
