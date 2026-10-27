<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { islandState } from '../state/island.svelte';
  import IslandHeader from './IslandHeader.svelte';
  import AgentPreview from './AgentPreview.svelte';
  import MediaPreview from './MediaPreview.svelte';
  import NotificationBanner from './NotificationBanner.svelte';
  import SystemPreview from './SystemPreview.svelte';
  import SettingsPreview from './SettingsPreview.svelte';
  import GitPreview from './GitPreview.svelte';
  import EditorPreview from './EditorPreview.svelte';
  import PortsPreview from './PortsPreview.svelte';
  import DockerPreview from './DockerPreview.svelte';
  import HardwarePreview from './HardwarePreview.svelte';
  import ClipboardPreview from './ClipboardPreview.svelte';
  import DropShelfPreview from './DropShelfPreview.svelte';

  const media = $derived(islandState.currentMedia);
  const telemetry = $derived(islandState.telemetry);
  const agent = $derived(islandState.agent);
  const build = $derived(islandState.build);
  const git = $derived(islandState.git);
  const editor = $derived(islandState.editor);
  const hasActiveEditor = $derived(islandState.hasActiveEditor);
  const settings = $derived(islandState.settings);
  const isMicMuted = $derived(islandState.isMicMuted);
  const shelfCount = $derived(islandState.shelfCount);
  const clipboardCount = $derived(islandState.clipboardHistoryCount);
  const activePanelMode = $derived(
    islandState.activePanel || (islandState.mode !== 'idle' ? islandState.mode : islandState.previousMode)
  );

  let unlistenDragDrop: UnlistenFn | null = null;
  let isSurfaceDragOver = $state(false);

  onMount(async () => {
    try {
      unlistenDragDrop = await getCurrentWebview().onDragDropEvent(async (event) => {
        if (event.payload.type === 'enter' || event.payload.type === 'over') {
          isSurfaceDragOver = true;
        } else if (event.payload.type === 'leave') {
          isSurfaceDragOver = false;
        } else if (event.payload.type === 'drop') {
          isSurfaceDragOver = false;
          if (event.payload.paths && event.payload.paths.length > 0) {
            await islandState.addShelfPaths(event.payload.paths);
            await islandState.openShelf();
          }
        }
      });
    } catch (err) {
      console.debug('Native drag-drop listener registration note:', err);
    }
  });

  onDestroy(() => {
    if (unlistenDragDrop) {
      unlistenDragDrop();
      unlistenDragDrop = null;
    }
  });

  function handleSurfaceDragOver(e: DragEvent) {
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = 'copy';
    }
    isSurfaceDragOver = true;
  }

  function handleSurfaceDragLeave(e: DragEvent) {
    e.preventDefault();
    isSurfaceDragOver = false;
  }

  async function handleSurfaceDrop(e: DragEvent) {
    e.preventDefault();
    isSurfaceDragOver = false;
    if (!e.dataTransfer) return;

    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      const paths: string[] = [];
      for (let i = 0; i < e.dataTransfer.files.length; i++) {
        const p = (e.dataTransfer.files[i] as any).path;
        if (p) paths.push(p);
      }
      if (paths.length > 0) {
        await islandState.addShelfPaths(paths);
        await islandState.openShelf();
        return;
      }
    }

    const text = e.dataTransfer.getData('text/plain') || e.dataTransfer.getData('text/uri-list');
    if (text) {
      await islandState.addShelfText(text);
      await islandState.openShelf();
    }
  }

  function handleSurfaceClick(e: MouseEvent) {
    if (islandState.mode === 'idle' || islandState.isCollapsing) {
      islandState.expand();
    }
  }

  function handleSurfaceMouseEnter() {
    islandState.handleSurfacePointerEnter();
  }

  function handleSurfaceMouseLeave() {
    islandState.handleSurfacePointerLeave();
  }

  function handleSurfacePointerMove() {
    islandState.handleSurfacePointerMove();
  }

  function handleOpenAgent(e: MouseEvent) {
    e.stopPropagation();
    islandState.openAgent();
  }

  function handleOpenMedia(e: MouseEvent) {
    e.stopPropagation();
    islandState.openMedia();
  }

  function handleOpenSystem(e: MouseEvent) {
    e.stopPropagation();
    islandState.openSystem();
  }

  function handleOpenSettings(e: MouseEvent) {
    e.stopPropagation();
    islandState.openSettings();
  }

  function handleOpenGit(e: MouseEvent) {
    e.stopPropagation();
    islandState.openGit();
  }

  function handleOpenEditor(e: MouseEvent) {
    e.stopPropagation();
    islandState.openEditor();
  }

  function handleOpenPorts(e: MouseEvent) {
    e.stopPropagation();
    islandState.openPorts();
  }

  function handleOpenDocker(e: MouseEvent) {
    e.stopPropagation();
    islandState.openDocker();
  }

  function handleOpenHardware(e: MouseEvent) {
    e.stopPropagation();
    islandState.openHardware();
  }

  function handleOpenClipboard(e: MouseEvent) {
    e.stopPropagation();
    islandState.openClipboard();
  }

  function handleOpenShelf(e: MouseEvent) {
    e.stopPropagation();
    islandState.openShelf();
  }

  function handleToggleMic(e: MouseEvent) {
    e.stopPropagation();
    islandState.toggleMicMute();
  }

  function handleTogglePlay(e: MouseEvent) {
    e.stopPropagation();
    islandState.togglePlayPause();
  }

  function handleCycleMonitor(e: MouseEvent) {
    e.stopPropagation();
    islandState.cycleMonitor();
  }

  function handleSelectMonitor(e: MouseEvent, name: string) {
    e.stopPropagation();
    islandState.moveToMonitor(name);
  }
</script>

<svelte:window
  onkeydown={() => islandState.handleUserActivity()}
  onpointerdown={() => islandState.handleUserActivity()}
  onwheel={() => islandState.handleUserActivity()}
  onblur={() => islandState.handleWindowBlur()}
/>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  class="island-surface"
  data-theme={islandState.settings.theme || 'dark'}
  class:compact={islandState.mode === 'idle' || islandState.isCollapsing}
  class:expanded={islandState.mode === 'expanded' && !islandState.isCollapsing}
  class:agent={islandState.mode === 'agent' && !islandState.isCollapsing}
  class:media={islandState.mode === 'media' && !islandState.isCollapsing}
  class:notification={islandState.mode === 'notification' && !islandState.isCollapsing}
  class:system={islandState.mode === 'system' && !islandState.isCollapsing}
  class:settings={islandState.mode === 'settings' && !islandState.isCollapsing}
  class:git={islandState.mode === 'git' && !islandState.isCollapsing}
  class:editor={islandState.mode === 'editor' && !islandState.isCollapsing}
  class:ports={islandState.mode === 'ports' && !islandState.isCollapsing}
  class:docker={islandState.mode === 'docker' && !islandState.isCollapsing}
  class:hardware={islandState.mode === 'hardware' && !islandState.isCollapsing}
  class:clipboard={islandState.mode === 'clipboard' && !islandState.isCollapsing}
  class:shelf={islandState.mode === 'shelf' && !islandState.isCollapsing}
  class:surface-drag-over={isSurfaceDragOver}
  class:dev-build-active={(islandState.mode === 'idle' || islandState.isCollapsing) && islandState.hasActiveBuild}
  class:agent-active={(islandState.mode === 'idle' || islandState.isCollapsing) && islandState.hasActiveAgent}
  class:media-active={(islandState.mode === 'idle' || islandState.isCollapsing) && islandState.hasActiveMedia}
  class:is-expanding={islandState.isExpanding}
  class:is-collapsing={islandState.isCollapsing}
  role={islandState.mode === 'idle' || islandState.isCollapsing ? 'button' : 'region'}
  tabindex={islandState.mode === 'idle' || islandState.isCollapsing ? 0 : -1}
  aria-label={islandState.mode === 'idle' || islandState.isCollapsing ? 'Expand Dynamic Island' : 'Dynamic Island'}
  onclick={handleSurfaceClick}
  onkeydown={(e) => {
    if ((islandState.mode === 'idle' || islandState.isCollapsing) && (e.key === 'Enter' || e.key === ' ')) {
      e.preventDefault();
      islandState.expand();
    }
  }}
  onmouseenter={handleSurfaceMouseEnter}
  onmouseleave={handleSurfaceMouseLeave}
  onpointermove={handleSurfacePointerMove}
  ondragover={handleSurfaceDragOver}
  ondragleave={handleSurfaceDragLeave}
  ondrop={handleSurfaceDrop}
  style="width: {islandState.dimensions.width}px; min-height: {islandState.dimensions.height}px; height: {islandState.dimensions.height}px;"
>
  {#if islandState.mode === 'idle' || islandState.isCollapsing}
    <div
      class="compact-pill-wrapper"
      class:is-collapsing={islandState.isCollapsing}
      class:is-expanding={islandState.isExpanding}
    >
      {#if islandState.hasActiveBuild && settings.show_build_in_pill}
        <!-- Compact Pill with Active Terminal Build -->
        <div class="pill-content">
          <div class="pill-left">
            <span class="spinner build-spinner" aria-hidden="true"></span>
            <div class="pill-media-text">
              <span class="pill-track-title">{build.command}</span>
              <span class="pill-track-artist">Running</span>
            </div>
          </div>
          <div class="pill-right">
            <button
              type="button"
              class="pill-badge pill-badge-build"
              onclick={(e) => { e.stopPropagation(); islandState.clearBuild(); }}
              title="Click to clear build indicator"
            >
              ⚡ Build
            </button>
          </div>
        </div>
      {:else if islandState.hasActiveAgent && settings.show_agent_in_pill}
        <!-- Compact Pill with Active AI Agent -->
        <div class="pill-content">
          <div class="pill-left">
            <span class="agent-pill-dot state-{agent.state}" aria-hidden="true"></span>
            <div class="pill-media-text">
              <span class="pill-track-title">{agent.task || agent.name}</span>
              <span class="pill-track-artist">{agent.state}</span>
            </div>
          </div>
          <div class="pill-right">
            {#if agent.progress !== null && agent.progress !== undefined}
              <span class="pill-badge pill-badge-agent">{agent.progress}%</span>
            {:else if agent.state === 'review'}
              <span class="pill-badge pill-badge-review">Review</span>
            {:else}
              <span class="pill-badge pill-badge-agent">{agent.state}</span>
            {/if}
          </div>
        </div>
      {:else if islandState.hasActiveMedia && media && settings.show_media_in_pill}
        <!-- Compact Pill with Media Controls -->
        <div class="pill-content media-pill">
          <div class="pill-left">
            <div class="equalizer" class:paused={!islandState.isPlaying} aria-hidden="true">
              <span class="bar bar-1"></span>
              <span class="bar bar-2"></span>
              <span class="bar bar-3"></span>
            </div>
            <div class="pill-media-text">
              <span class="pill-track-title">{media.title}</span>
              <span class="pill-track-artist">{media.artist}</span>
            </div>
          </div>

          <div class="pill-right">
            <button
              type="button"
              class="pill-quick-ctrl"
              onclick={handleTogglePlay}
              aria-label={islandState.isPlaying ? 'Pause' : 'Play'}
              title={islandState.isPlaying ? 'Pause' : 'Play'}
            >
              {#if islandState.isPlaying}
                <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                  <rect x="6" y="4" width="4" height="16" rx="1"></rect>
                  <rect x="14" y="4" width="4" height="16" rx="1"></rect>
                </svg>
              {:else}
                <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                  <polygon points="5 3 19 12 5 21 5 3"></polygon>
                </svg>
              {/if}
            </button>
          </div>
        </div>
      {:else if islandState.hasActiveGit && settings.show_git_in_pill}
        <!-- Compact Pill with Git Status -->
        <div class="pill-content">
          <button type="button" class="pill-media-main-btn" onclick={handleOpenGit}>
            <div class="pill-left">
              <svg class="git-pill-icon" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <line x1="6" y1="3" x2="6" y2="15"></line>
                <circle cx="18" cy="6" r="3"></circle>
                <circle cx="6" cy="18" r="3"></circle>
                <path d="M18 9a9 9 0 0 1-9 9"></path>
              </svg>
              <div class="pill-media-text">
                <span class="pill-track-title">{git.branch || 'git'}</span>
                <span class="pill-track-artist">
                  {#if islandState.gitDirtyCount > 0}
                    *{islandState.gitDirtyCount} dirty
                  {:else}
                    clean
                  {/if}
                </span>
              </div>
            </div>
          </button>
          <div class="pill-right">
            {#if git.ahead > 0 || git.behind > 0}
              <span class="pill-badge pill-badge-git" title="{git.ahead} to push, {git.behind} to pull">
                {#if git.ahead > 0}↑{git.ahead}{/if}{#if git.behind > 0}↓{git.behind}{/if}
              </span>
            {:else if islandState.gitDirtyCount > 0}
              <span class="pill-badge pill-badge-git-dirty">*{islandState.gitDirtyCount}</span>
            {:else}
              <span class="pill-badge pill-badge-git-clean">✓ clean</span>
            {/if}
          </div>
        </div>
      {:else if hasActiveEditor && settings.show_editor_in_pill && editor}
        <!-- Compact Pill with Editor Presence -->
        <div class="pill-content">
          <button type="button" class="pill-media-main-btn" onclick={handleOpenEditor}>
            <div class="pill-left">
              <svg class="editor-pill-icon" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="16 18 22 12 16 6"></polyline>
                <polyline points="8 6 2 12 8 18"></polyline>
              </svg>
              <div class="pill-media-text">
                <span class="pill-track-title">{editor.file_name}</span>
                <span class="pill-track-artist">{editor.workspace || editor.editor}</span>
              </div>
            </div>
          </button>
          <div class="pill-right">
            {#if editor.line}
              <span class="pill-badge pill-badge-editor">:{editor.line}</span>
            {:else}
              <span class="pill-badge pill-badge-editor">{editor.state}</span>
            {/if}
          </div>
        </div>
      {:else}
        <!-- Compact Idle Pill -->
        <div class="pill-content">
          <div class="pill-left">
            <span class="status-dot-pulse" aria-hidden="true"></span>
            <span class="pill-title">Dynamic Island</span>
          </div>
          <div class="pill-right">
            {#if isMicMuted && settings.show_mic_in_pill}
              <button
                type="button"
                class="pill-badge pill-badge-mic-muted"
                onclick={handleToggleMic}
                title="Microphone is MUTED (Click to Unmute)"
              >
                <span class="mic-dot-pulse" aria-hidden="true"></span>
                🎤 Muted
              </button>
            {/if}
            {#if shelfCount > 0 && settings.show_shelf_in_pill}
              <button
                type="button"
                class="pill-badge pill-badge-shelf"
                onclick={handleOpenShelf}
                title="{shelfCount} items parked on shelf"
              >
                📁 {shelfCount}
              </button>
            {/if}
            {#if islandState.isBatteryLow && telemetry?.battery && settings.show_battery_alert}
              <span class="pill-badge pill-badge-alert" title="Battery Low">
                🪫 {telemetry.battery.percent}%
              </span>
            {:else if telemetry && settings.show_system_in_pill}
              <span class="pill-badge pill-badge-sys" title="CPU: {telemetry.cpu_percent.toFixed(1)}% | RAM: {telemetry.mem_percent.toFixed(1)}%">
                {Math.round(telemetry.cpu_percent)}% · {Math.round(telemetry.mem_percent)}%
              </span>
            {:else if islandState.runningDockerCount > 0 && settings.show_docker_in_pill}
              <span class="pill-badge pill-badge-docker" title="{islandState.runningDockerCount} Docker containers running">
                🐳 {islandState.runningDockerCount}
              </span>
            {:else if islandState.webPortsCount > 0 && settings.show_ports_in_pill}
              <span class="pill-badge pill-badge-ports" title="{islandState.webPortsCount} web servers listening">
                🌐 {islandState.webPortsCount}
              </span>
            {:else if !isMicMuted && shelfCount === 0}
              <span class="pill-badge">Idle</span>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  {/if}

  {#if islandState.mode !== 'idle' || islandState.isCollapsing}
    <!-- Expanded Panel / Modal States -->
    <div
      class="expanded-panel-wrapper"
      class:is-expanding={islandState.isExpanding}
      class:is-collapsing={islandState.isCollapsing}
    >
      {#if activePanelMode === 'notification'}
        <!-- Notification Banner Mode -->
        <NotificationBanner />
      {:else if activePanelMode === 'expanded'}
        <!-- Overview Panel Mode -->
        <div class="panel-content">
          <IslandHeader title="Dynamic Island" />

          <div class="panel-body">
            <p class="panel-description">
              Floating desktop companion on Ubuntu Cinnamon (X11).
            </p>

            <div class="badge-row">
              <span class="status-tag">X11</span>
              <span class="status-tag">Always-On-Top</span>
              {#if islandState.monitors.length > 1}
                <button
                  type="button"
                  class="status-tag monitor-switch-tag"
                  onclick={handleCycleMonitor}
                  title="Click to cycle Dynamic Island to next monitor"
                >
                  Screen: {islandState.currentMonitorName} ⮂
                </button>
              {:else}
                <span class="status-tag">Screen: {islandState.currentMonitorName}</span>
              {/if}
            </div>

            <div class="action-grid">
              <!-- Media Quick Card -->
              <button
                type="button"
                class="action-card-btn"
                onclick={handleOpenMedia}
                aria-label="Open Media Player"
              >
                <div class="card-icon media-icon">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M9 18V5l12-2v13"></path>
                    <circle cx="6" cy="18" r="3"></circle>
                    <circle cx="18" cy="16" r="3"></circle>
                  </svg>
                </div>
                <div class="card-text">
                  <span class="card-title">
                    {islandState.hasActiveMedia && media ? `Now Playing: ${media.title}` : 'Media Player (MPRIS)'}
                  </span>
                  <span class="card-subtitle">
                    {islandState.hasActiveMedia && media ? `${media.artist} • ${media.player_name}` : 'Control Spotify, VLC, browser playback'}
                  </span>
                </div>
              </button>

              <!-- AI Agent Card -->
              <button
                type="button"
                class="action-card-btn"
                onclick={handleOpenAgent}
                aria-label="Open AI Agent Companion"
              >
                <div class="card-icon agent-icon">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M12 2a2 2 0 0 1 2 2v2a2 2 0 0 1-2 2 2 2 0 0 1-2-2V4a2 2 0 0 1 2-2z"></path>
                    <rect x="4" y="8" width="16" height="12" rx="4"></rect>
                    <circle cx="9" cy="14" r="1.5"></circle>
                    <circle cx="15" cy="14" r="1.5"></circle>
                  </svg>
                </div>
                <div class="card-text">
                  <span class="card-title">
                    {agent.state !== 'idle' ? `${agent.name}: ${agent.state.toUpperCase()}` : 'AI Agent Companion'}
                  </span>
                  <span class="card-subtitle">
                    {agent.task ? `${agent.task} • ` : ''}{agent.message}
                  </span>
                </div>
              </button>

              <!-- Terminal Build Monitor Row (shown when a build is active or recently completed) -->
              {#if build.state !== 'idle'}
                <div class="tool-row">
                  <span class="tool-label">Build Status:</span>
                  <div class="tool-actions">
                    {#if build.state === 'running'}
                      <button
                        type="button"
                        class="action-chip-btn active"
                        onclick={() => islandState.clearBuild()}
                        title="Running: {build.command} (Click to clear/dismiss)"
                      >
                        ⚡ {build.command}... ✕
                      </button>
                    {:else if build.state === 'success'}
                      <button
                        type="button"
                        class="action-chip-btn chip-success"
                        onclick={() => islandState.clearBuild()}
                        title="Last build passed (Click to dismiss)"
                      >
                        ✓ {build.command ? build.command.split(' ')[0] : 'Build'} passed ✕
                      </button>
                    {:else if build.state === 'failed'}
                      <button
                        type="button"
                        class="action-chip-btn chip-failed"
                        onclick={() => islandState.clearBuild()}
                        title="Last build failed (Click to dismiss)"
                      >
                        ✗ {build.command ? build.command.split(' ')[0] : 'Build'} failed ✕
                      </button>
                    {/if}
                  </div>
                </div>
              {/if}

              <!-- System Performance Card -->
              <button
                type="button"
                class="action-card-btn"
                onclick={handleOpenSystem}
                aria-label="Open System Performance Monitor"
              >
                <div class="card-icon system-icon">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <rect x="4" y="4" width="16" height="16" rx="2"></rect>
                    <rect x="9" y="9" width="6" height="6"></rect>
                    <path d="M9 1v3M15 1v3M9 20v3M15 20v3M20 9h3M20 14h3M1 9h3M1 14h3"></path>
                  </svg>
                </div>
                <div class="card-text">
                  <span class="card-title">System Performance</span>
                  <span class="card-subtitle">
                    {#if telemetry}
                      CPU {telemetry.cpu_percent.toFixed(0)}% • RAM {telemetry.mem_percent.toFixed(0)}% • {telemetry.battery ? `${telemetry.battery.percent}% Bat` : 'AC Power'}
                    {:else}
                      Inspect CPU, RAM, Battery, and Network
                    {/if}
                  </span>
                </div>
              </button>

              <!-- Git Repository Card -->
              <button
                type="button"
                class="action-card-btn"
                onclick={handleOpenGit}
                aria-label="Open Git Repository Details"
              >
                <div class="card-icon git-icon">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="6" y1="3" x2="6" y2="15"></line>
                    <circle cx="18" cy="6" r="3"></circle>
                    <circle cx="6" cy="18" r="3"></circle>
                    <path d="M18 9a9 9 0 0 1-9 9"></path>
                  </svg>
                </div>
                <div class="card-text">
                  <span class="card-title">
                    {git.is_repo ? `${git.repo_name || 'Git'}: ⎇ ${git.branch || 'detached'}` : 'Git Repository'}
                  </span>
                  <span class="card-subtitle">
                    {#if git.is_repo}
                      {#if islandState.gitDirtyCount > 0}
                        {islandState.gitDirtyCount} changed files{#if git.ahead > 0} • ↑{git.ahead} unpushed{/if}
                      {:else}
                        Working tree clean{#if git.ahead > 0} • ↑{git.ahead} unpushed{:else} • Synced{/if}
                      {/if}
                    {:else}
                      No active repository detected
                    {/if}
                  </span>
                </div>
              </button>

              <!-- Code Editor Presence Card -->
              <button
                type="button"
                class="action-card-btn"
                onclick={handleOpenEditor}
                aria-label="Open Code Editor Presence Details"
              >
                <div class="card-icon editor-icon">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="16 18 22 12 16 6"></polyline>
                    <polyline points="8 6 2 12 8 18"></polyline>
                  </svg>
                </div>
                <div class="card-text">
                  <span class="card-title">
                    {editor?.editor || 'Code Editor'}{#if editor?.file_name}: {editor.file_name}{/if}
                  </span>
                  <span class="card-subtitle">
                    {#if hasActiveEditor && editor}
                      {editor.workspace || 'Active'} • {editor.state}{#if editor.line} • Ln {editor.line}{/if}
                    {:else}
                      RPC Presence for Cursor & VS Code
                    {/if}
                  </span>
                </div>
              </button>

              <!-- Localhost & Ports Card -->
              {#if settings.show_ports_in_pill}
                <button
                  type="button"
                  class="action-card-btn"
                  onclick={handleOpenPorts}
                  aria-label="Open Localhost and Listening Ports Hub"
                >
                  <div class="card-icon ports-icon">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <circle cx="12" cy="12" r="10"></circle>
                      <line x1="2" y1="12" x2="22" y2="12"></line>
                      <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
                    </svg>
                  </div>
                  <div class="card-text">
                    <span class="card-title">
                      Localhost & Ports ({islandState.webPortsCount} Web Servers)
                    </span>
                    <span class="card-subtitle">
                      {islandState.webPortsCount} web servers • {islandState.ports.length} total listening
                    </span>
                  </div>
                </button>
              {/if}

              <!-- Docker Containers Card -->
              {#if settings.show_docker_in_pill}
                <button
                  type="button"
                  class="action-card-btn"
                  onclick={handleOpenDocker}
                  aria-label="Open Docker Containers Hub"
                >
                  <div class="card-icon docker-icon">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <rect x="2" y="2" width="20" height="20" rx="4"></rect>
                      <path d="M7 10h10M7 14h10"></path>
                    </svg>
                  </div>
                  <div class="card-text">
                    <span class="card-title">
                      Docker Hub ({islandState.runningDockerCount} Running)
                    </span>
                    <span class="card-subtitle">
                      {islandState.runningDockerCount} active of {islandState.dockerContainers.length} containers
                    </span>
                  </div>
                </button>
              {/if}

              <!-- Hardware & Audio Controls Card -->
              <button
                type="button"
                class="action-card-btn"
                onclick={handleOpenHardware}
                aria-label="Open Hardware and Audio Controls"
              >
                <div class="card-icon hardware-icon" class:muted-mic={isMicMuted}>
                  {#if isMicMuted}
                    🎤
                  {:else}
                    🎧
                  {/if}
                </div>
                <div class="card-text">
                  <span class="card-title">
                    Hardware & Audio ({isMicMuted ? 'Mic Muted' : 'Mic Active'})
                  </span>
                  <span class="card-subtitle">
                    {islandState.audio?.active_output_name || 'Audio Output'} • Vol {islandState.audio?.output_volume ?? 100}%
                  </span>
                </div>
              </button>

              <!-- Clipboard History & Snippets Card -->
              <button
                type="button"
                class="action-card-btn"
                onclick={handleOpenClipboard}
                aria-label="Open Clipboard History and Snippets"
              >
                <div class="card-icon clipboard-icon">
                  📋
                </div>
                <div class="card-text">
                  <span class="card-title">
                    Clipboard & Snippets ({clipboardCount} items)
                  </span>
                  <span class="card-subtitle">
                    {islandState.clipboard.snippets.length} pinned snippets • 1-click copy & paste
                  </span>
                </div>
              </button>

              <!-- Desktop Drop Shelf Card -->
              <button
                type="button"
                class="action-card-btn"
                onclick={handleOpenShelf}
                aria-label="Open Desktop Drop Shelf"
              >
                <div class="card-icon shelf-icon">
                  📁
                </div>
                <div class="card-text">
                  <span class="card-title">
                    Desktop Drop Shelf ({shelfCount} Parked)
                  </span>
                  <span class="card-subtitle">
                    Drag & drop files or screenshots to park while switching apps
                  </span>
                </div>
              </button>

              <!-- Settings & Preferences Card -->
              <button
                type="button"
                class="action-card-btn"
                onclick={handleOpenSettings}
                aria-label="Open Settings and Preferences"
              >
                <div class="card-icon settings-icon">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="3"></circle>
                    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
                  </svg>
                </div>
                <div class="card-text">
                  <span class="card-title">Settings & Preferences</span>
                  <span class="card-subtitle">
                    Hover: {settings.hover_expand_enabled ? `${settings.hover_delay_ms}ms` : 'Off'} • Margin: {settings.top_margin}px
                  </span>
                </div>
              </button>

              <!-- Multi-Monitor explicit selector (if > 1 screen detected) -->
              {#if islandState.monitors.length > 1}
                <div class="tool-row">
                  <span class="tool-label">Switch Screen:</span>
                  <div class="tool-actions">
                    {#each islandState.monitors as mon}
                      <button
                        type="button"
                        class="action-chip-btn"
                        class:active={mon.is_current}
                        onclick={(e) => handleSelectMonitor(e, mon.name)}
                        title="{mon.name} ({mon.width}x{mon.height})"
                      >
                        {mon.name}
                      </button>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          </div>
        </div>
      {:else if activePanelMode === 'media'}
        <div class="panel-content">
          <MediaPreview />
        </div>
      {:else if activePanelMode === 'agent'}
        <div class="panel-content">
          <AgentPreview />
        </div>
      {:else if activePanelMode === 'system'}
        <div class="panel-content">
          <SystemPreview />
        </div>
      {:else if activePanelMode === 'settings'}
        <div class="panel-content">
          <SettingsPreview />
        </div>
      {:else if activePanelMode === 'git'}
        <div class="panel-content">
          <GitPreview />
        </div>
      {:else if activePanelMode === 'editor'}
        <div class="panel-content">
          <EditorPreview />
        </div>
      {:else if activePanelMode === 'ports'}
        <div class="panel-content">
          <PortsPreview />
        </div>
      {:else if activePanelMode === 'docker'}
        <div class="panel-content">
          <DockerPreview />
        </div>
      {:else if activePanelMode === 'hardware'}
        <div class="panel-content">
          <HardwarePreview />
        </div>
      {:else if activePanelMode === 'clipboard'}
        <div class="panel-content">
          <ClipboardPreview />
        </div>
      {:else if activePanelMode === 'shelf'}
        <div class="panel-content">
          <DropShelfPreview />
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .island-surface {
    position: relative;
    background: var(--island-bg, #000000);
    border: 1px solid var(--island-border, rgba(255, 255, 255, 0.14));
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    cursor: default;
    color: var(--island-text-primary, #ffffff);
    text-align: left;
    transform: translateZ(0);
    backface-visibility: hidden;
    -webkit-backface-visibility: hidden;
    appearance: none;
    -webkit-appearance: none;
    box-shadow: var(--island-glow, none);
    will-change: width, height, border-radius;
    transition: width 0.34s cubic-bezier(0.32, 1.25, 0.32, 1),
                height 0.34s cubic-bezier(0.32, 1.25, 0.32, 1),
                min-height 0.34s cubic-bezier(0.32, 1.25, 0.32, 1),
                border-radius 0.34s cubic-bezier(0.32, 1.25, 0.32, 1),
                padding 0.24s cubic-bezier(0.2, 0.9, 0.3, 1),
                box-shadow 0.25s ease,
                background-color 0.25s ease,
                border-color 0.2s ease,
                color 0.2s ease;
  }

  .island-surface.compact {
    border-radius: 9999px;
    padding: 0 14px;
    justify-content: center;
    cursor: pointer;
    font-family: inherit;
    background: var(--island-bg, #000000);
    box-shadow: var(--island-glow, none);
  }

  .island-surface.compact:hover {
    border-color: var(--island-border-hover, rgba(255, 255, 255, 0.28));
  }

  .island-surface.expanded,
  .island-surface.agent,
  .island-surface.media,
  .island-surface.system,
  .island-surface.settings,
  .island-surface.git,
  .island-surface.editor,
  .island-surface.ports,
  .island-surface.docker,
  .island-surface.hardware,
  .island-surface.clipboard,
  .island-surface.shelf,
  .island-surface:not(.compact):not(.notification) {
    border-radius: 20px;
    padding: 14px 16px;
    box-shadow: var(--island-shadow, 0 8px 32px rgba(0, 0, 0, 0.75));
    justify-content: flex-start;
  }

  .island-surface.surface-drag-over {
    border-color: #3b82f6 !important;
    box-shadow: 0 0 24px rgba(59, 130, 246, 0.5) !important;
    transform: scale(1.02);
  }

  .island-surface.notification {
    border-radius: 18px;
    padding: 12px 14px;
    box-shadow: var(--island-shadow, 0 8px 24px rgba(0, 0, 0, 0.7));
    justify-content: flex-start;
  }

  /* Content Transition Wrappers */
  .compact-pill-wrapper {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    box-sizing: border-box;
    animation: pillFadeInNormal 0.20s cubic-bezier(0.32, 1.25, 0.32, 1) both;
  }

  .compact-pill-wrapper.is-collapsing {
    opacity: 0;
    animation: pillPopLively 0.22s cubic-bezier(0.32, 1.25, 0.32, 1) 0.12s forwards;
  }

  .compact-pill-wrapper.is-expanding {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 52px;
    padding: 0 14px;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.08s ease-out;
  }

  .expanded-panel-wrapper {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
  }

  .expanded-panel-wrapper.is-expanding {
    animation: panelExpandLively 0.28s cubic-bezier(0.32, 1.25, 0.32, 1) 0.05s both;
  }

  .expanded-panel-wrapper.is-collapsing {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    opacity: 0;
    transform: scale(0.95) translateY(-4px);
    transition: opacity 0.12s ease-out, transform 0.12s ease-out;
  }

  @keyframes pillFadeInNormal {
    0% {
      opacity: 0;
      transform: scale(0.95);
    }
    100% {
      opacity: 1;
      transform: scale(1);
    }
  }

  @keyframes pillPopLively {
    0% {
      opacity: 0;
      transform: scale(0.88);
    }
    60% {
      opacity: 1;
      transform: scale(1.05);
    }
    85% {
      transform: scale(0.98);
    }
    100% {
      opacity: 1;
      transform: scale(1);
    }
  }

  @keyframes panelExpandLively {
    0% {
      opacity: 0;
      transform: scale(0.94) translateY(-6px);
    }
    70% {
      opacity: 1;
      transform: scale(1.02) translateY(1px);
    }
    100% {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  /* Compact Pill Layout */
  .pill-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    height: 100%;
  }

  .pill-media-main-btn {
    background: none;
    border: none;
    color: inherit;
    font-family: inherit;
    padding: 0;
    margin: 0;
    cursor: pointer;
    display: flex;
    align-items: center;
    flex: 1;
    overflow: hidden;
    text-align: left;
  }

  .pill-left {
    display: flex;
    align-items: center;
    gap: 8px;
    overflow: hidden;
  }

  .status-dot-pulse {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--island-green, #10b981);
    box-shadow: 0 0 8px rgba(16, 185, 129, 0.7);
    animation: statusPulse 2s infinite ease-in-out;
    flex-shrink: 0;
  }

  @keyframes statusPulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.6; transform: scale(0.85); }
  }

  .pill-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
    letter-spacing: -0.01em;
  }

  .pill-right {
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .pill-badge {
    font-size: 10px;
    font-weight: 500;
    padding: 2px 8px;
    border-radius: 9999px;
    background: rgba(255, 255, 255, 0.08);
    color: var(--island-text-secondary, #9ca3af);
  }

  .pill-badge-alert {
    background: rgba(239, 68, 68, 0.25);
    color: #fca5a5;
    font-weight: 600;
  }

  .pill-badge-mic-muted {
    background: rgba(239, 68, 68, 0.25);
    border: 1px solid rgba(239, 68, 68, 0.5);
    color: #fca5a5;
    font-weight: 600;
    cursor: pointer;
    box-shadow: 0 0 10px rgba(239, 68, 68, 0.4);
    margin-right: 4px;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    transition: all 0.2s ease;
  }

  .pill-badge-mic-muted:hover {
    background: rgba(239, 68, 68, 0.4);
    transform: scale(1.04);
  }

  .mic-dot-pulse {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #ef4444;
    box-shadow: 0 0 6px #ef4444;
    animation: statusPulse 1.4s infinite ease-in-out;
  }

  .pill-badge-shelf {
    background: rgba(59, 130, 246, 0.2);
    border: 1px solid rgba(59, 130, 246, 0.4);
    color: #93c5fd;
    font-weight: 600;
    cursor: pointer;
    margin-right: 4px;
    transition: all 0.2s ease;
  }

  .pill-badge-shelf:hover {
    background: rgba(59, 130, 246, 0.35);
    transform: scale(1.04);
  }

  .pill-badge-sys {
    font-variant-numeric: tabular-nums;
    font-size: 9.5px;
    padding: 2px 7px;
  }

  /* Equalizer Animation in Pill */
  .equalizer {
    display: flex;
    align-items: flex-end;
    gap: 2px;
    height: 14px;
    width: 12px;
    flex-shrink: 0;
  }

  .equalizer .bar {
    width: 2px;
    background-color: var(--island-accent, #3b82f6);
    border-radius: 1px;
  }

  .equalizer:not(.paused) .bar-1 {
    animation: eqBounce 0.8s ease-in-out infinite alternate;
  }
  .equalizer:not(.paused) .bar-2 {
    animation: eqBounce 1.1s ease-in-out infinite 0.2s alternate;
  }
  .equalizer:not(.paused) .bar-3 {
    animation: eqBounce 0.9s ease-in-out infinite 0.4s alternate;
  }

  .equalizer.paused .bar {
    height: 4px !important;
    opacity: 0.6;
  }

  @keyframes eqBounce {
    0% { height: 3px; }
    50% { height: 14px; }
    100% { height: 6px; }
  }

  .pill-media-text {
    display: flex;
    align-items: baseline;
    gap: 6px;
    overflow: hidden;
    white-space: nowrap;
  }

  .pill-track-title {
    font-size: 12px;
    font-weight: 600;
    color: #ffffff;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 150px;
  }

  .pill-track-artist {
    font-size: 10px;
    color: var(--island-text-secondary, #9ca3af);
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 90px;
  }

  .pill-quick-ctrl {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: none;
    background: rgba(255, 255, 255, 0.12);
    color: #ffffff;
    cursor: pointer;
    padding: 0;
    transition: background 0.15s ease, transform 0.1s ease;
  }

  .pill-quick-ctrl:hover {
    background: rgba(255, 255, 255, 0.24);
  }

  .pill-quick-ctrl:active {
    transform: scale(0.9);
  }

  /* Expanded Panel Layout */
  .panel-content {
    display: flex;
    flex-direction: column;
    width: 100%;
    max-height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.15) transparent;
    animation: fadeIn 0.2s ease-out;
  }

  .panel-content::-webkit-scrollbar {
    width: 4px;
  }

  .panel-content::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.15);
    border-radius: 4px;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .panel-body {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .panel-description {
    margin: 0;
    font-size: 12px;
    color: var(--island-text-secondary, #9ca3af);
    line-height: 1.4;
  }

  .badge-row {
    display: flex;
    gap: 6px;
    align-items: center;
    flex-wrap: wrap;
  }

  .status-tag {
    font-size: 10px;
    font-weight: 500;
    padding: 2px 7px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--island-text-muted, #9ca3af);
    border: 1px solid rgba(255, 255, 255, 0.06);
  }

  .monitor-switch-tag {
    background: rgba(59, 130, 246, 0.15);
    color: #93c5fd;
    border-color: rgba(59, 130, 246, 0.3);
    cursor: pointer;
    font-family: inherit;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .monitor-switch-tag:hover {
    background: rgba(59, 130, 246, 0.25);
    border-color: rgba(59, 130, 246, 0.5);
    color: #ffffff;
  }

  .action-grid {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 2px;
  }

  .action-card-btn {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    border-radius: 12px;
    background: var(--island-card-bg, rgba(255, 255, 255, 0.05));
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    color: var(--island-text-primary, #ffffff);
    cursor: pointer;
    text-align: left;
    transition: background 0.15s ease, border-color 0.15s ease, transform 0.1s ease;
  }

  .action-card-btn:hover {
    background: var(--island-card-hover, rgba(255, 255, 255, 0.09));
    border-color: var(--island-border-hover, rgba(255, 255, 255, 0.16));
  }

  .action-card-btn:active {
    transform: scale(0.99);
  }

  .card-icon {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .card-icon.agent-icon {
    background: linear-gradient(135deg, #3b82f6 0%, #8b5cf6 100%);
    color: #ffffff;
  }

  .card-icon.media-icon {
    background: linear-gradient(135deg, #10b981 0%, #06b6d4 100%);
    color: #ffffff;
  }

  .card-icon.system-icon {
    background: linear-gradient(135deg, #f59e0b 0%, #ef4444 100%);
    color: #ffffff;
  }

  .card-icon.settings-icon {
    background: linear-gradient(135deg, #64748b 0%, #475569 100%);
    color: #ffffff;
  }

  .card-icon.git-icon {
    background: linear-gradient(135deg, #f97316 0%, #ea580c 100%);
    color: #ffffff;
  }

  .card-icon.editor-icon {
    background: linear-gradient(135deg, #06b6d4 0%, #3b82f6 100%);
    color: #ffffff;
  }

  .card-icon.ports-icon {
    background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
    color: #ffffff;
  }

  .card-icon.docker-icon {
    background: linear-gradient(135deg, #0ea5e9 0%, #0284c7 100%);
    color: #ffffff;
  }

  .card-icon.hardware-icon {
    background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
    color: #ffffff;
    font-size: 15px;
  }

  .card-icon.hardware-icon.muted-mic {
    background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
    box-shadow: 0 0 10px rgba(239, 68, 68, 0.4);
  }

  .card-icon.clipboard-icon {
    background: linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%);
    color: #ffffff;
    font-size: 15px;
  }

  .card-icon.shelf-icon {
    background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
    color: #ffffff;
    font-size: 15px;
  }

  .editor-pill-icon {
    color: #06b6d4;
    flex-shrink: 0;
  }

  .pill-badge-editor {
    font-size: 10px;
    font-weight: 600;
    padding: 2px 7px;
    border-radius: 9999px;
    background: rgba(6, 182, 212, 0.2);
    color: #67e8f9;
  }

  .git-pill-icon {
    color: #f97316;
    flex-shrink: 0;
  }

  .pill-badge-git {
    font-size: 10px;
    font-weight: 600;
    padding: 2px 7px;
    border-radius: 9999px;
    background: rgba(249, 115, 22, 0.2);
    color: #fdba74;
  }

  .pill-badge-git-dirty {
    font-size: 10px;
    font-weight: 600;
    padding: 2px 7px;
    border-radius: 9999px;
    background: rgba(245, 158, 11, 0.2);
    color: var(--island-amber, #f59e0b);
  }

  .pill-badge-git-clean {
    font-size: 10px;
    font-weight: 500;
    padding: 2px 7px;
    border-radius: 9999px;
    background: rgba(16, 185, 129, 0.15);
    color: var(--island-green, #10b981);
  }

  .card-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
    overflow: hidden;
  }

  .card-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .card-subtitle {
    font-size: 11px;
    color: var(--island-text-secondary, #9ca3af);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Utility tool row (Monitors & Notifications) */
  .tool-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    background: var(--island-card-bg, rgba(255, 255, 255, 0.03));
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.06));
    border-radius: 10px;
    gap: 8px;
  }

  .tool-label {
    font-size: 11px;
    color: var(--island-text-muted, #9ca3af);
    white-space: nowrap;
  }

  .tool-actions {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .action-chip-btn {
    padding: 3px 8px;
    font-size: 10px;
    font-weight: 500;
    border-radius: 6px;
    background: var(--island-card-bg, rgba(255, 255, 255, 0.08));
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    color: var(--island-text-secondary, #9ca3af);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
  }

  .action-chip-btn:hover {
    background: var(--island-card-hover, rgba(255, 255, 255, 0.16));
    border-color: var(--island-border-hover, rgba(255, 255, 255, 0.25));
    color: var(--island-text-primary, #ffffff);
  }

  .action-chip-btn.active {
    background: rgba(59, 130, 246, 0.25);
    color: #93c5fd;
    border: 1px solid rgba(59, 130, 246, 0.4);
  }

  .action-chip-btn.chip-success {
    background: rgba(16, 185, 129, 0.2);
    color: #34d399;
    border: 1px solid rgba(16, 185, 129, 0.4);
  }

  .action-chip-btn.chip-failed {
    background: rgba(239, 68, 68, 0.2);
    color: #fca5a5;
    border: 1px solid rgba(239, 68, 68, 0.4);
  }

  /* Active Developer Pills */
  .build-spinner {
    width: 11px;
    height: 11px;
    border: 2px solid rgba(255, 255, 255, 0.25);
    border-top-color: #3b82f6;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    flex-shrink: 0;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .agent-pill-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .agent-pill-dot.state-thinking {
    background: #c084fc;
    box-shadow: 0 0 8px rgba(192, 132, 252, 0.8);
    animation: statusPulse 1.2s infinite ease-in-out;
  }

  .agent-pill-dot.state-working {
    background: #60a5fa;
    box-shadow: 0 0 8px rgba(96, 165, 250, 0.8);
    animation: statusPulse 1.4s infinite ease-in-out;
  }

  .agent-pill-dot.state-review {
    background: #fbbf24;
    box-shadow: 0 0 8px rgba(251, 191, 36, 0.8);
    animation: statusPulse 1.6s infinite ease-in-out;
  }

  .pill-badge-build {
    background: rgba(59, 130, 246, 0.25);
    color: #93c5fd;
    font-weight: 600;
  }

  .pill-badge-agent {
    background: rgba(168, 85, 247, 0.25);
    color: #d8b4fe;
    font-weight: 600;
  }

  .pill-badge-review {
    background: rgba(245, 158, 11, 0.3);
    color: #fde68a;
    font-weight: 700;
  }
</style>
