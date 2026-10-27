<script lang="ts">
  import { islandState } from '../state/island.svelte';
  import IslandHeader from './IslandHeader.svelte';

  const telemetry = $derived(islandState.telemetry);

  let isRefreshing = $state(false);

  function formatSpeed(kb: number): string {
    if (kb < 1) return '0 KB/s';
    if (kb < 1024) return `${kb.toFixed(1)} KB/s`;
    return `${(kb / 1024).toFixed(2)} MB/s`;
  }

  function getCpuColor(percent: number): string {
    if (percent >= 85) return 'var(--island-red, #ef4444)';
    if (percent >= 60) return 'var(--island-amber, #f59e0b)';
    return 'var(--island-green, #10b981)';
  }

  function getMemColor(percent: number): string {
    if (percent >= 90) return 'var(--island-red, #ef4444)';
    if (percent >= 75) return 'var(--island-amber, #f59e0b)';
    return 'var(--island-accent, #3b82f6)';
  }

  async function handleRefresh(e: MouseEvent) {
    e.stopPropagation();
    isRefreshing = true;
    await islandState.refreshTelemetry();
    setTimeout(() => {
      isRefreshing = false;
    }, 400);
  }
</script>

<div class="system-container">
  <IslandHeader title="System Performance" />

  {#if telemetry}
    <div class="metrics-grid">
      <!-- CPU Metric Card -->
      <div class="metric-card">
        <div class="card-header">
          <div class="card-title-group">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="4" y="4" width="16" height="16" rx="2"></rect>
              <rect x="9" y="9" width="6" height="6"></rect>
              <path d="M9 1v3M15 1v3M9 20v3M15 20v3M20 9h3M20 14h3M1 9h3M1 14h3"></path>
            </svg>
            <span class="card-label">CPU Usage</span>
          </div>
          <span class="metric-val" style="color: {getCpuColor(telemetry.cpu_percent)}">
            {telemetry.cpu_percent.toFixed(1)}%
          </span>
        </div>

        <div class="progress-track">
          <div
            class="progress-fill"
            style="width: {Math.min(100, Math.max(2, telemetry.cpu_percent))}%; background-color: {getCpuColor(telemetry.cpu_percent)}"
          ></div>
        </div>

        <div class="card-footer">
          <span>{telemetry.core_count} Logical Cores</span>
        </div>
      </div>

      <!-- Memory (RAM) Metric Card -->
      <div class="metric-card">
        <div class="card-header">
          <div class="card-title-group">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M6 19v-3M10 19v-3M14 19v-3M18 19v-3M6 8V5M10 8V5M14 8V5M18 8V5"></path>
              <rect x="2" y="8" width="20" height="8" rx="1"></rect>
            </svg>
            <span class="card-label">RAM Usage</span>
          </div>
          <span class="metric-val" style="color: {getMemColor(telemetry.mem_percent)}">
            {telemetry.mem_percent.toFixed(1)}%
          </span>
        </div>

        <div class="progress-track">
          <div
            class="progress-fill"
            style="width: {Math.min(100, Math.max(2, telemetry.mem_percent))}%; background-color: {getMemColor(telemetry.mem_percent)}"
          ></div>
        </div>

        <div class="card-footer">
          <span>{(telemetry.mem_used_mb / 1024).toFixed(1)} / {(telemetry.mem_total_mb / 1024).toFixed(1)} GB</span>
        </div>
      </div>

      <!-- Network Throughput Card -->
      <div class="metric-card">
        <div class="card-header">
          <div class="card-title-group">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M5 12.55a11 11 0 0 1 14.08 0"></path>
              <path d="M1.42 9a16 16 0 0 1 21.16 0"></path>
              <path d="M8.53 16.11a6 6 0 0 1 6.95 0"></path>
              <line x1="12" y1="20" x2="12.01" y2="20"></line>
            </svg>
            <span class="card-label">Network</span>
          </div>
        </div>

        <div class="net-stats">
          <div class="net-row">
            <span class="net-tag dl">↓ DL</span>
            <span class="net-val">{formatSpeed(telemetry.net_rx_speed_kb)}</span>
          </div>
          <div class="net-row">
            <span class="net-tag ul">↑ UL</span>
            <span class="net-val">{formatSpeed(telemetry.net_tx_speed_kb)}</span>
          </div>
        </div>
      </div>

      <!-- Battery / Power Card -->
      <div class="metric-card">
        <div class="card-header">
          <div class="card-title-group">
            {#if telemetry.battery}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <rect x="2" y="7" width="16" height="10" rx="2" ry="2"></rect>
                <line x1="22" y1="11" x2="22" y2="13"></line>
              </svg>
              <span class="card-label">Battery</span>
            {:else}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M18 6L6 18M6 6l12 12"></path>
              </svg>
              <span class="card-label">Power</span>
            {/if}
          </div>

          {#if telemetry.battery}
            <span class="metric-val battery-val">
              {telemetry.battery.percent}%
              {#if telemetry.battery.is_charging}
                <span class="charging-bolt" title="Charging">⚡</span>
              {/if}
            </span>
          {:else}
            <span class="metric-val ac-val">AC Power</span>
          {/if}
        </div>

        {#if telemetry.battery}
          <div class="progress-track">
            <div
              class="progress-fill"
              style="width: {telemetry.battery.percent}%; background-color: {telemetry.battery.is_charging ? 'var(--island-green, #10b981)' : (telemetry.battery.percent <= 20 ? 'var(--island-red, #ef4444)' : 'var(--island-accent, #3b82f6)')}"
            ></div>
          </div>
          <div class="card-footer">
            <span>Status: {telemetry.battery.status}</span>
          </div>
        {:else}
          <div class="ac-status-box">
            <span>Plugged In (Desktop / Wall)</span>
          </div>
        {/if}
      </div>
    </div>
  {:else}
    <div class="loading-box">
      <span class="spinner"></span>
      <span>Sampling Linux hardware telemetry...</span>
    </div>
  {/if}

  <!-- Bottom Navigation Row -->
  <div class="system-bottom-bar">
    <button
      type="button"
      class="refresh-btn"
      class:spinning={isRefreshing}
      onclick={handleRefresh}
      title="Sample system telemetry now"
      aria-label="Refresh telemetry"
    >
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"></path>
      </svg>
      Refresh
    </button>
  </div>
</div>

<style>
  .system-container {
    display: flex;
    flex-direction: column;
    gap: 12px;
    width: 100%;
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }

  .metric-card {
    background: var(--island-card-bg, rgba(255, 255, 255, 0.05));
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    border-radius: 12px;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .card-title-group {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--island-text-secondary, #9ca3af);
    font-size: 11px;
    font-weight: 500;
  }

  .metric-val {
    font-size: 12px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .progress-track {
    width: 100%;
    height: 5px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 9999px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    border-radius: 9999px;
    transition: width 0.3s cubic-bezier(0.16, 1, 0.3, 1), background-color 0.2s ease;
  }

  .card-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 10px;
    color: var(--island-text-muted, #6b7280);
    font-variant-numeric: tabular-nums;
  }

  /* Network Stats */
  .net-stats {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 2px;
  }

  .net-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .net-tag {
    font-size: 9px;
    font-weight: 600;
    padding: 1px 4px;
    border-radius: 4px;
  }

  .net-tag.dl {
    background: rgba(16, 185, 129, 0.2);
    color: #34d399;
  }

  .net-tag.ul {
    background: rgba(59, 130, 246, 0.2);
    color: #60a5fa;
  }

  .net-val {
    font-size: 11px;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
    font-variant-numeric: tabular-nums;
  }

  /* Battery & Power */
  .charging-bolt {
    font-size: 10px;
    color: #fbbf24;
    margin-left: 2px;
  }

  .ac-val {
    color: #34d399;
    font-size: 11px;
  }

  .ac-status-box {
    font-size: 10px;
    color: var(--island-text-secondary, #9ca3af);
    margin-top: 4px;
  }

  /* Bottom bar */
  .system-bottom-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 4px;
  }

  .back-btn {
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

  .back-btn:hover {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.08);
  }

  .refresh-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: var(--island-text-secondary, #9ca3af);
    font-size: 10px;
    font-weight: 500;
    padding: 3px 8px;
    border-radius: 6px;
    cursor: pointer;
    font-family: inherit;
    transition: background 0.15s ease, color 0.15s ease;
  }

  .refresh-btn:hover {
    background: rgba(255, 255, 255, 0.16);
    color: #ffffff;
  }

  .refresh-btn.spinning svg {
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .loading-box {
    display: flex;
    align-items: center;
    gap: 8px;
    justify-content: center;
    padding: 32px 0;
    font-size: 12px;
    color: var(--island-text-secondary, #9ca3af);
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.2);
    border-top-color: var(--island-accent, #3b82f6);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
</style>
