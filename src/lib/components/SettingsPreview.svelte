<script lang="ts">
  import IslandHeader from './IslandHeader.svelte';
  import { islandState } from '../state/island.svelte';
  import type { IslandTheme, ModuleKey, IslandSettings } from '../types';

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
    </div>

    <!-- Section 4: System Integration -->
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
</style>
