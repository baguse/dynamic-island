<script lang="ts">
  import IslandHeader from './IslandHeader.svelte';
  import { islandState } from '../state/island.svelte';

  let isRefreshing = $state(false);

  const audio = $derived(islandState.audio);
  const isMicMuted = $derived(audio?.mic_muted ?? false);
  const micVolume = $derived(audio?.mic_volume ?? 100);
  const isOutputMuted = $derived(audio?.output_muted ?? false);
  const outputVolume = $derived(audio?.output_volume ?? 100);
  const outputs = $derived(audio?.outputs ?? []);

  async function handleRefresh(e: MouseEvent) {
    e.stopPropagation();
    isRefreshing = true;
    await islandState.refreshAudio();
    setTimeout(() => {
      isRefreshing = false;
    }, 350);
  }

  async function handleToggleMic(e: MouseEvent) {
    e.stopPropagation();
    await islandState.toggleMicMute();
  }

  async function handleMicVolumeChange(e: Event) {
    const val = Number((e.target as HTMLInputElement).value);
    await islandState.setMicVolume(val);
  }

  async function handleToggleOutputMute(e: MouseEvent) {
    e.stopPropagation();
    await islandState.toggleOutputMute();
  }

  async function handleOutputVolumeChange(e: Event) {
    const val = Number((e.target as HTMLInputElement).value);
    await islandState.setOutputVolume(val);
  }

  async function handleSelectOutput(id: string) {
    await islandState.setAudioOutput(id);
  }
</script>

<div class="hardware-preview-wrapper">
  <IslandHeader title="Hardware & Audio">
    <button
      type="button"
      class="btn-icon-refresh"
      onclick={handleRefresh}
      title="Refresh audio devices"
      aria-label="Refresh audio devices"
      disabled={isRefreshing}
    >
      <svg class:spinning={isRefreshing} width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="23 4 23 10 17 10"></polyline>
        <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
      </svg>
    </button>
  </IslandHeader>

  <div class="hardware-content custom-scroll">
    <!-- Microphone Card with Glowing Mute Badge -->
    <div class="control-card mic-card" class:muted={isMicMuted}>
      <div class="card-header">
        <div class="header-left-info">
          <div class="icon-avatar mic-avatar" class:muted={isMicMuted}>
            {#if isMicMuted}
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="1" y1="1" x2="23" y2="23"></line>
                <path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6"></path>
                <path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2a7 7 0 0 1-.11 1.23"></path>
                <line x1="12" y1="19" x2="12" y2="23"></line>
                <line x1="8" y1="23" x2="16" y2="23"></line>
              </svg>
            {:else}
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"></path>
                <path d="M19 10v2a7 7 0 0 1-14 0v-2"></path>
                <line x1="12" y1="19" x2="12" y2="23"></line>
                <line x1="8" y1="23" x2="16" y2="23"></line>
              </svg>
            {/if}
          </div>
          <div>
            <div class="card-title">Microphone</div>
            <div class="card-subtitle">
              {isMicMuted ? 'Muted — meeting safe' : 'Active & listening'}
            </div>
          </div>
        </div>

        <!-- 1-Click Toggle Button with Glow -->
        <button
          type="button"
          class="mute-toggle-btn"
          class:is-muted={isMicMuted}
          onclick={handleToggleMic}
          title={isMicMuted ? 'Click to Unmute' : 'Click to Mute'}
        >
          <span class="status-pulse-dot" class:muted-dot={isMicMuted}></span>
          <span>{isMicMuted ? 'Unmute' : 'Mute'}</span>
        </button>
      </div>

      <!-- Mic Volume Slider -->
      <div class="volume-slider-row">
        <span class="volume-icon">🎙️</span>
        <input
          type="range"
          min="0"
          max="100"
          value={micVolume}
          onchange={handleMicVolumeChange}
          class="slider-input"
          aria-label="Microphone volume"
        />
        <span class="volume-pct">{micVolume}%</span>
      </div>
    </div>

    <!-- Output Volume & Switching -->
    <div class="control-card output-card">
      <div class="card-header">
        <div class="header-left-info">
          <div class="icon-avatar output-avatar">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
              <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
              <path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"></path>
            </svg>
          </div>
          <div>
            <div class="card-title">Audio Output</div>
            <div class="card-subtitle">
              {audio?.active_output_name || 'System Default'}
            </div>
          </div>
        </div>

        <button
          type="button"
          class="btn-output-mute"
          class:muted={isOutputMuted}
          onclick={handleToggleOutputMute}
          title={isOutputMuted ? 'Unmute Output' : 'Mute Output'}
        >
          {isOutputMuted ? 'Muted' : 'Sound On'}
        </button>
      </div>

      <!-- Master Volume Slider -->
      <div class="volume-slider-row">
        <span class="volume-icon">🔊</span>
        <input
          type="range"
          min="0"
          max="100"
          value={outputVolume}
          onchange={handleOutputVolumeChange}
          class="slider-input"
          aria-label="Master output volume"
        />
        <span class="volume-pct">{outputVolume}%</span>
      </div>

      <!-- Audio Output Devices List (Headphones vs Speakers) -->
      <div class="output-devices-list">
        {#each outputs as device (device.id)}
          <button
            type="button"
            class="device-chip-btn"
            class:active={device.is_active}
            onclick={() => handleSelectOutput(device.id)}
          >
            <div class="device-icon">
              {#if device.device_type === 'headphones'}
                🎧
              {:else if device.device_type === 'speakers'}
                🔈
              {:else}
                📻
              {/if}
            </div>
            <div class="device-details">
              <div class="device-name">{device.name}</div>
              <div class="device-desc">{device.description}</div>
            </div>
            {#if device.is_active}
              <div class="active-badge">Active</div>
            {/if}
          </button>
        {/each}
      </div>
    </div>
  </div>
</div>

<style>
  .hardware-preview-wrapper {
    display: flex;
    flex-direction: column;
    height: 100%;
    color: var(--text-primary, #ffffff);
    box-sizing: border-box;
  }

  .btn-icon-refresh {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: var(--text-secondary, #94a3b8);
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    padding: 0;
  }

  .btn-icon-refresh:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.18);
    color: #ffffff;
    transform: rotate(45deg);
  }

  .spinning {
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .hardware-content {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 6px 4px 10px 4px;
    overflow-y: auto;
    max-height: 310px;
  }

  .control-card {
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    transition: all 0.25s ease;
  }

  .mic-card.muted {
    background: rgba(239, 68, 68, 0.07);
    border-color: rgba(239, 68, 68, 0.28);
    box-shadow: 0 0 16px rgba(239, 68, 68, 0.12);
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .header-left-info {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .icon-avatar {
    width: 32px;
    height: 32px;
    border-radius: 10px;
    background: rgba(34, 197, 94, 0.14);
    color: #22c55e;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.25s ease;
  }

  .icon-avatar.muted {
    background: rgba(239, 68, 68, 0.18);
    color: #ef4444;
  }

  .output-avatar {
    background: rgba(59, 130, 246, 0.14);
    color: #3b82f6;
  }

  .card-title {
    font-size: 13px;
    font-weight: 600;
    color: #f8fafc;
  }

  .card-subtitle {
    font-size: 11px;
    color: var(--text-secondary, #94a3b8);
  }

  .mute-toggle-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: 20px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    background: rgba(34, 197, 94, 0.15);
    border: 1px solid rgba(34, 197, 94, 0.35);
    color: #4ade80;
    transition: all 0.2s ease;
  }

  .mute-toggle-btn:hover {
    background: rgba(34, 197, 94, 0.25);
    transform: scale(1.02);
  }

  .mute-toggle-btn.is-muted {
    background: rgba(239, 68, 68, 0.2);
    border-color: rgba(239, 68, 68, 0.5);
    color: #fca5a5;
    box-shadow: 0 0 12px rgba(239, 68, 68, 0.35);
  }

  .mute-toggle-btn.is-muted:hover {
    background: rgba(239, 68, 68, 0.3);
  }

  .status-pulse-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #22c55e;
    box-shadow: 0 0 6px #22c55e;
  }

  .status-pulse-dot.muted-dot {
    background: #ef4444;
    box-shadow: 0 0 8px #ef4444;
    animation: pulse-red 1.8s infinite;
  }

  @keyframes pulse-red {
    0% { transform: scale(0.95); opacity: 0.8; }
    50% { transform: scale(1.2); opacity: 1; }
    100% { transform: scale(0.95); opacity: 0.8; }
  }

  .btn-output-mute {
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 500;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: #cbd5e1;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-output-mute.muted {
    background: rgba(239, 68, 68, 0.15);
    border-color: rgba(239, 68, 68, 0.3);
    color: #fca5a5;
  }

  .volume-slider-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 2px 0;
  }

  .volume-icon {
    font-size: 13px;
    user-select: none;
  }

  .slider-input {
    flex: 1;
    accent-color: #3b82f6;
    height: 4px;
    border-radius: 2px;
    cursor: pointer;
  }

  .volume-pct {
    font-size: 11px;
    font-family: monospace;
    color: var(--text-secondary, #94a3b8);
    min-width: 32px;
    text-align: right;
  }

  .output-devices-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 4px;
  }

  .device-chip-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 10px;
    padding: 8px 10px;
    cursor: pointer;
    text-align: left;
    transition: all 0.2s ease;
    color: inherit;
  }

  .device-chip-btn:hover {
    background: rgba(255, 255, 255, 0.07);
    border-color: rgba(255, 255, 255, 0.12);
  }

  .device-chip-btn.active {
    background: rgba(59, 130, 246, 0.12);
    border-color: rgba(59, 130, 246, 0.4);
    box-shadow: 0 0 10px rgba(59, 130, 246, 0.15);
  }

  .device-icon {
    font-size: 16px;
    line-height: 1;
  }

  .device-details {
    flex: 1;
    min-width: 0;
  }

  .device-name {
    font-size: 12px;
    font-weight: 500;
    color: #f1f5f9;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .device-desc {
    font-size: 10px;
    color: var(--text-secondary, #94a3b8);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .active-badge {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #60a5fa;
    background: rgba(59, 130, 246, 0.2);
    padding: 2px 6px;
    border-radius: 6px;
  }

  .custom-scroll::-webkit-scrollbar {
    width: 4px;
  }

  .custom-scroll::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.12);
    border-radius: 4px;
  }
</style>
