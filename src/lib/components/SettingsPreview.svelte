<script lang="ts">
  import IslandHeader from './IslandHeader.svelte';
  import { islandState } from '../state/island.svelte';
  import type { IslandTheme } from '../types';

  const settings = $derived(islandState.settings);

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

  function handleToggleModule(module: 'system' | 'battery' | 'media' | 'build' | 'agent' | 'git' | 'editor' | 'ports' | 'docker' | 'mic' | 'shelf' | 'clipboard' | 'autostart') {
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

  function handleBack(e: MouseEvent) {
    e.stopPropagation();
    islandState.openOverview();
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
        <button
          type="button"
          class="theme-card"
          class:active={settings.theme === 'dark'}
          onclick={() => handleThemeChange('dark')}
          aria-label="Select Dark OLED Theme"
          title="Dark OLED Theme"
        >
          <div class="theme-swatch dark-swatch">
            <span class="swatch-dot"></span>
            <span class="swatch-bar"></span>
          </div>
          <span class="theme-name">Dark</span>
        </button>

        <button
          type="button"
          class="theme-card"
          class:active={settings.theme === 'light'}
          onclick={() => handleThemeChange('light')}
          aria-label="Select Light Acrylic Theme"
          title="Light Acrylic Theme"
        >
          <div class="theme-swatch light-swatch">
            <span class="swatch-dot"></span>
            <span class="swatch-bar"></span>
          </div>
          <span class="theme-name">Light</span>
        </button>

        <button
          type="button"
          class="theme-card"
          class:active={settings.theme === 'neon'}
          onclick={() => handleThemeChange('neon')}
          aria-label="Select Neon Cyberpunk Theme"
          title="Neon Cyberpunk Theme"
        >
          <div class="theme-swatch neon-swatch">
            <span class="swatch-dot"></span>
            <span class="swatch-bar"></span>
          </div>
          <span class="theme-name">Neon</span>
        </button>

        <button
          type="button"
          class="theme-card"
          class:active={settings.theme === 'anime'}
          onclick={() => handleThemeChange('anime')}
          aria-label="Select Anime Sakura Theme"
          title="Anime Sakura Theme"
        >
          <div class="theme-swatch anime-swatch">
            <span class="swatch-dot"></span>
            <span class="swatch-bar"></span>
          </div>
          <span class="theme-name">Anime</span>
        </button>
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
      <span class="group-title">Position & Placement</span>

      <div class="setting-item slider-item">
        <div class="slider-header">
          <span class="setting-name">Top Screen Margin</span>
          <span class="slider-val">{settings.top_margin} px</span>
        </div>
        <input
          type="range"
          min="4"
          max="36"
          step="2"
          value={settings.top_margin}
          oninput={handleTopMargin}
          class="range-slider"
        />
      </div>
    </div>

    <!-- Section 3: Active Modules & Features -->
    <div class="settings-group">
      <span class="group-title">Modules & Features</span>

      <div class="setting-item">
        <span class="setting-name">CPU & RAM Telemetry</span>
        <button
          type="button"
          class="toggle-switch"
          class:checked={settings.show_system_in_pill}
          onclick={() => handleToggleModule('system')}
          aria-label="Toggle system telemetry module"
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>

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

      <div class="setting-item">
        <span class="setting-name">Media Player (MPRIS)</span>
        <button
          type="button"
          class="toggle-switch"
          class:checked={settings.show_media_in_pill}
          onclick={() => handleToggleModule('media')}
          aria-label="Toggle media player module"
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>

      <div class="setting-item">
        <span class="setting-name">Terminal Build Monitor</span>
        <button
          type="button"
          class="toggle-switch"
          class:checked={settings.show_build_in_pill}
          onclick={() => handleToggleModule('build')}
          aria-label="Toggle terminal build monitor"
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>

      <div class="setting-item">
        <span class="setting-name">AI Agent Companion</span>
        <button
          type="button"
          class="toggle-switch"
          class:checked={settings.show_agent_in_pill}
          onclick={() => handleToggleModule('agent')}
          aria-label="Toggle AI agent companion module"
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>

      <div class="setting-item">
        <span class="setting-name">Git Repository & Branch</span>
        <button
          type="button"
          class="toggle-switch"
          class:checked={settings.show_git_in_pill}
          onclick={() => handleToggleModule('git')}
          aria-label="Toggle git repository module"
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>

      <div class="setting-item">
        <span class="setting-name">Code Editor Presence (Cursor/VS Code)</span>
        <button
          type="button"
          class="toggle-switch"
          class:checked={settings.show_editor_in_pill}
          onclick={() => handleToggleModule('editor')}
          aria-label="Toggle code editor presence module"
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>

      <div class="setting-item">
        <span class="setting-name">Localhost & Ports Hub</span>
        <button
          type="button"
          class="toggle-switch"
          class:checked={settings.show_ports_in_pill}
          onclick={() => handleToggleModule('ports')}
          aria-label="Toggle localhost & ports module"
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>

      <div class="setting-item">
        <span class="setting-name">Docker Containers Hub</span>
        <button
          type="button"
          class="toggle-switch"
          class:checked={settings.show_docker_in_pill}
          onclick={() => handleToggleModule('docker')}
          aria-label="Toggle docker containers module"
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>

      <div class="setting-item">
        <span class="setting-name">Hardware & Audio (Mic Mute & Output Switcher)</span>
        <button
          type="button"
          class="toggle-switch"
          class:checked={settings.show_mic_in_pill}
          onclick={() => handleToggleModule('mic')}
          aria-label="Toggle hardware & audio module"
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>

      <div class="setting-item">
        <span class="setting-name">Desktop Drop Shelf (Drag & Drop Zone)</span>
        <button
          type="button"
          class="toggle-switch"
          class:checked={settings.show_shelf_in_pill}
          onclick={() => handleToggleModule('shelf')}
          aria-label="Toggle desktop drop shelf module"
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>

      <div class="setting-item">
        <span class="setting-name">Quick Clipboard History & Snippets</span>
        <button
          type="button"
          class="toggle-switch"
          class:checked={settings.show_clipboard_in_pill}
          onclick={() => handleToggleModule('clipboard')}
          aria-label="Toggle clipboard history module"
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>
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
    <button type="button" class="back-link" onclick={handleBack}>
      ← Back to Overview
    </button>
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
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
    margin-top: 4px;
  }

  .theme-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 8px 6px;
    border-radius: 10px;
    background: var(--island-card-bg, rgba(255, 255, 255, 0.04));
    border: 1.5px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
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
    height: 28px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 0 8px;
    box-sizing: border-box;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
  }

  .swatch-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .swatch-bar {
    width: 24px;
    height: 4px;
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

  .theme-name {
    font-size: 11px;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
  }

  .group-title {
    font-size: 10.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--island-accent, #3b82f6);
    margin-bottom: 2px;
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
