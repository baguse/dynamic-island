<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
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
  import TodoPreview from './TodoPreview.svelte';
  import FocusPreview from './FocusPreview.svelte';
  import PrayerPreview from './PrayerPreview.svelte';
  import AudioEqualizer from './AudioEqualizer.svelte';
  import AmbientSkyCanvas from './AmbientSkyCanvas.svelte';
  import PixelMascot from './PixelMascot.svelte';
  import { FUNNY_FOCUS_QUOTES } from '../state/island.svelte';

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
  const pendingTodosCount = $derived(islandState.pendingTodosCount);
  const clipboardCount = $derived(islandState.clipboardHistoryCount);
  const weather = $derived(islandState.weather);
  const activePanelMode = $derived(
    islandState.activePanel || (islandState.mode !== 'idle' ? islandState.mode : (islandState.isFocusActive ? 'focus' : islandState.previousMode))
  );

  function getWeatherIcon(condition?: string, timePeriod?: string): string {
    if (!condition) return '🌤️';
    if (condition === 'rain') return '🌧️';
    if (condition === 'thunderstorm') return '⛈️';
    if (condition === 'snow') return '❄️';
    if (condition === 'fog') return '🌫️';
    if (condition === 'clouds') return '☁️';
    if (timePeriod === 'night') return '🌙';
    if (timePeriod === 'sunset' || timePeriod === 'dawn') return '🌅';
    return '☀️';
  }

  const weatherIcon = $derived(getWeatherIcon(weather?.condition, weather?.time_period));

  let funnyQuoteIdx = $state(0);
  let funnyQuoteTimer: ReturnType<typeof setInterval> | null = null;
  const currentFunnyQuote = $derived(FUNNY_FOCUS_QUOTES[funnyQuoteIdx % FUNNY_FOCUS_QUOTES.length]);

  let unlistenDragDrop: UnlistenFn | null = null;
  let isSurfaceDragOver = $state(false);

  onMount(async () => {
    funnyQuoteTimer = setInterval(() => {
      if (islandState.isFocusActive) {
        funnyQuoteIdx = (funnyQuoteIdx + 1) % FUNNY_FOCUS_QUOTES.length;
      }
    }, 4500);
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
    if (funnyQuoteTimer) {
      clearInterval(funnyQuoteTimer);
      funnyQuoteTimer = null;
    }
    if (unlistenDragDrop) {
      unlistenDragDrop();
      unlistenDragDrop = null;
    }
  });

  function handleSurfaceDragOver(e: DragEvent) {
    if (islandState.mode === 'settings') {
      return;
    }
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
    if (islandState.mode === 'settings') {
      return;
    }
    if (e.dataTransfer?.types.includes('application/x-dynamic-island-module')) {
      return;
    }
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

  let isDraggingWindow = false;
  let justFinishedDrag = false;

  async function handleSurfacePointerDown(e: PointerEvent) {
    islandState.handleUserActivity();
    if (e.altKey || e.shiftKey || e.button === 1) {
      e.preventDefault();
      e.stopPropagation();
      isDraggingWindow = true;
      justFinishedDrag = true;
      try {
        await invoke('start_window_drag');
      } catch (err) {
        console.debug('start_window_drag note:', err);
      }
    }
  }

  async function handleWindowPointerUp() {
    if (isDraggingWindow) {
      isDraggingWindow = false;
      await islandState.saveCustomPosition();
      setTimeout(() => {
        justFinishedDrag = false;
      }, 200);
    }
  }

  function handleSurfaceClick(e: MouseEvent) {
    if (e.altKey || e.shiftKey || justFinishedDrag || isDraggingWindow) {
      justFinishedDrag = false;
      if (isDraggingWindow) {
        isDraggingWindow = false;
        islandState.saveCustomPosition();
      }
      return;
    }
    if (islandState.mode === 'idle' || islandState.isCollapsing) {
      if (islandState.isFocusActive) {
        islandState.openFocus();
      } else {
        islandState.expand();
      }
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

  function handleOpenTodo(e: MouseEvent) {
    e.stopPropagation();
    islandState.openTodo();
  }

  function handleOpenFocus(e?: MouseEvent) {
    if (e) e.stopPropagation();
    islandState.openFocus();
  }

  function handleOpenPrayer(e?: MouseEvent) {
    if (e) e.stopPropagation();
    islandState.openPrayer();
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

  function handleCyclePosition(e: MouseEvent) {
    e.stopPropagation();
    islandState.cyclePosition();
  }

  function handleSelectMonitor(e: MouseEvent, name: string) {
    e.stopPropagation();
    islandState.moveToMonitor(name);
  }
</script>

<svelte:window
  onkeydown={() => islandState.handleUserActivity()}
  onpointerdown={() => islandState.handleUserActivity()}
  onpointerup={handleWindowPointerUp}
  onwheel={() => islandState.handleUserActivity()}
  onblur={() => {
    if (isDraggingWindow) {
      handleWindowPointerUp();
    }
    islandState.handleWindowBlur();
  }}
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
  class:todo={islandState.mode === 'todo' && !islandState.isCollapsing}
  class:focus={islandState.mode === 'focus' && !islandState.isCollapsing}
  class:surface-drag-over={isSurfaceDragOver}
  class:focus-active={(islandState.mode === 'idle' || islandState.isCollapsing) && islandState.isFocusActive}
  class:dev-build-active={(islandState.mode === 'idle' || islandState.isCollapsing) && islandState.hasActiveBuild}
  class:agent-active={(islandState.mode === 'idle' || islandState.isCollapsing) && islandState.hasActiveAgent}
  class:media-active={(islandState.mode === 'idle' || islandState.isCollapsing) && islandState.hasActiveMedia}
  class:is-expanding={islandState.isExpanding}
  class:is-collapsing={islandState.isCollapsing}
  role={islandState.mode === 'idle' || islandState.isCollapsing ? 'button' : 'region'}
  tabindex={islandState.mode === 'idle' || islandState.isCollapsing ? 0 : -1}
  aria-label={islandState.mode === 'idle' || islandState.isCollapsing ? 'Expand Dynamic Island' : 'Dynamic Island'}
  onclick={handleSurfaceClick}
  onpointerdown={handleSurfacePointerDown}
  onpointerup={handleWindowPointerUp}
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
  <AmbientSkyCanvas isExpanded={islandState.mode !== 'idle' && !islandState.isCollapsing} />

  {#if islandState.mode === 'idle' || islandState.isCollapsing}
    <div
      class="compact-pill-wrapper"
      class:is-collapsing={islandState.isCollapsing}
      class:is-expanding={islandState.isExpanding}
    >
      {#if islandState.activePillModule === 'focus' && islandState.isFocusActive}
        <!-- Compact Pill with Active Focus Mode (Funny UI) -->
        <div class="pill-content focus-pill">
          <div class="pill-left">
            <PixelMascot size="sm" stateOverride="typing" />
            <span class="focus-flame-icon" class:pulsing={!islandState.isFocusPaused} aria-hidden="true">🔥</span>
            <div class="pill-media-text focus-pill-text">
              <span class="pill-track-title focus-time-digits">{islandState.focusRemainingFormatted}</span>
              <span class="pill-track-artist focus-quote-text">{currentFunnyQuote}</span>
            </div>
          </div>
          <div class="pill-right">
            <span class="pill-badge pill-badge-focus">
              {islandState.isFocusPaused ? '⏸ Paused' : '🔥 Focus'}
            </span>
          </div>
        </div>
      {:else if islandState.activePillModule === 'prayer' && islandState.nextPrayer}
        <!-- Compact Pill with Imminent Prayer Alert -->
        <div class="pill-content prayer-pill">
          <div class="pill-left">
            <PixelMascot size="sm" />
            <span class="prayer-pill-glow-icon" aria-hidden="true">🕌</span>
            <div class="pill-media-text">
              <span class="pill-track-title">{islandState.nextPrayer.label}</span>
              <span class="pill-track-artist">{islandState.nextPrayerCountdownFormatted} until adhan</span>
            </div>
          </div>
          <div class="pill-right">
            <span class="pill-badge pill-badge-prayer-imminent">
              {islandState.nextPrayer.time}
            </span>
          </div>
        </div>
      {:else if islandState.activePillModule === 'build'}
        <!-- Compact Pill with Active Terminal Build -->
        <div class="pill-content">
          <div class="pill-left">
            <PixelMascot size="sm" />
            <span class="spinner build-spinner" aria-hidden="true"></span>
            <div class="pill-media-text">
              <span class="pill-track-title">{build.command}</span>
              <span class="pill-track-artist">Running</span>
            </div>
          </div>
          <div class="pill-right">
            <span class="pill-badge pill-badge-build">
              ⚡ Build
            </span>
          </div>
        </div>
      {:else if islandState.activePillModule === 'agent'}
        <!-- Compact Pill with Active AI Agent -->
        <div class="pill-content">
          <div class="pill-left">
            <PixelMascot size="sm" />
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
      {:else if islandState.activePillModule === 'media' && media}
        <!-- Compact Pill with Media Controls -->
        <div class="pill-content media-pill">
          <div class="pill-left">
            <PixelMascot size="sm" />
            <AudioEqualizer size="sm" isPlaying={islandState.isPlaying} />
            <div class="pill-media-text">
              <span class="pill-track-title">{media.title}</span>
              <span class="pill-track-artist">{media.artist}</span>
            </div>
          </div>

          <div class="pill-right">
            <span class="pill-badge pill-badge-media">
              {islandState.isPlaying ? '▶ Playing' : '⏸ Paused'}
            </span>
          </div>
        </div>
      {:else if islandState.activePillModule === 'git'}
        <!-- Compact Pill with Git Status -->
        <div class="pill-content">
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
      {:else if islandState.activePillModule === 'editor' && editor}
        <!-- Compact Pill with Editor Presence -->
        <div class="pill-content">
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
            <PixelMascot size="sm" />
            <span class="status-dot-pulse" aria-hidden="true"></span>
            <span class="pill-title">Dynamic Island</span>
          </div>
          <div class="pill-right">
            {#if isMicMuted && settings.show_mic_in_pill}
              <span
                class="pill-badge pill-badge-mic-muted"
                title="Microphone is MUTED"
              >
                <span class="mic-dot-pulse" aria-hidden="true"></span>
                🎤 Muted
              </span>
            {/if}
            {#if shelfCount > 0 && settings.show_shelf_in_pill}
              <span
                class="pill-badge pill-badge-shelf"
                title="{shelfCount} items parked on shelf"
              >
                📁 {shelfCount}
              </span>
            {/if}
            {#if pendingTodosCount > 0 && settings.show_todo_in_pill}
              <span
                class="pill-badge pill-badge-todo"
                title="{pendingTodosCount} pending tasks to do"
              >
                📝 {pendingTodosCount}
              </span>
            {/if}
            {#if islandState.nextPrayer && settings.show_prayer_in_pill}
              <span
                class="pill-badge pill-badge-prayer-idle"
                class:imminent={islandState.isPrayerImminent}
                title="Next: {islandState.nextPrayer.label} at {islandState.nextPrayer.time} ({islandState.nextPrayerCountdownFormatted} remaining)"
              >
                🕌 {islandState.nextPrayer.name} {islandState.nextPrayer.time}
              </span>
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
              {#if weather && settings.ambient_sky_enabled}
                <span class="pill-badge pill-badge-weather" title="{weather.description || 'Weather'} • {weather.location || 'Local'}">
                  <span class="pill-weather-icon">{weatherIcon}</span>
                  <span>{Math.round(weather.temp_c)}°</span>
                </span>
              {:else}
                <span class="pill-badge">Idle</span>
              {/if}
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
      {:else}
        <!-- Unified Side-by-Side Dashboard Layout -->
        <div class="dashboard-container">
          <!-- Top Window Bar -->
          <header class="dashboard-topbar">
            <div class="topbar-left">
              <span class="topbar-logo" aria-hidden="true">🏝️</span>
              <span class="topbar-title">Dynamic Island</span>
              <PixelMascot size="sm" />
              <button
                type="button"
                class="topbar-chip-btn position-btn"
                onclick={handleCyclePosition}
                title="Click to cycle position: Center ➔ Right ➔ Left (Quick Dodge)"
              >
                Pos: {(settings.horizontal_position || 'center').toUpperCase()} ⮂
              </button>
              {#if islandState.monitors.length > 1}
                <button
                  type="button"
                  class="topbar-chip-btn monitor-btn"
                  onclick={handleCycleMonitor}
                  title="Click to cycle Dynamic Island to next monitor"
                >
                  Screen: {islandState.currentMonitorName} ⮂
                </button>
              {:else}
                <span class="topbar-chip-btn non-clickable">Screen: {islandState.currentMonitorName}</span>
              {/if}
            </div>

            <div class="topbar-right">
              {#if weather && settings.ambient_sky_enabled}
                <!-- Quick Weather Status & Refresh -->
                <button
                  type="button"
                  class="topbar-weather-badge"
                  onclick={(e) => { e.stopPropagation(); islandState.refreshWeather(); }}
                  title="{weather.description || 'Weather'}, {Math.round(weather.temp_c)}°C ({Math.round(weather.temp_f)}°F) in {weather.location || 'Local'} • Click to refresh"
                >
                  <span class="weather-icon">{weatherIcon}</span>
                  <span class="weather-temp">{Math.round(weather.temp_c)}°C</span>
                </button>
              {/if}

              <!-- Quick Hardware Mic Mute Toggle -->
              <button
                type="button"
                class="topbar-mic-badge"
                class:muted={isMicMuted}
                onclick={handleToggleMic}
                title={isMicMuted ? 'Microphone is MUTED (Click to Unmute)' : 'Microphone is LIVE (Click to Mute)'}
              >
                <span class="mic-dot" class:muted-pulse={isMicMuted} class:active={!isMicMuted}></span>
                <span class="mic-text">{isMicMuted ? 'Muted' : 'Mic Active'}</span>
              </button>

              <!-- Collapse / Minimize to Pill -->
              <button
                type="button"
                class="topbar-icon-btn collapse-btn"
                onclick={(e) => { e.stopPropagation(); islandState.collapse(); }}
                title="Collapse to compact pill"
                aria-label="Collapse Island"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="18 15 12 9 6 15"></polyline>
                </svg>
              </button>
            </div>
          </header>

          <!-- Main Area: Sidebar + Detail Pane -->
          <div class="dashboard-body">
            <!-- Left Sidebar Navigation -->
            <aside class="dashboard-sidebar">
              <nav class="sidebar-nav">
                {#each islandState.moduleOrder as modKey (modKey)}
                  {#if modKey === 'media' && settings.show_media_in_pill}
                    <button
                      type="button"
                      class="nav-tab-btn"
                      class:active={activePanelMode === 'media'}
                      onclick={handleOpenMedia}
                    >
                      <div class="tab-icon media-tab-icon">
                        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                          <path d="M9 18V5l12-2v13"></path>
                          <circle cx="6" cy="18" r="3"></circle>
                          <circle cx="18" cy="16" r="3"></circle>
                        </svg>
                      </div>
                      <div class="tab-content">
                        <span class="tab-title">Media Player</span>
                        <span class="tab-sub">
                          {#if islandState.hasActiveMedia && media}
                            {media.title}
                          {:else}
                            MPRIS Audio
                          {/if}
                        </span>
                      </div>
                      {#if islandState.isPlaying}
                        <AudioEqualizer size="sm" isPlaying={true} />
                      {/if}
                    </button>

                  {:else if modKey === 'agent' && settings.show_agent_in_pill}
                    <button
                      type="button"
                      class="nav-tab-btn"
                      class:active={activePanelMode === 'agent'}
                      onclick={handleOpenAgent}
                    >
                      <div class="tab-icon agent-tab-icon">
                        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                          <rect x="3" y="11" width="18" height="10" rx="2"></rect>
                          <circle cx="12" cy="5" r="2"></circle>
                          <path d="M12 7v4"></path>
                          <line x1="8" y1="16" x2="8" y2="16"></line>
                          <line x1="16" y1="16" x2="16" y2="16"></line>
                        </svg>
                      </div>
                      <div class="tab-content">
                        <span class="tab-title">AI Agent</span>
                        <span class="tab-sub">
                          {#if agent.state === 'review'}
                            <span class="tab-badge-review">Review Ready</span>
                          {:else if agent.state === 'working'}
                            Working...
                          {:else if agent.state === 'thinking'}
                            Thinking...
                          {:else}
                            Agent Companion
                          {/if}
                        </span>
                      </div>
                      <span class="sidebar-status-dot state-{agent.state}"></span>
                    </button>

                  {:else if modKey === 'git' && settings.show_git_in_pill}
                    <button
                      type="button"
                      class="nav-tab-btn"
                      class:active={activePanelMode === 'git'}
                      onclick={handleOpenGit}
                    >
                      <div class="tab-icon git-tab-icon">
                        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                          <line x1="6" y1="3" x2="6" y2="15"></line>
                          <circle cx="18" cy="6" r="3"></circle>
                          <circle cx="6" cy="18" r="3"></circle>
                          <path d="M18 9a9 9 0 0 1-9 9"></path>
                        </svg>
                      </div>
                      <div class="tab-content">
                        <span class="tab-title">Git Repository</span>
                        <span class="tab-sub">
                          {#if git.is_repo}
                            ⎇ {git.branch || 'git'} {islandState.gitDirtyCount > 0 ? `*${islandState.gitDirtyCount}` : '✓ clean'}
                          {:else}
                            No active repo
                          {/if}
                        </span>
                      </div>
                      {#if islandState.gitDirtyCount > 0}
                        <span class="sidebar-badge git-dirty-badge">*{islandState.gitDirtyCount}</span>
                      {/if}
                    </button>

                  {:else if modKey === 'editor' && settings.show_editor_in_pill}
                    <button
                      type="button"
                      class="nav-tab-btn"
                      class:active={activePanelMode === 'editor'}
                      onclick={handleOpenEditor}
                    >
                      <div class="tab-icon editor-tab-icon">
                        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                          <polyline points="16 18 22 12 16 6"></polyline>
                          <polyline points="8 6 2 12 8 18"></polyline>
                        </svg>
                      </div>
                      <div class="tab-content">
                        <span class="tab-title">Code Editor</span>
                        <span class="tab-sub">
                          {#if hasActiveEditor && editor}
                            {editor.file_name}
                          {:else}
                            VS Code / Cursor
                          {/if}
                        </span>
                      </div>
                    </button>

                  {:else if modKey === 'ports' && settings.show_ports_in_pill}
                    <button
                      type="button"
                      class="nav-tab-btn"
                      class:active={activePanelMode === 'ports'}
                      onclick={handleOpenPorts}
                    >
                      <div class="tab-icon ports-tab-icon">
                        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                          <circle cx="12" cy="12" r="10"></circle>
                          <line x1="2" y1="12" x2="22" y2="12"></line>
                          <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
                        </svg>
                      </div>
                      <div class="tab-content">
                        <span class="tab-title">Ports & Web</span>
                        <span class="tab-sub">{islandState.webPortsCount} web servers</span>
                      </div>
                      {#if islandState.webPortsCount > 0}
                        <span class="sidebar-badge">{islandState.webPortsCount}</span>
                      {/if}
                    </button>

                  {:else if modKey === 'docker' && settings.show_docker_in_pill}
                    <button
                      type="button"
                      class="nav-tab-btn"
                      class:active={activePanelMode === 'docker'}
                      onclick={handleOpenDocker}
                    >
                      <div class="tab-icon docker-tab-icon">🐳</div>
                      <div class="tab-content">
                        <span class="tab-title">Docker</span>
                        <span class="tab-sub">{islandState.runningDockerCount} containers</span>
                      </div>
                      {#if islandState.runningDockerCount > 0}
                        <span class="sidebar-badge">{islandState.runningDockerCount}</span>
                      {/if}
                    </button>

                  {:else if modKey === 'mic' && settings.show_mic_in_pill}
                    <button
                      type="button"
                      class="nav-tab-btn"
                      class:active={activePanelMode === 'hardware'}
                      onclick={handleOpenHardware}
                    >
                      <div class="tab-icon hardware-tab-icon">🎙️</div>
                      <div class="tab-content">
                        <span class="tab-title">Hardware Audio</span>
                        <span class="tab-sub">{isMicMuted ? 'Muted' : 'Active'} • {islandState.activeOutputDeviceName}</span>
                      </div>
                    </button>

                  {:else if modKey === 'clipboard' && settings.show_clipboard_in_pill}
                    <button
                      type="button"
                      class="nav-tab-btn"
                      class:active={activePanelMode === 'clipboard'}
                      onclick={handleOpenClipboard}
                    >
                      <div class="tab-icon clipboard-tab-icon">📋</div>
                      <div class="tab-content">
                        <span class="tab-title">Clipboard History</span>
                        <span class="tab-sub">{clipboardCount} items</span>
                      </div>
                    </button>

                  {:else if modKey === 'shelf' && settings.show_shelf_in_pill}
                    <button
                      type="button"
                      class="nav-tab-btn"
                      class:active={activePanelMode === 'shelf'}
                      onclick={handleOpenShelf}
                    >
                      <div class="tab-icon shelf-tab-icon">📁</div>
                      <div class="tab-content">
                        <span class="tab-title">Desktop Drop Shelf</span>
                        <span class="tab-sub">{shelfCount} parked items</span>
                      </div>
                      {#if shelfCount > 0}
                        <span class="sidebar-badge shelf-badge">{shelfCount}</span>
                      {/if}
                    </button>

                  {:else if modKey === 'todo' && settings.show_todo_in_pill}
                    <button
                      type="button"
                      class="nav-tab-btn"
                      class:active={activePanelMode === 'todo'}
                      onclick={handleOpenTodo}
                    >
                      <div class="tab-icon todo-tab-icon">📝</div>
                      <div class="tab-content">
                        <span class="tab-title">To-Do List</span>
                        <span class="tab-sub">{pendingTodosCount} pending tasks</span>
                      </div>
                      {#if pendingTodosCount > 0}
                        <span class="sidebar-badge todo-badge">{pendingTodosCount}</span>
                      {/if}
                    </button>

                  {:else if modKey === 'focus' && settings.show_focus_in_pill !== false}
                    <button
                      type="button"
                      class="nav-tab-btn"
                      class:active={activePanelMode === 'focus'}
                      onclick={handleOpenFocus}
                    >
                      <div class="tab-icon focus-tab-icon">🔥</div>
                      <div class="tab-content">
                        <span class="tab-title">Focus Mode</span>
                        <span class="tab-sub">
                          {#if islandState.isFocusActive}
                            {islandState.focusRemainingFormatted} • {islandState.focusSession.label}
                          {:else}
                            Lock in & Pomodoro
                          {/if}
                        </span>
                      </div>
                      {#if islandState.isFocusActive}
                        <span class="sidebar-badge focus-badge">{islandState.isFocusPaused ? 'PAUSED' : 'ACTIVE'}</span>
                      {/if}
                    </button>

                  {:else if modKey === 'prayer' && settings.show_prayer_in_pill}
                    <button
                      type="button"
                      class="nav-tab-btn"
                      class:active={activePanelMode === 'prayer'}
                      onclick={handleOpenPrayer}
                    >
                      <div class="tab-icon prayer-tab-icon">🕌</div>
                      <div class="tab-content">
                        <span class="tab-title">Prayer Times</span>
                        <span class="tab-sub">
                          {#if islandState.nextPrayer}
                            {islandState.nextPrayer.name} {islandState.nextPrayer.time}
                          {:else}
                            Daily Schedule
                          {/if}
                        </span>
                      </div>
                      {#if islandState.isPrayerImminent}
                        <span class="sidebar-badge prayer-imminent-badge">Soon</span>
                      {/if}
                    </button>
                  {/if}
                {/each}

                <!-- Always available System Monitor & Settings tabs -->
                <div class="sidebar-section-divider"></div>

                <button
                  type="button"
                  class="nav-tab-btn"
                  class:active={activePanelMode === 'system' || activePanelMode === 'expanded'}
                  onclick={handleOpenSystem}
                >
                  <div class="tab-icon system-tab-icon">
                    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <rect x="2" y="2" width="20" height="8" rx="2"></rect>
                      <rect x="2" y="14" width="20" height="8" rx="2"></rect>
                      <line x1="6" y1="6" x2="6.01" y2="6"></line>
                      <line x1="6" y1="18" x2="6.01" y2="18"></line>
                    </svg>
                  </div>
                  <div class="tab-content">
                    <span class="tab-title">System Monitor</span>
                    <span class="tab-sub">CPU {telemetry?.cpu_percent.toFixed(0) || 0}% • RAM {telemetry?.mem_percent.toFixed(0) || 0}%</span>
                  </div>
                </button>

                <button
                  type="button"
                  class="nav-tab-btn"
                  class:active={activePanelMode === 'settings'}
                  onclick={handleOpenSettings}
                >
                  <div class="tab-icon settings-tab-icon">
                    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <circle cx="12" cy="12" r="3"></circle>
                      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
                    </svg>
                  </div>
                  <div class="tab-content">
                    <span class="tab-title">Settings</span>
                    <span class="tab-sub">Theme: {settings.theme || 'dark'}</span>
                  </div>
                </button>
              </nav>

              <!-- Sidebar Footer: Telemetry Mini-Spark -->
              {#if telemetry}
                <div class="sidebar-footer">
                  <div class="footer-telemetry-row">
                    <div class="telemetry-chip">
                      <span class="chip-label">CPU</span>
                      <span class="chip-val">{telemetry.cpu_percent.toFixed(0)}%</span>
                    </div>
                    <div class="telemetry-chip">
                      <span class="chip-label">RAM</span>
                      <span class="chip-val">{telemetry.mem_percent.toFixed(0)}%</span>
                    </div>
                    {#if telemetry.battery}
                      <div class="telemetry-chip" class:low-battery={islandState.isBatteryLow}>
                        <span class="chip-label">{telemetry.battery.is_charging ? '⚡' : '🔋'}</span>
                        <span class="chip-val">{telemetry.battery.percent.toFixed(0)}%</span>
                      </div>
                    {/if}
                  </div>
                </div>
              {/if}
            </aside>

            <!-- Right Detail Pane -->
            <main class="dashboard-detail-pane">
              {#if activePanelMode === 'media'}
                <MediaPreview />
              {:else if activePanelMode === 'agent'}
                <AgentPreview />
              {:else if activePanelMode === 'system' || activePanelMode === 'expanded'}
                <SystemPreview />
              {:else if activePanelMode === 'settings'}
                <SettingsPreview />
              {:else if activePanelMode === 'git'}
                <GitPreview />
              {:else if activePanelMode === 'editor'}
                <EditorPreview />
              {:else if activePanelMode === 'ports'}
                <PortsPreview />
              {:else if activePanelMode === 'docker'}
                <DockerPreview />
              {:else if activePanelMode === 'hardware'}
                <HardwarePreview />
              {:else if activePanelMode === 'clipboard'}
                <ClipboardPreview />
              {:else if activePanelMode === 'shelf'}
                <DropShelfPreview />
              {:else if activePanelMode === 'todo'}
                <TodoPreview />
              {:else if activePanelMode === 'focus'}
                <FocusPreview />
              {:else if activePanelMode === 'prayer'}
                <PrayerPreview />
              {/if}
            </main>
          </div>
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
    padding: 12px 20px;
    box-shadow: var(--island-shadow, 0 8px 24px rgba(0, 0, 0, 0.7));
    justify-content: flex-start;
  }

  /* Content Transition Wrappers */
  .compact-pill-wrapper {
    position: relative;
    z-index: 1;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    box-sizing: border-box;
    opacity: 1;
    cursor: pointer;
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
    position: relative;
    z-index: 1;
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
    box-shadow: 0 0 10px rgba(239, 68, 68, 0.4);
    margin-right: 4px;
    display: inline-flex;
    align-items: center;
    gap: 4px;
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
    margin-right: 4px;
  }

  .pill-badge-todo {
    background: rgba(16, 185, 129, 0.2);
    border: 1px solid rgba(16, 185, 129, 0.4);
    color: #6ee7b7;
    font-weight: 600;
    margin-right: 4px;
  }

  .pill-badge-media {
    background: rgba(168, 85, 247, 0.2);
    border: 1px solid rgba(168, 85, 247, 0.4);
    color: #c084fc;
    font-weight: 600;
  }

  .pill-badge-focus {
    background: rgba(245, 158, 11, 0.2);
    border: 1px solid rgba(245, 158, 11, 0.45);
    color: #fbbf24;
    font-weight: 600;
  }

  .pill-badge-prayer-idle {
    background: rgba(20, 184, 166, 0.15);
    border: 1px solid rgba(45, 212, 191, 0.35);
    color: #5eead4;
    font-weight: 600;
    margin-right: 4px;
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .pill-badge-prayer-idle.imminent {
    background: rgba(245, 158, 11, 0.25);
    border-color: rgba(245, 158, 11, 0.6);
    color: #fcd34d;
    animation: flameBounce 1.5s infinite;
  }

  .pill-badge-prayer-imminent {
    background: rgba(20, 184, 166, 0.25);
    border: 1px solid rgba(45, 212, 191, 0.5);
    color: #5eead4;
    font-weight: 700;
  }

  .prayer-pill-glow-icon {
    font-size: 13px;
    display: inline-block;
    filter: drop-shadow(0 0 6px rgba(45, 212, 191, 0.6));
  }

  .prayer-imminent-badge {
    background: #f59e0b;
    color: #000;
    font-weight: 800;
    font-size: 9px;
    padding: 1px 5px;
    border-radius: 4px;
  }

  .focus-flame-icon {
    font-size: 13px;
    line-height: 1;
    display: inline-block;
  }

  .focus-flame-icon.pulsing {
    animation: flameBounce 1.5s infinite;
  }

  .focus-time-digits {
    font-size: 12.5px;
    font-weight: 700;
    color: #f59e0b;
    font-family: ui-monospace, SFMono-Regular, monospace;
    letter-spacing: -0.3px;
  }

  .focus-quote-text {
    font-size: 10px;
    color: #fbbf24;
    max-width: 130px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-style: italic;
  }

  .focus-badge {
    background: rgba(245, 158, 11, 0.2);
    color: #fbbf24;
  }

  .focus-tab-icon {
    font-size: 14px;
  }

  .island-surface.focus-active {
    border-color: rgba(245, 158, 11, 0.55);
    box-shadow: 0 0 16px rgba(245, 158, 11, 0.35), 0 8px 24px rgba(0, 0, 0, 0.7);
    animation: flamePulse 2.5s ease-in-out infinite;
  }

  @keyframes flamePulse {
    0%, 100% {
      border-color: rgba(245, 158, 11, 0.45);
      box-shadow: 0 0 14px rgba(245, 158, 11, 0.25), 0 8px 24px rgba(0, 0, 0, 0.7);
    }
    50% {
      border-color: rgba(239, 68, 68, 0.7);
      box-shadow: 0 0 22px rgba(245, 158, 11, 0.55), 0 0 8px rgba(239, 68, 68, 0.4), 0 8px 24px rgba(0, 0, 0, 0.7);
    }
  }

  .pill-badge-sys {
    font-variant-numeric: tabular-nums;
    font-size: 9.5px;
    padding: 2px 7px;
  }

  .pill-badge-weather {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: rgba(255, 255, 255, 0.09);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: var(--island-text-primary, #ffffff);
    font-weight: 500;
    font-variant-numeric: tabular-nums;
    font-size: 10px;
    padding: 2px 7px;
    border-radius: 9999px;
  }

  .pill-weather-icon {
    font-size: 11px;
    line-height: 1;
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

  .monitor-switch-tag,
  .position-switch-tag {
    background: rgba(59, 130, 246, 0.15);
    color: #93c5fd;
    border-color: rgba(59, 130, 246, 0.3);
    cursor: pointer;
    font-family: inherit;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .monitor-switch-tag:hover,
  .position-switch-tag:hover {
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

  /* Dashboard Container & Topbar */
  .dashboard-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--island-bg, #000000);
    border-radius: inherit;
  }

  .dashboard-topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 42px;
    padding: 0 14px;
    background: var(--island-surface, rgba(255, 255, 255, 0.04));
    border-bottom: 1px solid var(--island-border, rgba(255, 255, 255, 0.1));
    flex-shrink: 0;
    user-select: none;
    -webkit-user-select: none;
  }

  .topbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .topbar-logo {
    font-size: 15px;
    line-height: 1;
  }

  .topbar-title {
    font-size: 12.5px;
    font-weight: 700;
    letter-spacing: -0.01em;
    color: var(--island-text-primary, #ffffff);
    margin-right: 4px;
  }

  .topbar-chip-btn {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--island-border, rgba(255, 255, 255, 0.14));
    border-radius: 6px;
    padding: 2px 7px;
    font-size: 10px;
    font-weight: 600;
    color: var(--island-text-secondary, #9ca3af);
    cursor: pointer;
    transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
  }

  .topbar-chip-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: var(--island-text-primary, #ffffff);
    border-color: rgba(255, 255, 255, 0.25);
  }

  .topbar-chip-btn.non-clickable {
    cursor: default;
  }

  .topbar-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .topbar-weather-badge {
    display: flex;
    align-items: center;
    gap: 5px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--island-border, rgba(255, 255, 255, 0.14));
    border-radius: 6px;
    padding: 3px 8px;
    font-size: 11px;
    font-weight: 500;
    color: var(--island-text-primary, #ffffff);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .topbar-weather-badge:hover {
    background: rgba(255, 255, 255, 0.12);
    border-color: rgba(255, 255, 255, 0.25);
  }

  .weather-icon {
    font-size: 12px;
    line-height: 1;
  }

  .weather-temp {
    font-size: 10.5px;
    font-variant-numeric: tabular-nums;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
  }

  .topbar-mic-badge {
    display: flex;
    align-items: center;
    gap: 5px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--island-border, rgba(255, 255, 255, 0.14));
    border-radius: 6px;
    padding: 3px 8px;
    font-size: 11px;
    font-weight: 600;
    color: var(--island-text-secondary, #9ca3af);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .topbar-mic-badge:hover {
    background: rgba(255, 255, 255, 0.12);
    color: var(--island-text-primary, #ffffff);
  }

  .topbar-mic-badge.muted {
    background: rgba(239, 68, 68, 0.18);
    border-color: rgba(239, 68, 68, 0.45);
    color: #fca5a5;
  }

  .mic-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .mic-dot.active {
    background: #10b981;
    box-shadow: 0 0 5px rgba(16, 185, 129, 0.6);
  }

  .mic-dot.muted-pulse {
    background: #ef4444;
    box-shadow: 0 0 6px #ef4444;
    animation: micPulse 1.6s infinite ease-in-out;
  }

  @keyframes micPulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.5; transform: scale(0.85); }
  }

  .topbar-icon-btn {
    background: transparent;
    border: none;
    color: var(--island-text-secondary, #9ca3af);
    cursor: pointer;
    width: 28px;
    height: 28px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    transition: background 0.15s ease, color 0.15s ease;
  }

  .topbar-icon-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: var(--island-text-primary, #ffffff);
  }

  /* Dashboard Body Layout */
  .dashboard-body {
    display: flex;
    flex-direction: row;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  /* Left Sidebar */
  .dashboard-sidebar {
    width: 240px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--island-border, rgba(255, 255, 255, 0.1));
    background: var(--island-surface, rgba(255, 255, 255, 0.02));
    overflow: hidden;
  }

  .sidebar-nav {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 8px 6px;
    display: flex;
    flex-direction: column;
    gap: 3px;
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.12) transparent;
  }

  .sidebar-nav::-webkit-scrollbar {
    width: 3px;
  }

  .sidebar-nav::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.12);
    border-radius: 3px;
  }

  .nav-tab-btn {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    padding: 6px 9px;
    border-radius: 8px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--island-text-secondary, #9ca3af);
    cursor: pointer;
    text-align: left;
    transition: background 0.15s ease, border-color 0.15s ease, color 0.15s ease;
    position: relative;
    user-select: none;
    -webkit-user-select: none;
  }

  .nav-tab-btn:hover {
    background: var(--island-surface-hover, rgba(255, 255, 255, 0.06));
    color: var(--island-text-primary, #ffffff);
  }

  .nav-tab-btn.active {
    background: var(--island-surface-active, rgba(59, 130, 246, 0.15));
    border-color: var(--island-border-active, rgba(59, 130, 246, 0.35));
    color: var(--island-text-primary, #ffffff);
    font-weight: 600;
  }

  .nav-tab-btn.active::before {
    content: '';
    position: absolute;
    left: 0;
    top: 6px;
    bottom: 6px;
    width: 3px;
    border-radius: 2px;
    background: var(--island-accent, #3b82f6);
  }

  .tab-icon {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    font-size: 13px;
    color: var(--island-text-primary, #ffffff);
  }

  .tab-content {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
    overflow: hidden;
  }

  .tab-title {
    font-size: 11.5px;
    font-weight: 600;
    line-height: 1.25;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--island-text-primary, #ffffff);
  }

  .tab-sub {
    font-size: 9.5px;
    line-height: 1.2;
    color: var(--island-text-muted, #6b7280);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 1px;
  }

  .tab-badge-review {
    color: #f59e0b;
    font-weight: 700;
  }

  .sidebar-section-divider {
    height: 1px;
    background: var(--island-border, rgba(255, 255, 255, 0.1));
    margin: 5px 6px;
    flex-shrink: 0;
  }

  .sidebar-badge {
    background: rgba(255, 255, 255, 0.12);
    border-radius: 10px;
    font-size: 9px;
    font-weight: 700;
    padding: 1px 5px;
    color: var(--island-text-primary, #ffffff);
    flex-shrink: 0;
  }

  .git-dirty-badge {
    background: rgba(245, 158, 11, 0.2);
    color: #f59e0b;
  }

  .shelf-badge {
    background: rgba(59, 130, 246, 0.2);
    color: #60a5fa;
  }

  .todo-badge {
    background: rgba(16, 185, 129, 0.2);
    color: #34d399;
  }

  .mini-equalizer {
    display: flex;
    align-items: flex-end;
    gap: 2px;
    height: 11px;
    flex-shrink: 0;
  }

  .mini-bar {
    width: 2px;
    background: var(--island-accent, #3b82f6);
    border-radius: 1px;
    animation: eq-bounce 0.8s ease infinite alternate;
  }

  .mini-bar:nth-child(1) { height: 5px; animation-delay: 0.1s; }
  .mini-bar:nth-child(2) { height: 11px; animation-delay: 0.3s; }
  .mini-bar:nth-child(3) { height: 8px; animation-delay: 0.2s; }

  @keyframes eq-bounce {
    from { height: 3px; }
    to { height: 11px; }
  }

  .sidebar-status-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .sidebar-status-dot.state-idle { background: #6b7280; }
  .sidebar-status-dot.state-thinking { background: #3b82f6; box-shadow: 0 0 5px #3b82f6; }
  .sidebar-status-dot.state-working { background: #10b981; box-shadow: 0 0 5px #10b981; }
  .sidebar-status-dot.state-review { background: #f59e0b; box-shadow: 0 0 7px #f59e0b; animation: reviewPulse 1.2s infinite; }

  /* Sidebar Footer */
  .sidebar-footer {
    padding: 7px 10px;
    border-top: 1px solid var(--island-border, rgba(255, 255, 255, 0.1));
    background: var(--island-surface, rgba(0, 0, 0, 0.2));
    flex-shrink: 0;
  }

  .footer-telemetry-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
  }

  .telemetry-chip {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--island-border, rgba(255, 255, 255, 0.1));
    border-radius: 6px;
    padding: 3px 2px;
    font-size: 9.5px;
  }

  .telemetry-chip.low-battery {
    background: rgba(239, 68, 68, 0.15);
    border-color: rgba(239, 68, 68, 0.4);
  }

  .chip-label {
    color: var(--island-text-muted, #6b7280);
    font-size: 8.5px;
    font-weight: 600;
  }

  .chip-val {
    color: var(--island-text-primary, #ffffff);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  /* Right Detail Pane */
  .dashboard-detail-pane {
    flex: 1;
    min-width: 0;
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 14px 18px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.14) transparent;
  }

  .dashboard-detail-pane::-webkit-scrollbar {
    width: 4px;
  }

  .dashboard-detail-pane::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.15);
    border-radius: 4px;
  }
</style>
