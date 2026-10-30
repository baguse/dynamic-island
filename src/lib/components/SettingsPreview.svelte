<script lang="ts">
  import IslandHeader from './IslandHeader.svelte';
  import PixelMascot from './PixelMascot.svelte';
  import { islandState } from '../state/island.svelte';
  import type { IslandTheme, ModuleKey, IslandSettings, MascotState } from '../types';

  const settings = $derived(islandState.settings);

  const MODULE_DEFINITIONS: Record<ModuleKey, { label: string; icon: string; settingKey: keyof IslandSettings }> = {
    media: { label: 'Media Player (MPRIS)', icon: '🎵', settingKey: 'show_media_in_pill' },
    agent: { label: 'AI Agent Companion', icon: '🤖', settingKey: 'show_agent_in_pill' },
    build: { label: 'Terminal Build Monitor', icon: '⚡', settingKey: 'show_build_in_pill' },
    system: { label: 'CPU & RAM Telemetry', icon: '🖥️', settingKey: 'show_system_in_pill' },
    git: { label: 'Git Repository & Branch', icon: '⎇', settingKey: 'show_git_in_pill' },
    editor: { label: 'Code Editor Presence', icon: '⌨️', settingKey: 'show_editor_in_pill' },
    ports: { label: 'Localhost & Ports Hub', icon: '🌐', settingKey: 'show_ports_in_pill' },
    docker: { label: 'Docker Containers Hub', icon: '🐳', settingKey: 'show_docker_in_pill' },
    mic: { label: 'Hardware & Audio (Mic & Volume)', icon: '🎧', settingKey: 'show_mic_in_pill' },
    clipboard: { label: 'Quick Clipboard & Snippets', icon: '📋', settingKey: 'show_clipboard_in_pill' },
    shelf: { label: 'Desktop Drop Shelf', icon: '📁', settingKey: 'show_shelf_in_pill' },
  };

  interface ThemeDef {
    id: IslandTheme;
    name: string;
    swatchClass: string;
    title: string;
  }

  const THEME_OPTIONS: ThemeDef[] = [
    { id: 'dark', name: 'Dark', swatchClass: 'dark-swatch', title: 'Deep pitch black OLED with emerald & white' },
    { id: 'light', name: 'Light', swatchClass: 'light-swatch', title: 'Frosted clean acrylic with sapphire blue' },
    { id: 'catppuccin', name: 'Mocha', swatchClass: 'catppuccin-swatch', title: 'Catppuccin Mocha pastel dark palette with mauve & teal' },
    { id: 'nord', name: 'Nord', swatchClass: 'nord-swatch', title: 'Nord arctic cold blue with frost cyan & aurora' },
    { id: 'dracula', name: 'Dracula', swatchClass: 'dracula-swatch', title: 'Iconic vampire palette with vibrant purple & pink' },
    { id: 'tokyo-night', name: 'Tokyo', swatchClass: 'tokyo-swatch', title: 'Tokyo Night deep indigo with neon blue & lavender' },
    { id: 'gruvbox', name: 'Gruvbox', swatchClass: 'gruvbox-swatch', title: 'Warm retro groove palette with amber orange & cream' },
    { id: 'neon', name: 'Neon', swatchClass: 'neon-swatch', title: 'Cyberpunk synthwave glowing cyan & hot pink' },
    { id: 'matrix', name: 'Matrix', swatchClass: 'matrix-swatch', title: 'Terminal CRT phosphor green on pitch black' },
    { id: 'anime', name: 'Sakura', swatchClass: 'anime-swatch', title: 'Kawaii sakura blossom pink & soft violet' },
  ];

  let listContainer = $state<HTMLElement | null>(null);
  let draggedIndex = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);
  let isPointerDragging = $state(false);

  // Pointer-based drag (100% reliable across WebKitGTK / Linux / Tauri)
  function handlePointerDown(e: PointerEvent, index: number) {
    if (e.button !== 0) return;
    e.stopPropagation();
    draggedIndex = index;
    dragOverIndex = index;
    isPointerDragging = true;
    const target = e.currentTarget as HTMLElement;
    target.setPointerCapture(e.pointerId);
  }

  function handlePointerMove(e: PointerEvent) {
    if (!isPointerDragging || draggedIndex === null || !listContainer) return;
    e.preventDefault();
    e.stopPropagation();

    const items = Array.from(listContainer.querySelectorAll('.module-order-item'));
    const mouseY = e.clientY;
    for (let i = 0; i < items.length; i++) {
      const rect = items[i].getBoundingClientRect();
      if (mouseY >= rect.top && mouseY <= rect.bottom) {
        if (dragOverIndex !== i) {
          dragOverIndex = i;
        }
        break;
      }
    }
  }

  function handlePointerUp(e: PointerEvent) {
    if (!isPointerDragging) return;
    e.stopPropagation();
    try {
      const target = e.currentTarget as HTMLElement;
      target.releasePointerCapture(e.pointerId);
    } catch {
      // Ignored
    }
    isPointerDragging = false;
    if (draggedIndex !== null && dragOverIndex !== null && draggedIndex !== dragOverIndex) {
      const current = [...islandState.moduleOrder];
      const [moved] = current.splice(draggedIndex, 1);
      current.splice(dragOverIndex, 0, moved);
      islandState.setModuleOrder(current);
    }
    draggedIndex = null;
    dragOverIndex = null;
  }

  function handlePointerCancel() {
    isPointerDragging = false;
    draggedIndex = null;
    dragOverIndex = null;
  }

  // HTML5 drag fallback
  function handleDragStart(e: DragEvent, index: number) {
    e.stopPropagation();
    draggedIndex = index;
    dragOverIndex = index;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', String(index));
      e.dataTransfer.setData('application/x-dynamic-island-module', String(index));
    }
  }

  function handleDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    e.stopPropagation();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = 'move';
    }
    if (dragOverIndex !== index) {
      dragOverIndex = index;
    }
  }

  function handleDragLeave(e: DragEvent) {
    e.stopPropagation();
  }

  function handleDrop(e: DragEvent, targetIndex: number) {
    e.preventDefault();
    e.stopPropagation();
    const sourceIdx = draggedIndex !== null ? draggedIndex : parseInt(e.dataTransfer?.getData('text/plain') || '-1', 10);
    if (sourceIdx >= 0 && sourceIdx !== targetIndex) {
      const current = [...islandState.moduleOrder];
      const [moved] = current.splice(sourceIdx, 1);
      current.splice(targetIndex, 0, moved);
      islandState.setModuleOrder(current);
    }
    draggedIndex = null;
    dragOverIndex = null;
  }

  function handleDragEnd(e: DragEvent) {
    e.stopPropagation();
    draggedIndex = null;
    dragOverIndex = null;
  }

  function handleMoveUp(index: number) {
    islandState.moveModule(index, -1);
  }

  function handleMoveDown(index: number) {
    islandState.moveModule(index, 1);
  }

  function handleResetOrder() {
    islandState.resetModuleOrder();
  }

  function handleThemeChange(theme: IslandTheme) {
    islandState.updateSettings({ theme });
  }

  function handleToggleHover() {
    islandState.updateSettings({
      hover_expand_enabled: !settings.hover_expand_enabled,
    });
  }

  function handleHoverDelay(e: Event) {
    const val = parseInt((e.target as HTMLInputElement).value, 10);
    islandState.updateSettings({ hover_delay_ms: val });
  }

  function handleLeaveDelay(e: Event) {
    const val = parseInt((e.target as HTMLInputElement).value, 10);
    islandState.updateSettings({ leave_delay_ms: val });
  }

  function handleInactivity(e: Event) {
    const val = parseInt((e.target as HTMLInputElement).value, 10);
    islandState.updateSettings({ inactivity_timeout_s: val });
  }

  function handleTopMargin(e: Event) {
    const val = parseFloat((e.target as HTMLInputElement).value);
    islandState.updateSettings({ top_margin: val });
  }

  function handleHorizontalPosition(pos: 'left' | 'center' | 'right') {
    islandState.setHorizontalPosition(pos, 0);
  }

  function handleXOffset(e: Event) {
    const val = parseFloat((e.target as HTMLInputElement).value);
    islandState.updateSettings({ x_offset: val });
  }

  function handleResetXOffset() {
    islandState.updateSettings({ x_offset: 0 });
  }

  function handleToggleModule(module: ModuleKey | 'battery' | 'autostart') {
    if (module === 'system') {
      islandState.updateSettings({ show_system_in_pill: !settings.show_system_in_pill });
    } else if (module === 'battery') {
      islandState.updateSettings({ show_battery_alert: !settings.show_battery_alert });
    } else if (module === 'media') {
      islandState.updateSettings({ show_media_in_pill: !settings.show_media_in_pill });
    } else if (module === 'build') {
      islandState.updateSettings({ show_build_in_pill: !settings.show_build_in_pill });
    } else if (module === 'agent') {
      islandState.updateSettings({ show_agent_in_pill: !settings.show_agent_in_pill });
    } else if (module === 'git') {
      islandState.updateSettings({ show_git_in_pill: !settings.show_git_in_pill });
    } else if (module === 'editor') {
      islandState.updateSettings({ show_editor_in_pill: !settings.show_editor_in_pill });
    } else if (module === 'ports') {
      islandState.updateSettings({ show_ports_in_pill: !settings.show_ports_in_pill });
    } else if (module === 'docker') {
      islandState.updateSettings({ show_docker_in_pill: !settings.show_docker_in_pill });
    } else if (module === 'mic') {
      islandState.updateSettings({ show_mic_in_pill: !settings.show_mic_in_pill });
    } else if (module === 'shelf') {
      islandState.updateSettings({ show_shelf_in_pill: !settings.show_shelf_in_pill });
    } else if (module === 'clipboard') {
      islandState.updateSettings({ show_clipboard_in_pill: !settings.show_clipboard_in_pill });
    } else if (module === 'autostart') {
      islandState.updateSettings({ autostart_enabled: !settings.autostart_enabled });
    }
  }

  function handleReset() {
    islandState.resetSettings();
  }

  const weather = $derived(islandState.weather);

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

  let cityInput = $state('');

  $effect(() => {
    cityInput = settings.weather_city || '';
  });

  async function handleSaveCity() {
    const trimmed = cityInput.trim();
    const newCity = trimmed.length > 0 ? trimmed : null;
    await islandState.updateSettings({ weather_city: newCity });
    await islandState.refreshWeather(newCity || undefined);
  }

  let previewMascotMood = $state<MascotState | undefined>(undefined);

  // ---------------------------------------------------------------------------
  // Keyboard Shortcuts
  // ---------------------------------------------------------------------------

  const SHORTCUT_DEFINITIONS: { key: string; label: string; icon: string }[] = [
    { key: 'toggle',    label: 'Toggle Island',           icon: '⬛' },
    { key: 'agent',     label: 'AI Agent Panel',          icon: '🤖' },
    { key: 'media',     label: 'Media Player',            icon: '🎵' },
    { key: 'git',       label: 'Git Status',              icon: '⎇' },
    { key: 'system',    label: 'System Telemetry',        icon: '🖥️' },
    { key: 'clipboard', label: 'Clipboard & Snippets',    icon: '📋' },
    { key: 'editor',    label: 'Code Editor Presence',    icon: '⌨️' },
    { key: 'docker',    label: 'Docker Containers',       icon: '🐳' },
    { key: 'ports',     label: 'Ports Hub',               icon: '🌐' },
    { key: 'settings',  label: 'Settings',                icon: '⚙️' },
  ];

  // Which shortcut row is currently being recorded
  let recordingKey = $state<string | null>(null);
  // Temporary input while recording
  let recordingValue = $state('');

  function startRecording(key: string) {
    recordingKey = key;
    recordingValue = settings.shortcuts?.[key] ?? '';
  }

  function cancelRecording() {
    recordingKey = null;
    recordingValue = '';
  }

  function handleShortcutKeydown(e: KeyboardEvent, key: string) {
    e.preventDefault();
    e.stopPropagation();

    if (e.key === 'Escape') {
      cancelRecording();
      return;
    }

    // Build shortcut string from pressed combo
    const parts: string[] = [];
    if (e.metaKey || e.key === 'Meta')   parts.push('Super');
    if (e.ctrlKey  || e.key === 'Control') parts.push('Ctrl');
    if (e.altKey   || e.key === 'Alt')   parts.push('Alt');
    if (e.shiftKey || e.key === 'Shift') parts.push('Shift');

    const ignoredKeys = new Set(['Control', 'Shift', 'Alt', 'Meta', 'CapsLock']);
    if (!ignoredKeys.has(e.key)) {
      // Normalize key name for Tauri global-shortcut format
      let keyName = e.key;
      if (keyName === ',') keyName = 'Comma';
      else if (keyName === '.') keyName = 'Period';
      else if (keyName === '/') keyName = 'Slash';
      else if (keyName === ' ') keyName = 'Space';
      else if (keyName === 'ArrowUp')    keyName = 'Up';
      else if (keyName === 'ArrowDown')  keyName = 'Down';
      else if (keyName === 'ArrowLeft')  keyName = 'Left';
      else if (keyName === 'ArrowRight') keyName = 'Right';
      else if (keyName.length === 1)     keyName = keyName.toUpperCase();
      parts.push(keyName);
      const combo = parts.join('+');
      saveShortcut(key, combo);
    } else {
      // Show modifiers-only progress while holding
      recordingValue = parts.join('+') + '+…';
    }
  }

  function saveShortcut(key: string, combo: string) {
    const updated = { ...(settings.shortcuts ?? {}), [key]: combo };
    islandState.updateSettings({ shortcuts: updated });
    recordingKey = null;
    recordingValue = '';
  }

  function clearShortcut(key: string) {
    const updated = { ...(settings.shortcuts ?? {}), [key]: '' };
    islandState.updateSettings({ shortcuts: updated });
    if (recordingKey === key) cancelRecording();
  }

  function resetAllShortcuts() {
    islandState.updateSettings({
      shortcuts: {
        toggle:    'Super+Shift+I',
        agent:     'Super+Shift+A',
        media:     'Super+Shift+M',
        git:       'Super+Shift+G',
        system:    'Super+Shift+S',
        clipboard: 'Super+Shift+C',
        editor:    'Super+Shift+E',
        docker:    'Super+Shift+D',
        ports:     'Super+Shift+P',
        settings:  'Super+Shift+Comma',
      },
    });
  }
</script>

<div class="settings-wrapper">
  <IslandHeader title="Settings & Preferences" />

  <div class="settings-scroll-area">
    <!-- Theme & Appearance Section -->
    <div class="settings-group">
      <div class="group-header">
        <span class="group-title">Theme & Appearance</span>
        <span class="active-theme-tag">{settings.theme.toUpperCase()}</span>
      </div>

      <div class="theme-grid">
        {#each THEME_OPTIONS as t (t.id)}
          <button
            type="button"
            class="theme-card"
            class:active={settings.theme === t.id}
            onclick={() => handleThemeChange(t.id)}
            aria-label="Select {t.name} Theme"
            title={t.title}
          >
            <div class="theme-swatch {t.swatchClass}">
              <span class="swatch-dot"></span>
              <span class="swatch-bar"></span>
            </div>
            <span class="theme-name">{t.name}</span>
          </button>
        {/each}
      </div>
    </div>

    <!-- Section 1: Interaction & Behavior -->
    <div class="settings-group">
      <span class="group-title">Interaction & Behavior</span>

      <div class="setting-item">
        <div class="setting-info">
          <span class="setting-name">Hover to Expand</span>
          <span class="setting-desc">Expand island automatically on cursor hover</span>
        </div>
        <button
          type="button"
          class="toggle-switch"
          class:checked={settings.hover_expand_enabled}
          onclick={handleToggleHover}
          aria-label="Toggle hover to expand"
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>

      {#if settings.hover_expand_enabled}
        <div class="setting-item slider-item">
          <div class="slider-header">
            <span class="setting-name">Hover Intent Delay</span>
            <span class="slider-val">{settings.hover_delay_ms} ms</span>
          </div>
          <input
            type="range"
            min="100"
            max="600"
            step="20"
            value={settings.hover_delay_ms}
            oninput={handleHoverDelay}
            class="range-slider"
          />
        </div>
      {/if}

      <div class="setting-item slider-item">
        <div class="slider-header">
          <span class="setting-name">Mouse Leave Grace Period</span>
          <span class="slider-val">{settings.leave_delay_ms} ms</span>
        </div>
        <input
          type="range"
          min="200"
          max="1000"
          step="50"
          value={settings.leave_delay_ms}
          oninput={handleLeaveDelay}
          class="range-slider"
        />
      </div>

      <div class="setting-item slider-item">
        <div class="slider-header">
          <span class="setting-name">Inactivity Auto-Collapse</span>
          <span class="slider-val">{settings.inactivity_timeout_s} s</span>
        </div>
        <input
          type="range"
          min="5"
          max="30"
          step="1"
          value={settings.inactivity_timeout_s}
          oninput={handleInactivity}
          class="range-slider"
        />
      </div>
    </div>

    <!-- Section 2: Position & Placement -->
    <div class="settings-group">
      <div class="group-header">
        <span class="group-title">Position & Placement</span>
        <span class="active-theme-tag">
          {settings.horizontal_position ? settings.horizontal_position.toUpperCase() : 'CENTER'}
        </span>
      </div>

      <!-- Horizontal Screen Alignment Preset Buttons -->
      <div class="position-segmented">
        <button
          type="button"
          class="position-btn"
          class:active={settings.horizontal_position === 'left'}
          onclick={() => handleHorizontalPosition('left')}
          title="Align to Left edge of screen (with margin)"
        >
          <span class="pos-icon">⇤</span>
          <span class="pos-label">Left</span>
        </button>
        <button
          type="button"
          class="position-btn"
          class:active={settings.horizontal_position === 'center' || !settings.horizontal_position}
          onclick={() => handleHorizontalPosition('center')}
          title="Center on screen (Default)"
        >
          <span class="pos-icon">⇥ ⇤</span>
          <span class="pos-label">Center</span>
        </button>
        <button
          type="button"
          class="position-btn"
          class:active={settings.horizontal_position === 'right'}
          onclick={() => handleHorizontalPosition('right')}
          title="Align to Right edge of screen (with margin)"
        >
          <span class="pos-icon">⇥</span>
          <span class="pos-label">Right</span>
        </button>
      </div>

      <!-- Horizontal Shift Offset (X) Slider -->
      <div class="setting-item slider-item">
        <div class="slider-header">
          <div class="slider-title-row">
            <span class="setting-name">Horizontal Shift (X Offset)</span>
            {#if settings.x_offset !== 0}
              <button
                type="button"
                class="reset-offset-btn"
                onclick={handleResetXOffset}
                title="Reset offset to 0"
              >
                Reset
              </button>
            {/if}
          </div>
          <span class="slider-val">{Math.round(settings.x_offset || 0)} px</span>
        </div>
        <input
          type="range"
          min="-500"
          max="500"
          step="5"
          value={settings.x_offset || 0}
          oninput={handleXOffset}
          class="range-slider"
        />
      </div>

      <!-- Top Screen Margin (Y) Slider -->
      <div class="setting-item slider-item">
        <div class="slider-header">
          <span class="setting-name">Top Screen Margin</span>
          <span class="slider-val">{settings.top_margin} px</span>
        </div>
        <input
          type="range"
          min="0"
          max="80"
          step="2"
          value={settings.top_margin}
          oninput={handleTopMargin}
          class="range-slider"
        />
      </div>

      <!-- Position Tips & Shortcut -->
      <div class="position-tip-card">
        <div class="tip-line">
          <span class="tip-bullet">💡</span>
          <span><strong>Alt + Drag</strong> (or Shift + Drag) the island pill to move it freely anywhere on screen!</span>
        </div>
        <div class="tip-line">
          <span class="tip-bullet">⚡</span>
          <span>Run <code>island-cli position cycle</code> or click the position badge in Overview to dodge quickly!</span>
        </div>
      </div>
    </div>

    <!-- Section 3: Active Modules & Ordering -->
    <div class="settings-group">
      <div class="group-header">
        <div class="group-title-col">
          <span class="group-title">Modules & Ordering</span>
          <span class="group-desc">Reorder with ▲ / ▼ or drag. Order applies to Overview & pill.</span>
        </div>
        <button
          type="button"
          class="reset-order-btn"
          onclick={handleResetOrder}
          title="Restore default module order"
        >
          Reset Order
        </button>
      </div>

      <div class="module-order-list" role="list" bind:this={listContainer}>
        {#each islandState.moduleOrder as modKey, index (modKey)}
          {@const mod = MODULE_DEFINITIONS[modKey]}
          {#if mod}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="module-order-item"
              class:is-dragging={draggedIndex === index}
              class:drag-over={dragOverIndex === index && draggedIndex !== index}
              role="listitem"
              draggable="true"
              ondragstart={(e) => handleDragStart(e, index)}
              ondragover={(e) => handleDragOver(e, index)}
              ondragleave={handleDragLeave}
              ondrop={(e) => handleDrop(e, index)}
              ondragend={handleDragEnd}
            >
              <div class="module-item-left">
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                  class="drag-handle"
                  title="Drag to reorder"
                  aria-hidden="true"
                  onpointerdown={(e) => handlePointerDown(e, index)}
                  onpointermove={handlePointerMove}
                  onpointerup={handlePointerUp}
                  onpointercancel={handlePointerCancel}
                >
                  <svg width="10" height="12" viewBox="0 0 10 16" fill="currentColor">
                    <circle cx="2" cy="2" r="1.5"></circle>
                    <circle cx="8" cy="2" r="1.5"></circle>
                    <circle cx="2" cy="8" r="1.5"></circle>
                    <circle cx="8" cy="8" r="1.5"></circle>
                    <circle cx="2" cy="14" r="1.5"></circle>
                    <circle cx="8" cy="14" r="1.5"></circle>
                  </svg>
                </div>

                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div class="reorder-arrows" onpointerdown={(e) => e.stopPropagation()}>
                  <button
                    type="button"
                    class="arrow-btn"
                    draggable="false"
                    disabled={index === 0}
                    onclick={() => handleMoveUp(index)}
                    title="Move up"
                    aria-label="Move {mod.label} up"
                  >
                    ▲
                  </button>
                  <button
                    type="button"
                    class="arrow-btn"
                    draggable="false"
                    disabled={index === islandState.moduleOrder.length - 1}
                    onclick={() => handleMoveDown(index)}
                    title="Move down"
                    aria-label="Move {mod.label} down"
                  >
                    ▼
                  </button>
                </div>

                <span class="module-order-badge">{index + 1}</span>
                <span class="module-icon">{mod.icon}</span>
                <span class="setting-name">{mod.label}</span>
              </div>

              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div onpointerdown={(e) => e.stopPropagation()}>
                <button
                  type="button"
                  class="toggle-switch"
                  draggable="false"
                  class:checked={Boolean(settings[mod.settingKey])}
                  onclick={() => handleToggleModule(modKey)}
                  aria-label="Toggle {mod.label}"
                >
                  <span class="toggle-thumb"></span>
                </button>
              </div>
            </div>
          {/if}
        {/each}
      </div>

      <div class="system-toggles-divider"></div>

      <div class="setting-item">
        <span class="setting-name">Low Battery Warning (&lt; 20%)</span>
        <button
          type="button"
          class="toggle-switch"
          class:checked={settings.show_battery_alert}
          onclick={() => handleToggleModule('battery')}
          aria-label="Toggle low battery alert"
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>
    </div>

    <!-- Section: Media Visual Companion -->
    <div class="settings-group">
      <div class="group-header">
        <div class="group-title-col">
          <span class="group-title">Media Visual Companion</span>
          <span class="group-desc">Video/GIF player or spinning vinyl/cassette in Media tab</span>
        </div>
        <span class="active-theme-tag">
          {settings.media_companion_style ? settings.media_companion_style.toUpperCase() : 'VINYL'}
        </span>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <span class="setting-name">Enable Media Companion Player</span>
          <span class="setting-desc">Fills the bottom space with custom media or vinyl/cassette turntable</span>
        </div>
        <button
          type="button"
          class="toggle-switch"
          class:checked={settings.show_media_companion}
          onclick={() => islandState.updateSettings({ show_media_companion: !settings.show_media_companion })}
          aria-label="Toggle media companion"
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>

      {#if settings.show_media_companion}
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Default Visualizer Style</span>
            <span class="setting-desc">Shown when no custom video or image is loaded</span>
          </div>
          <div class="position-segmented" style="max-width: 200px;">
            <button
              type="button"
              class="position-btn"
              class:active={settings.media_companion_style === 'vinyl'}
              onclick={() => islandState.updateSettings({ media_companion_style: 'vinyl' })}
            >
              <span class="pos-icon">💿</span>
              <span class="pos-label">Vinyl</span>
            </button>
            <button
              type="button"
              class="position-btn"
              class:active={settings.media_companion_style === 'cassette'}
              onclick={() => islandState.updateSettings({ media_companion_style: 'cassette' })}
            >
              <span class="pos-icon">📼</span>
              <span class="pos-label">Cassette</span>
            </button>
          </div>
        </div>
      {/if}

      <!-- Equalizer Waves Style -->
      <div class="setting-item">
        <div class="setting-info">
          <span class="setting-name">Pill Equalizer Waveform</span>
          <span class="setting-desc">Animated audio-reactive waves next to track name in pill</span>
        </div>
        <div class="position-segmented" style="max-width: 240px;">
          <button
            type="button"
            class="position-btn"
            class:active={settings.equalizer_style === 'bars'}
            onclick={() => islandState.updateSettings({ equalizer_style: 'bars' })}
            title="Neon Spectrum Equalizer Bars"
          >
            <span class="pos-icon">📊</span>
            <span class="pos-label">Bars</span>
          </button>
          <button
            type="button"
            class="position-btn"
            class:active={settings.equalizer_style === 'wave'}
            onclick={() => islandState.updateSettings({ equalizer_style: 'wave' })}
            title="Fluid Sine Waveform (Apple Style)"
          >
            <span class="pos-icon">〰️</span>
            <span class="pos-label">Wave</span>
          </button>
          <button
            type="button"
            class="position-btn"
            class:active={settings.equalizer_style === 'dots'}
            onclick={() => islandState.updateSettings({ equalizer_style: 'dots' })}
            title="Minimalist Pulsing Dots"
          >
            <span class="pos-icon">•••</span>
            <span class="pos-label">Dots</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Section 4: Live Weather & Ambient Sky Canvas -->
    <div class="settings-group">
      <div class="group-header">
        <div class="group-title-col">
          <span class="group-title">Live Weather & Ambient Sky Canvas</span>
          <span class="group-desc">Subtle animated sky background (stars, rain, petals, clouds)</span>
        </div>
        {#if weather}
          <span class="weather-pill-indicator" title="{weather.description || 'Weather'} in {weather.location || 'Local'}">
            {weatherIcon} {Math.round(weather.temp_c)}°C
          </span>
        {/if}
      </div>

      <!-- Toggle Ambient Sky -->
      <div class="setting-item">
        <div class="setting-info">
          <span class="setting-name">Ambient Sky Atmosphere</span>
          <span class="setting-desc">Drifting stars, neon rain, falling sakura petals, and sunset clouds</span>
        </div>
        <button
          type="button"
          class="toggle-switch"
          class:checked={settings.ambient_sky_enabled}
          onclick={() => islandState.updateSettings({ ambient_sky_enabled: !settings.ambient_sky_enabled })}
          aria-label="Toggle ambient sky canvas"
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>

      {#if settings.ambient_sky_enabled}
        <!-- Sky Theme Selector -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Atmospheric Theme</span>
            <span class="setting-desc">Preset sky or dynamic sync with live weather & sun cycle</span>
          </div>
          <div class="sky-theme-segmented">
            <button
              type="button"
              class="sky-theme-btn"
              class:active={settings.ambient_sky_theme === 'auto'}
              onclick={() => islandState.updateSettings({ ambient_sky_theme: 'auto' })}
              title="Live Weather & Sun Cycle (Dynamic Day/Night/Rain/Clouds/Snow/Sakura)"
            >
              <span class="sky-btn-icon">☀️🌧️</span>
              <span class="sky-btn-label">Auto</span>
            </button>
            <button
              type="button"
              class="sky-theme-btn"
              class:active={settings.ambient_sky_theme === 'stars'}
              onclick={() => islandState.updateSettings({ ambient_sky_theme: 'stars' })}
              title="Cosmic Night & Twinkling Meteorites"
            >
              <span class="sky-btn-icon">✨</span>
              <span class="sky-btn-label">Stars</span>
            </button>
            <button
              type="button"
              class="sky-theme-btn"
              class:active={settings.ambient_sky_theme === 'rain'}
              onclick={() => islandState.updateSettings({ ambient_sky_theme: 'rain' })}
              title="Cyberpunk Neon Rain & Ripple Splashes"
            >
              <span class="sky-btn-icon">🌧️</span>
              <span class="sky-btn-label">Rain</span>
            </button>
            <button
              type="button"
              class="sky-theme-btn"
              class:active={settings.ambient_sky_theme === 'sakura'}
              onclick={() => islandState.updateSettings({ ambient_sky_theme: 'sakura' })}
              title="Falling Cherry Blossom Petals"
            >
              <span class="sky-btn-icon">🌸</span>
              <span class="sky-btn-label">Sakura</span>
            </button>
            <button
              type="button"
              class="sky-theme-btn"
              class:active={settings.ambient_sky_theme === 'clouds'}
              onclick={() => islandState.updateSettings({ ambient_sky_theme: 'clouds' })}
              title="Warm Sunset Clouds & Twilight Glow"
            >
              <span class="sky-btn-icon">⛅</span>
              <span class="sky-btn-label">Clouds</span>
            </button>
            <button
              type="button"
              class="sky-theme-btn"
              class:active={settings.ambient_sky_theme === 'snow'}
              onclick={() => islandState.updateSettings({ ambient_sky_theme: 'snow' })}
              title="Crystalline Snowflakes & Gentle Drift"
            >
              <span class="sky-btn-icon">❄️</span>
              <span class="sky-btn-label">Snow</span>
            </button>
          </div>
        </div>

        <!-- Sky Intensity Selector -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Visual Density & Opacity</span>
            <span class="setting-desc">Particle visibility and atmospheric brightness</span>
          </div>
          <div class="position-segmented" style="max-width: 240px;">
            <button
              type="button"
              class="position-btn"
              class:active={settings.ambient_sky_intensity === 'subtle'}
              onclick={() => islandState.updateSettings({ ambient_sky_intensity: 'subtle' })}
              title="Subtle (15% opacity, serene)"
            >
              <span class="pos-label">Subtle</span>
            </button>
            <button
              type="button"
              class="position-btn"
              class:active={settings.ambient_sky_intensity === 'medium'}
              onclick={() => islandState.updateSettings({ ambient_sky_intensity: 'medium' })}
              title="Medium (28% opacity, balanced)"
            >
              <span class="pos-label">Medium</span>
            </button>
            <button
              type="button"
              class="position-btn"
              class:active={settings.ambient_sky_intensity === 'vivid'}
              onclick={() => islandState.updateSettings({ ambient_sky_intensity: 'vivid' })}
              title="Vivid (45% opacity, expressive)"
            >
              <span class="pos-label">Vivid</span>
            </button>
          </div>
        </div>

        <!-- Live Weather Status Card & City Config -->
        <div class="weather-dashboard-card">
          {#if weather}
            <div class="weather-main-row">
              <div class="weather-left-info">
                <span class="weather-big-icon">{weatherIcon}</span>
                <div class="weather-temp-group">
                  <span class="weather-temp-c">{Math.round(weather.temp_c)}°C</span>
                  <span class="weather-temp-f">{Math.round(weather.temp_f)}°F</span>
                </div>
              </div>
              <div class="weather-center-info">
                <span class="weather-condition-title">{weather.description || weather.condition}</span>
                <span class="weather-loc-subtitle">📍 {weather.location || 'Detected Location'}</span>
              </div>
              <div class="weather-right-actions">
                <button
                  type="button"
                  class="weather-refresh-btn"
                  onclick={() => islandState.refreshWeather()}
                  title="Query latest weather from wttr.in"
                >
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="23 4 23 10 17 10"></polyline>
                    <polyline points="1 20 1 14 7 14"></polyline>
                    <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
                  </svg>
                  <span>Refresh</span>
                </button>
              </div>
            </div>

            <div class="weather-details-grid">
              <div class="weather-stat-chip">
                <span class="stat-icon">💧</span>
                <span class="stat-label">Humidity:</span>
                <span class="stat-value">{weather.humidity}%</span>
              </div>
              <div class="weather-stat-chip">
                <span class="stat-icon">☁️</span>
                <span class="stat-label">Clouds:</span>
                <span class="stat-value">{weather.cloud_cover}%</span>
              </div>
              {#if weather.sunrise}
                <div class="weather-stat-chip">
                  <span class="stat-icon">🌅</span>
                  <span class="stat-label">Sunrise:</span>
                  <span class="stat-value">{weather.sunrise}</span>
                </div>
              {/if}
              {#if weather.sunset}
                <div class="weather-stat-chip">
                  <span class="stat-icon">🌇</span>
                  <span class="stat-label">Sunset:</span>
                  <span class="stat-value">{weather.sunset}</span>
                </div>
              {/if}
            </div>
          {:else}
            <div class="weather-loading-row">
              <span class="weather-loading-icon">🌤️</span>
              <span>Fetching live local weather...</span>
            </div>
          {/if}

          <!-- City Override Input -->
          <div class="weather-city-input-row">
            <span class="city-input-label">City:</span>
            <input
              type="text"
              class="city-text-input"
              placeholder="e.g. London, Tokyo (blank for auto IP)"
              value={cityInput}
              oninput={(e) => { cityInput = (e.target as HTMLInputElement).value; }}
              onkeydown={(e) => { if (e.key === 'Enter') handleSaveCity(); }}
            />
            <button type="button" class="btn-save-city" onclick={handleSaveCity}>
              Save
            </button>
          </div>
        </div>
      {/if}
    </div>

    <!-- Section 5: Desktop Cyber-Pet Companion -->
    <div class="settings-group">
      <div class="group-header">
        <div class="group-title-col">
          <span class="group-title">Desktop Cyber-Pet Companion</span>
          <span class="group-desc">Interactive pixel-art mascot reacting to music, builds, and AI agent</span>
        </div>
        {#if settings.mascot_enabled !== false}
          <span class="mascot-status-pill">Active</span>
        {/if}
      </div>

      <!-- Toggle Mascot -->
      <div class="setting-item">
        <div class="setting-info">
          <span class="setting-name">Enable Cyber-Pet</span>
          <span class="setting-desc">Perches in compact pill and expanded topbar</span>
        </div>
        <button
          type="button"
          class="toggle-switch"
          class:checked={settings.mascot_enabled !== false}
          onclick={() => islandState.updateSettings({ mascot_enabled: settings.mascot_enabled === false })}
          aria-label="Toggle mascot companion"
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>

      {#if settings.mascot_enabled !== false}
        <!-- Interactive Petting Toggle -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Interactive Petting</span>
            <span class="setting-desc">Click on mascot to pet it, trigger happy purrs & floating hearts</span>
          </div>
          <button
            type="button"
            class="toggle-switch"
            class:checked={settings.mascot_interactive !== false}
            onclick={() => islandState.updateSettings({ mascot_interactive: settings.mascot_interactive === false })}
            aria-label="Toggle interactive petting"
          >
            <span class="toggle-thumb"></span>
          </button>
        </div>

        <!-- Mascot Character Selector -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Mascot Character</span>
            <span class="setting-desc">Choose your companion persona</span>
          </div>
          <div class="mascot-character-grid">
            <button
              type="button"
              class="mascot-choice-btn"
              class:active={settings.mascot_type === 'cat' || !settings.mascot_type}
              onclick={() => islandState.updateSettings({ mascot_type: 'cat' })}
            >
              <span class="mascot-choice-icon">🐱</span>
              <div class="mascot-choice-text">
                <span class="mascot-choice-title">Cyber-Cat</span>
                <span class="mascot-choice-desc">Bongo typing & neon beats</span>
              </div>
            </button>
            <button
              type="button"
              class="mascot-choice-btn"
              class:active={settings.mascot_type === 'duck'}
              onclick={() => islandState.updateSettings({ mascot_type: 'duck' })}
            >
              <span class="mascot-choice-icon">🦆</span>
              <div class="mascot-choice-text">
                <span class="mascot-choice-title">Chill Duck</span>
                <span class="mascot-choice-desc">Sunglasses & wing typing</span>
              </div>
            </button>
            <button
              type="button"
              class="mascot-choice-btn"
              class:active={settings.mascot_type === 'robot'}
              onclick={() => islandState.updateSettings({ mascot_type: 'robot' })}
            >
              <span class="mascot-choice-icon">🤖</span>
              <div class="mascot-choice-text">
                <span class="mascot-choice-title">Retro Bot</span>
                <span class="mascot-choice-desc">Equalizer CRT & turbo treads</span>
              </div>
            </button>
            <button
              type="button"
              class="mascot-choice-btn"
              class:active={settings.mascot_type === 'fox'}
              onclick={() => islandState.updateSettings({ mascot_type: 'fox' })}
            >
              <span class="mascot-choice-icon">🦊</span>
              <div class="mascot-choice-text">
                <span class="mascot-choice-title">Kitsune Fox</span>
                <span class="mascot-choice-desc">Fluffy tail & spirit flame</span>
              </div>
            </button>
          </div>
        </div>

        <!-- Live Interactive Playground Card -->
        <div class="mascot-preview-card">
          <div class="mascot-preview-avatar">
            <PixelMascot size="lg" stateOverride={previewMascotMood} />
          </div>
          <div class="mascot-preview-content">
            <div class="mascot-preview-header">
              <span class="mascot-preview-name">{
                settings.mascot_type === 'duck' ? 'Chill Duck' :
                settings.mascot_type === 'robot' ? 'Retro Bot' :
                settings.mascot_type === 'fox' ? 'Kitsune Fox' : 'Cyber-Cat'
              } Playground</span>
              <span class="mascot-preview-hint">Test reactive expressions below or click mascot to pet:</span>
            </div>
            <div class="mood-chips-row">
              <button
                type="button"
                class="mood-chip"
                class:active={previewMascotMood === undefined}
                onclick={() => (previewMascotMood = undefined)}
              >
                Auto (Live)
              </button>
              <button
                type="button"
                class="mood-chip"
                class:active={previewMascotMood === 'sleeping'}
                onclick={() => (previewMascotMood = 'sleeping')}
              >
                😴 Sleeping
              </button>
              <button
                type="button"
                class="mood-chip"
                class:active={previewMascotMood === 'music'}
                onclick={() => (previewMascotMood = 'music')}
              >
                🎧 Grooving
              </button>
              <button
                type="button"
                class="mood-chip"
                class:active={previewMascotMood === 'typing'}
                onclick={() => (previewMascotMood = 'typing')}
              >
                ⚡ Bongo Typing
              </button>
              <button
                type="button"
                class="mood-chip"
                class:active={previewMascotMood === 'agent'}
                onclick={() => (previewMascotMood = 'agent')}
              >
                🤖 AI Visor
              </button>
              <button
                type="button"
                class="mood-chip"
                class:active={previewMascotMood === 'petting'}
                onclick={() => (previewMascotMood = 'petting')}
              >
                ♥ Purring
              </button>
            </div>
          </div>
        </div>
      {/if}
    </div>

    <!-- Section 6: Keyboard Shortcuts -->
    <div class="settings-group">
      <div class="group-header">
        <span class="group-title">Keyboard Shortcuts</span>
        <button type="button" class="reset-small-btn" onclick={resetAllShortcuts}>Reset</button>
      </div>
      <p class="group-desc">Global hotkeys — work even when the island is not focused. Click a shortcut badge to remap it, then press the new key combo.</p>

      {#each SHORTCUT_DEFINITIONS as def (def.key)}
        {@const current = settings.shortcuts?.[def.key] ?? ''}
        {@const isRecording = recordingKey === def.key}
        <div class="setting-item shortcut-row">
          <div class="setting-info">
            <span class="setting-name">{def.icon} {def.label}</span>
          </div>
          <div class="shortcut-controls">
            {#if isRecording}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
              <div
                class="shortcut-capture active"
                tabindex="0"
                onkeydown={(e) => handleShortcutKeydown(e, def.key)}
                onblur={cancelRecording}
              >
                {recordingValue || 'Press keys…'}
              </div>
              <button type="button" class="shortcut-cancel-btn" onclick={cancelRecording} title="Cancel">✕</button>
            {:else}
              <button
                type="button"
                class="shortcut-badge"
                class:empty={!current}
                onclick={() => startRecording(def.key)}
                title="Click to remap"
              >
                {current || 'Unset'}
              </button>
              {#if current}
                <button
                  type="button"
                  class="shortcut-clear-btn"
                  onclick={() => clearShortcut(def.key)}
                  title="Clear shortcut"
                >✕</button>
              {/if}
            {/if}
          </div>
        </div>
      {/each}
    </div>

    <!-- Section 7: System Integration -->
    <div class="settings-group">
      <span class="group-title">System Integration</span>

      <div class="setting-item">
        <div class="setting-info">
          <span class="setting-name">Launch on System Login</span>
          <span class="setting-desc">Auto-start via ~/.config/autostart</span>
        </div>
        <button
          type="button"
          class="toggle-switch"
          class:checked={settings.autostart_enabled}
          onclick={() => handleToggleModule('autostart')}
          aria-label="Toggle launch on login"
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>
    </div>
  </div>

  <!-- Bottom Action Bar -->
  <div class="settings-bottom-bar">
    <button type="button" class="reset-btn" onclick={handleReset}>
      Reset to Defaults
    </button>
  </div>
</div>

<style>
  .settings-wrapper {
    display: flex;
    flex-direction: column;
    gap: 10px;
    width: 100%;
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .settings-scroll-area {
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-height: 390px;
    overflow-y: auto;
    padding-right: 4px;
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.15) transparent;
  }

  /* Custom thin scrollbar */
  .settings-scroll-area::-webkit-scrollbar {
    width: 4px;
  }
  .settings-scroll-area::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.15);
    border-radius: 4px;
  }

  .settings-group {
    background: var(--island-card-bg, rgba(255, 255, 255, 0.04));
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    border-radius: 12px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .group-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 2px;
  }

  .active-theme-tag {
    font-size: 9.5px;
    font-weight: 700;
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--island-card-bg, rgba(255, 255, 255, 0.08));
    color: var(--island-accent, #3b82f6);
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.1));
    letter-spacing: 0.05em;
  }

  .theme-grid {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 6px;
    margin-top: 4px;
  }

  .theme-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 6px 4px;
    border-radius: 8px;
    background: var(--island-card-bg, rgba(255, 255, 255, 0.04));
    border: 1.5px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    min-width: 0;
  }

  .theme-card:hover {
    background: var(--island-card-hover, rgba(255, 255, 255, 0.08));
    border-color: var(--island-border-hover, rgba(255, 255, 255, 0.2));
    transform: translateY(-1px);
  }

  .theme-card.active {
    border-color: var(--island-accent, #3b82f6);
    background: var(--island-card-hover, rgba(255, 255, 255, 0.1));
    box-shadow: 0 0 10px rgba(59, 130, 246, 0.3);
  }

  .theme-swatch {
    width: 100%;
    height: 22px;
    border-radius: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 0 4px;
    box-sizing: border-box;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
  }

  .swatch-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .swatch-bar {
    width: 18px;
    height: 3.5px;
    border-radius: 2px;
    flex-shrink: 0;
  }

  /* Dark Swatch */
  .dark-swatch {
    background: #000000;
    border: 1px solid rgba(255, 255, 255, 0.25);
  }
  .dark-swatch .swatch-dot {
    background: #10b981;
  }
  .dark-swatch .swatch-bar {
    background: rgba(255, 255, 255, 0.6);
  }

  /* Light Swatch */
  .light-swatch {
    background: #ffffff;
    border: 1px solid rgba(0, 0, 0, 0.15);
  }
  .light-swatch .swatch-dot {
    background: #2563eb;
  }
  .light-swatch .swatch-bar {
    background: #475569;
  }

  /* Neon Swatch */
  .neon-swatch {
    background: #090a18;
    border: 1.5px solid #00f0ff;
    box-shadow: 0 0 8px rgba(0, 240, 255, 0.5);
  }
  .neon-swatch .swatch-dot {
    background: #00f0ff;
    box-shadow: 0 0 4px #00f0ff;
  }
  .neon-swatch .swatch-bar {
    background: #ff007f;
    box-shadow: 0 0 4px #ff007f;
  }

  /* Anime Swatch */
  .anime-swatch {
    background: #231633;
    border: 1.5px solid #ff77a8;
    box-shadow: 0 0 8px rgba(255, 119, 168, 0.45);
  }
  .anime-swatch .swatch-dot {
    background: #ff77a8;
    box-shadow: 0 0 4px #ff77a8;
  }
  .anime-swatch .swatch-bar {
    background: #c084fc;
    box-shadow: 0 0 4px #c084fc;
  }

  /* Catppuccin Swatch */
  .catppuccin-swatch {
    background: #181825;
    border: 1px solid rgba(203, 166, 247, 0.45);
  }
  .catppuccin-swatch .swatch-dot {
    background: #a6e3a1;
  }
  .catppuccin-swatch .swatch-bar {
    background: #cba6f7;
  }

  /* Nord Swatch */
  .nord-swatch {
    background: #242933;
    border: 1px solid rgba(136, 192, 208, 0.45);
  }
  .nord-swatch .swatch-dot {
    background: #88c0d0;
  }
  .nord-swatch .swatch-bar {
    background: #81a1c1;
  }

  /* Dracula Swatch */
  .dracula-swatch {
    background: #1e1f29;
    border: 1px solid rgba(189, 147, 249, 0.45);
  }
  .dracula-swatch .swatch-dot {
    background: #50fa7b;
  }
  .dracula-swatch .swatch-bar {
    background: #bd93f9;
  }

  /* Tokyo Night Swatch */
  .tokyo-swatch {
    background: #16161e;
    border: 1px solid rgba(122, 162, 247, 0.45);
  }
  .tokyo-swatch .swatch-dot {
    background: #7aa2f7;
  }
  .tokyo-swatch .swatch-bar {
    background: #bb9af7;
  }

  /* Gruvbox Swatch */
  .gruvbox-swatch {
    background: #1d2021;
    border: 1px solid rgba(254, 128, 25, 0.45);
  }
  .gruvbox-swatch .swatch-dot {
    background: #fe8019;
  }
  .gruvbox-swatch .swatch-bar {
    background: #ebdbb2;
  }

  /* Matrix Swatch */
  .matrix-swatch {
    background: #040804;
    border: 1.5px solid #00ff41;
    box-shadow: 0 0 6px rgba(0, 255, 65, 0.4);
  }
  .matrix-swatch .swatch-dot {
    background: #00ff41;
    box-shadow: 0 0 3px #00ff41;
  }
  .matrix-swatch .swatch-bar {
    background: #55ff77;
    box-shadow: 0 0 3px #55ff77;
  }

  .theme-name {
    font-size: 10px;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
    text-align: center;
  }

  .group-title {
    font-size: 10.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--island-accent, #3b82f6);
    margin-bottom: 2px;
  }

  .group-title-col {
    display: flex;
    flex-direction: column;
    gap: 1px;
    flex: 1;
    min-width: 0;
  }

  .group-desc {
    font-size: 9.5px;
    color: var(--island-text-muted, #94a3b8);
    line-height: 1.25;
  }

  .reset-order-btn {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: var(--island-text-secondary, #cbd5e1);
    font-size: 9.5px;
    font-weight: 600;
    padding: 2px 7px;
    border-radius: 5px;
    cursor: pointer;
    font-family: inherit;
    transition: all 0.15s ease;
    white-space: nowrap;
    align-self: flex-start;
  }

  .reset-order-btn:hover {
    background: rgba(255, 255, 255, 0.15);
    color: #ffffff;
    border-color: rgba(255, 255, 255, 0.25);
  }

  .module-order-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 4px;
  }

  .module-order-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    transition: all 0.15s ease;
    user-select: none;
  }

  .module-order-item:hover {
    background: rgba(255, 255, 255, 0.06);
    border-color: rgba(255, 255, 255, 0.12);
  }

  .module-order-item.is-dragging {
    opacity: 0.4;
    border: 1px dashed var(--island-accent, #3b82f6);
  }

  .module-order-item.drag-over {
    border-color: var(--island-accent, #3b82f6);
    background: rgba(59, 130, 246, 0.18);
    transform: translateY(2px) scale(1.01);
    box-shadow: 0 0 10px rgba(59, 130, 246, 0.3);
  }

  .module-item-left {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    min-width: 0;
  }

  .drag-handle {
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: grab;
    color: var(--island-text-muted, #94a3b8);
    opacity: 0.6;
    padding: 3px 4px;
    touch-action: none;
    user-select: none;
    -webkit-user-select: none;
    transition: opacity 0.15s ease, color 0.15s ease;
  }

  .drag-handle:hover {
    opacity: 1;
    color: #ffffff;
  }

  .drag-handle:active {
    cursor: grabbing;
  }

  .reorder-arrows {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .arrow-btn {
    background: none;
    border: none;
    padding: 0;
    margin: 0;
    width: 14px;
    height: 9px;
    line-height: 9px;
    font-size: 7.5px;
    color: var(--island-text-muted, #94a3b8);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 2px;
    transition: color 0.15s ease, background 0.15s ease;
  }

  .arrow-btn:hover:not(:disabled) {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.12);
  }

  .arrow-btn:disabled {
    opacity: 0.18;
    cursor: default;
  }

  .module-order-badge {
    font-size: 9px;
    font-weight: 700;
    color: var(--island-text-muted, #94a3b8);
    background: rgba(255, 255, 255, 0.06);
    border-radius: 4px;
    width: 15px;
    height: 15px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    font-variant-numeric: tabular-nums;
  }

  .module-icon {
    font-size: 13px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    flex-shrink: 0;
  }

  .system-toggles-divider {
    height: 1px;
    background: rgba(255, 255, 255, 0.08);
    margin: 4px 0;
  }

  .setting-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 3px 0;
  }

  .setting-item.slider-item {
    flex-direction: column;
    align-items: stretch;
    gap: 4px;
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .setting-name {
    font-size: 11.5px;
    font-weight: 500;
    color: var(--island-text-primary, #ffffff);
  }

  .setting-desc {
    font-size: 10px;
    color: var(--island-text-muted, #6b7280);
  }

  .slider-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .slider-val {
    font-size: 10.5px;
    font-weight: 600;
    color: var(--island-text-secondary, #9ca3af);
    font-variant-numeric: tabular-nums;
  }

  .range-slider {
    width: 100%;
    height: 4px;
    -webkit-appearance: none;
    appearance: none;
    background: rgba(255, 255, 255, 0.15);
    border-radius: 9999px;
    outline: none;
    margin: 4px 0;
  }

  .range-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--island-accent, #3b82f6);
    cursor: pointer;
    box-shadow: 0 0 6px rgba(59, 130, 246, 0.6);
    transition: transform 0.1s ease;
  }

  .range-slider::-webkit-slider-thumb:hover {
    transform: scale(1.15);
  }

  /* Toggle Switch */
  .toggle-switch {
    width: 32px;
    height: 18px;
    border-radius: 9999px;
    background: rgba(255, 255, 255, 0.15);
    border: none;
    cursor: pointer;
    position: relative;
    padding: 2px;
    transition: background-color 0.2s ease;
    flex-shrink: 0;
  }

  .toggle-switch.checked {
    background: var(--island-green, #10b981);
  }

  .toggle-thumb {
    display: block;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #ffffff;
    transition: transform 0.2s ease;
  }

  .toggle-switch.checked .toggle-thumb {
    transform: translateX(14px);
  }

  /* Position Controls */
  .position-segmented {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
    background: rgba(0, 0, 0, 0.25);
    padding: 3px;
    border-radius: 8px;
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
  }

  .position-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    padding: 6px 4px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 6px;
    color: var(--island-text-muted, #94a3b8);
    font-size: 11.5px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .position-btn:hover {
    color: var(--island-text-primary, #ffffff);
    background: rgba(255, 255, 255, 0.06);
  }

  .position-btn.active {
    background: var(--island-card-hover, rgba(255, 255, 255, 0.12));
    border-color: var(--island-accent, #3b82f6);
    color: var(--island-text-primary, #ffffff);
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
  }

  .pos-icon {
    font-size: 13px;
    line-height: 1;
  }

  .slider-title-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .reset-offset-btn {
    font-size: 9.5px;
    font-weight: 600;
    color: var(--island-accent, #3b82f6);
    background: rgba(59, 130, 246, 0.1);
    border: 1px solid rgba(59, 130, 246, 0.25);
    border-radius: 4px;
    padding: 1px 5px;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .reset-offset-btn:hover {
    background: rgba(59, 130, 246, 0.25);
  }

  .position-tip-card {
    background: rgba(0, 0, 0, 0.2);
    border: 1px dashed var(--island-card-border, rgba(255, 255, 255, 0.1));
    border-radius: 8px;
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 5px;
    margin-top: 2px;
  }

  .tip-line {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    font-size: 10.5px;
    line-height: 1.35;
    color: var(--island-text-muted, #94a3b8);
  }

  .tip-line strong {
    color: var(--island-text-primary, #ffffff);
  }

  .tip-line code {
    font-size: 9.5px;
    font-family: monospace;
    background: rgba(255, 255, 255, 0.08);
    padding: 1px 4px;
    border-radius: 3px;
    color: #38bdf8;
  }

  .tip-bullet {
    font-size: 12px;
    flex-shrink: 0;
  }

  /* Live Weather & Ambient Sky Settings */
  .weather-pill-indicator {
    font-size: 10px;
    font-weight: 600;
    color: var(--island-accent, #38bdf8);
    background: rgba(56, 189, 248, 0.12);
    border: 1px solid rgba(56, 189, 248, 0.28);
    padding: 2px 7px;
    border-radius: 9999px;
    font-variant-numeric: tabular-nums;
  }

  .sky-theme-segmented {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 4px;
    background: rgba(0, 0, 0, 0.25);
    padding: 3px;
    border-radius: 8px;
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    width: 100%;
    box-sizing: border-box;
  }

  .sky-theme-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 3px;
    padding: 6px 2px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 6px;
    color: var(--island-text-muted, #94a3b8);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .sky-theme-btn:hover {
    color: var(--island-text-primary, #ffffff);
    background: rgba(255, 255, 255, 0.05);
  }

  .sky-theme-btn.active {
    background: var(--island-accent, #3b82f6);
    color: #ffffff;
    border-color: rgba(255, 255, 255, 0.2);
    box-shadow: 0 0 10px rgba(59, 130, 246, 0.35);
  }

  .sky-btn-icon {
    font-size: 13px;
    line-height: 1;
  }

  .sky-btn-label {
    font-size: 9.5px;
    font-weight: 600;
  }

  .weather-dashboard-card {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    border-radius: 10px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .weather-main-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .weather-left-info {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .weather-big-icon {
    font-size: 22px;
    line-height: 1;
  }

  .weather-temp-group {
    display: flex;
    align-items: baseline;
    gap: 4px;
  }

  .weather-temp-c {
    font-size: 15px;
    font-weight: 700;
    color: var(--island-text-primary, #ffffff);
    font-variant-numeric: tabular-nums;
  }

  .weather-temp-f {
    font-size: 11px;
    color: var(--island-text-muted, #94a3b8);
    font-variant-numeric: tabular-nums;
  }

  .weather-center-info {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }

  .weather-condition-title {
    font-size: 11.5px;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .weather-loc-subtitle {
    font-size: 10px;
    color: var(--island-text-muted, #94a3b8);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .weather-refresh-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 6px;
    color: var(--island-text-secondary, #cbd5e1);
    font-size: 10.5px;
    font-weight: 500;
    padding: 4px 8px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .weather-refresh-btn:hover {
    background: rgba(255, 255, 255, 0.14);
    color: #ffffff;
    border-color: rgba(255, 255, 255, 0.25);
  }

  .weather-details-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
    background: rgba(0, 0, 0, 0.2);
    padding: 6px 8px;
    border-radius: 6px;
  }

  .weather-stat-chip {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 10px;
  }

  .stat-icon {
    font-size: 11px;
    line-height: 1;
  }

  .stat-label {
    color: var(--island-text-muted, #94a3b8);
  }

  .stat-value {
    color: var(--island-text-primary, #ffffff);
    font-weight: 600;
  }

  .weather-loading-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    color: var(--island-text-muted, #94a3b8);
    padding: 6px 0;
  }

  .weather-loading-icon {
    font-size: 15px;
  }

  .weather-city-input-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 2px;
  }

  .city-input-label {
    font-size: 10.5px;
    color: var(--island-text-muted, #94a3b8);
    flex-shrink: 0;
  }

  .city-text-input {
    flex: 1;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 6px;
    color: var(--island-text-primary, #ffffff);
    font-size: 11px;
    padding: 4px 8px;
    outline: none;
    transition: border-color 0.15s ease;
  }

  .city-text-input:focus {
    border-color: var(--island-accent, #3b82f6);
  }

  .city-text-input::placeholder {
    color: rgba(255, 255, 255, 0.3);
  }

  .btn-save-city {
    background: rgba(59, 130, 246, 0.2);
    border: 1px solid rgba(59, 130, 246, 0.4);
    color: #93c5fd;
    font-size: 10.5px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-save-city:hover {
    background: rgba(59, 130, 246, 0.35);
    color: #ffffff;
  }

  /* Desktop Cyber-Pet Styles */
  .mascot-status-pill {
    font-size: 10px;
    font-weight: 600;
    color: #10b981;
    background: rgba(16, 185, 129, 0.12);
    border: 1px solid rgba(16, 185, 129, 0.28);
    padding: 2px 7px;
    border-radius: 9999px;
  }

  .mascot-character-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
    width: 100%;
  }

  .mascot-choice-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 8px 4px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .mascot-choice-btn:hover:not(.disabled) {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .mascot-choice-btn.active {
    background: rgba(59, 130, 246, 0.15);
    border-color: var(--island-accent, #3b82f6);
    box-shadow: 0 0 12px rgba(59, 130, 246, 0.25);
  }

  .mascot-choice-btn.disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .mascot-choice-icon {
    font-size: 18px;
    line-height: 1;
  }

  .mascot-choice-text {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }

  .mascot-choice-title {
    font-size: 10.5px;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
  }

  .mascot-choice-desc {
    font-size: 8.5px;
    color: var(--island-text-muted, #94a3b8);
  }

  .mascot-choice-badge {
    font-size: 8px;
    font-weight: 600;
    color: #94a3b8;
    background: rgba(255, 255, 255, 0.1);
    padding: 1px 4px;
    border-radius: 4px;
  }

  .mascot-preview-card {
    display: flex;
    align-items: center;
    gap: 12px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    border-radius: 10px;
    padding: 10px 12px;
  }

  .mascot-preview-avatar {
    width: 72px;
    height: 72px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.35);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    flex-shrink: 0;
  }

  .mascot-preview-content {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 1;
    min-width: 0;
  }

  .mascot-preview-header {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .mascot-preview-name {
    font-size: 11.5px;
    font-weight: 700;
    color: var(--island-text-primary, #ffffff);
  }

  .mascot-preview-hint {
    font-size: 9.5px;
    color: var(--island-text-muted, #94a3b8);
  }

  .mood-chips-row {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .mood-chip {
    font-size: 9.5px;
    font-weight: 500;
    padding: 2px 7px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: var(--island-text-secondary, #cbd5e1);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .mood-chip:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #ffffff;
  }

  .mood-chip.active {
    background: var(--island-accent, #3b82f6);
    color: #ffffff;
    border-color: rgba(255, 255, 255, 0.25);
    font-weight: 600;
    box-shadow: 0 0 8px rgba(59, 130, 246, 0.3);
  }

  /* Bottom bar */
  .settings-bottom-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 4px;
  }

  .back-link {
    background: none;
    border: none;
    color: var(--island-text-secondary, #9ca3af);
    font-size: 11px;
    cursor: pointer;
    font-family: inherit;
    padding: 4px 8px;
    border-radius: 6px;
    transition: color 0.15s ease, background 0.15s ease;
  }

  .back-link:hover {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.08);
  }

  .reset-btn {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: var(--island-text-secondary, #9ca3af);
    font-size: 10px;
    padding: 3px 8px;
    border-radius: 6px;
    cursor: pointer;
    font-family: inherit;
    transition: background 0.15s ease, color 0.15s ease;
  }

  .reset-btn:hover {
    background: rgba(255, 255, 255, 0.16);
    color: #ffffff;
  }

  /* ---- Keyboard Shortcuts ---- */
  .group-desc {
    font-size: 10.5px;
    color: var(--island-text-secondary, #9ca3af);
    margin: 0 0 6px 0;
    line-height: 1.45;
  }

  .shortcut-row {
    gap: 8px;
  }

  .shortcut-controls {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .shortcut-badge {
    font-family: 'SFMono-Regular', 'Consolas', 'Liberation Mono', monospace;
    font-size: 10.5px;
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.14);
    color: var(--island-accent, #10b981);
    padding: 3px 8px;
    border-radius: 6px;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s ease, border-color 0.15s ease, color 0.15s ease;
    min-width: 120px;
    text-align: center;
  }

  .shortcut-badge:hover {
    background: rgba(255, 255, 255, 0.14);
    border-color: var(--island-accent, #10b981);
    color: #ffffff;
  }

  .shortcut-badge.empty {
    color: var(--island-text-secondary, #9ca3af);
    font-style: italic;
  }

  .shortcut-capture {
    font-family: 'SFMono-Regular', 'Consolas', 'Liberation Mono', monospace;
    font-size: 10.5px;
    background: rgba(16, 185, 129, 0.12);
    border: 1.5px solid var(--island-accent, #10b981);
    color: #ffffff;
    padding: 3px 8px;
    border-radius: 6px;
    min-width: 120px;
    text-align: center;
    white-space: nowrap;
    outline: none;
    animation: shortcut-pulse 1.2s ease-in-out infinite;
    cursor: text;
  }

  @keyframes shortcut-pulse {
    0%, 100% { box-shadow: 0 0 0 0 rgba(16, 185, 129, 0.35); }
    50%       { box-shadow: 0 0 0 3px rgba(16, 185, 129, 0); }
  }

  .shortcut-clear-btn,
  .shortcut-cancel-btn {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.06);
    color: var(--island-text-secondary, #9ca3af);
    font-size: 9px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: background 0.15s ease, color 0.15s ease;
  }

  .shortcut-clear-btn:hover,
  .shortcut-cancel-btn:hover {
    background: rgba(239, 68, 68, 0.25);
    color: #ef4444;
    border-color: rgba(239, 68, 68, 0.4);
  }

  .reset-small-btn {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: var(--island-text-secondary, #9ca3af);
    font-size: 9.5px;
    padding: 2px 7px;
    border-radius: 5px;
    cursor: pointer;
    font-family: inherit;
    transition: background 0.15s ease, color 0.15s ease;
  }

  .reset-small-btn:hover {
    background: rgba(255, 255, 255, 0.14);
    color: #ffffff;
  }
</style>
