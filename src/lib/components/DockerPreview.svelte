<script lang="ts">
  import { tick } from 'svelte';
  import IslandHeader from './IslandHeader.svelte';
  import { islandState } from '../state/island.svelte';
  import type { DockerContainerInfo } from '../types';

  let filterStatus = $state<'all' | 'running' | 'stopped'>('all');
  let searchQuery = $state('');
  let isRefreshing = $state(false);
  let activeActionId = $state<string | null>(null);
  let activeActionType = $state<'restart' | 'stop' | 'start' | null>(null);

  const containers = $derived(islandState.dockerContainers);

  const runningCount = $derived(containers.filter((c) => c.is_running).length);
  const stoppedCount = $derived(containers.filter((c) => !c.is_running).length);

  const filteredContainers = $derived.by(() => {
    return containers.filter((c) => {
      // Filter status
      if (filterStatus === 'running' && !c.is_running) return false;
      if (filterStatus === 'stopped' && c.is_running) return false;

      // Filter search
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase().trim();
        const matchesName = c.name.toLowerCase().includes(q);
        const matchesImage = c.image.toLowerCase().includes(q);
        const matchesPorts = c.ports.toLowerCase().includes(q);
        return matchesName || matchesImage || matchesPorts;
      }
      return true;
    });
  });

  async function handleRefresh(e: MouseEvent) {
    e.stopPropagation();
    isRefreshing = true;
    await islandState.refreshDocker();
    setTimeout(() => {
      isRefreshing = false;
    }, 400);
  }

  async function handleRestart(id: string, e: MouseEvent) {
    e.stopPropagation();
    if (activeActionId) return;
    activeActionId = id;
    activeActionType = 'restart';
    await tick();
    try {
      await islandState.restartDocker(id);
    } finally {
      activeActionId = null;
      activeActionType = null;
    }
  }

  async function handleStop(id: string, e: MouseEvent) {
    e.stopPropagation();
    if (activeActionId) return;
    activeActionId = id;
    activeActionType = 'stop';
    await tick();
    try {
      await islandState.stopDocker(id);
    } finally {
      activeActionId = null;
      activeActionType = null;
    }
  }

  async function handleStart(id: string, e: MouseEvent) {
    e.stopPropagation();
    if (activeActionId) return;
    activeActionId = id;
    activeActionType = 'start';
    await tick();
    try {
      await islandState.startDocker(id);
    } finally {
      activeActionId = null;
      activeActionType = null;
    }
  }

  function formatPorts(portsStr: string): string {
    if (!portsStr) return '';
    const parts = portsStr.split(',').map((p) => p.trim());
    const seenHostPorts = new Set<string>();
    const formatted: string[] = [];

    for (const p of parts) {
      // Matches e.g. 0.0.0.0:8080->80/tcp or :::8080->80/tcp or 0.0.0.0:8080->8080/tcp
      const match = p.match(/(?:0\.0\.0\.0|::|\[::\]):(\d+)->(\d+)/);
      if (match) {
        const host = match[1];
        const container = match[2];
        if (!seenHostPorts.has(host)) {
          seenHostPorts.add(host);
          if (host === container) {
            formatted.push(`:${host}`);
          } else {
            formatted.push(`:${host}->${container}`);
          }
        }
      }
    }

    if (formatted.length === 0) return '';
    if (formatted.length === 1) return formatted[0];
    if (formatted.length === 2) return `${formatted[0]}, ${formatted[1]}`;
    return `${formatted[0]} +${formatted.length - 1}`;
  }
</script>

<div class="docker-preview-wrapper">
  <IslandHeader title="Docker Containers">
    <button
      type="button"
      class="btn-icon-refresh"
      onclick={handleRefresh}
      title="Refresh Docker containers"
      aria-label="Refresh Docker containers"
      disabled={isRefreshing}
    >
      <svg class:spinning={isRefreshing} width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="23 4 23 10 17 10"></polyline>
        <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
      </svg>
    </button>
  </IslandHeader>

  <!-- Controls Bar -->
  <div class="controls-bar">
    <div class="filter-tabs">
      <button
        type="button"
        class="filter-chip"
        class:active={filterStatus === 'all'}
        onclick={() => (filterStatus = 'all')}
      >
        All <span class="chip-count">{containers.length}</span>
      </button>
      <button
        type="button"
        class="filter-chip"
        class:active={filterStatus === 'running'}
        onclick={() => (filterStatus = 'running')}
      >
        <span class="status-dot running-dot"></span>
        Running <span class="chip-count">{runningCount}</span>
      </button>
      <button
        type="button"
        class="filter-chip"
        class:active={filterStatus === 'stopped'}
        onclick={() => (filterStatus = 'stopped')}
      >
        <span class="status-dot stopped-dot"></span>
        Stopped <span class="chip-count">{stoppedCount}</span>
      </button>
    </div>

    <div class="search-box">
      <svg class="search-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="11" cy="11" r="8"></circle>
        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
      </svg>
      <input
        type="text"
        placeholder="Filter container name or image..."
        bind:value={searchQuery}
        class="search-input"
      />
      {#if searchQuery}
        <button type="button" class="btn-clear-search" onclick={() => (searchQuery = '')}>×</button>
      {/if}
    </div>
  </div>

  <!-- Containers List -->
  {#if filteredContainers.length === 0}
    <div class="empty-docker-card">
      <div class="empty-icon">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="2" y="2" width="20" height="20" rx="4"></rect>
          <path d="M7 10h10M7 14h10"></path>
        </svg>
      </div>
      <span class="empty-title">No Containers Found</span>
      <span class="empty-desc">
        {#if containers.length === 0}
          Make sure the Docker daemon is running and containers are registered.
        {:else}
          No containers match your search query.
        {/if}
      </span>
    </div>
  {:else}
    <div class="containers-list">
      {#each filteredContainers as item (item.id)}
        <div class="container-card">
          <div class="container-left">
            <span
              class="container-state-dot"
              class:is-running={item.is_running}
              title={item.state}
            ></span>

            <div class="container-info-col">
              <div class="container-title-row">
                <span class="container-name" title={item.name}>{item.name}</span>
                {#if formatPorts(item.ports)}
                  <span class="ports-tag" title={item.ports}>{formatPorts(item.ports)}</span>
                {/if}
              </div>

              <div class="container-sub-row">
                <span class="image-name" title={item.image}>{item.image}</span>
                <span class="status-text">• {item.status}</span>
              </div>
            </div>
          </div>

          <div class="container-actions">
            {#if item.is_running}
              <!-- Restart Button -->
              <button
                type="button"
                class="btn-act btn-restart"
                class:loading={activeActionId === item.id && activeActionType === 'restart'}
                onclick={(e) => handleRestart(item.id, e)}
                title="Restart container"
                disabled={activeActionId === item.id}
              >
                {#if activeActionId === item.id && activeActionType === 'restart'}
                  <span class="btn-spinner"></span>
                  <span>Restarting...</span>
                {:else}
                  <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="23 4 23 10 17 10"></polyline>
                    <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
                  </svg>
                  <span>Restart</span>
                {/if}
              </button>

              <!-- Stop Button -->
              <button
                type="button"
                class="btn-act btn-stop"
                class:loading={activeActionId === item.id && activeActionType === 'stop'}
                onclick={(e) => handleStop(item.id, e)}
                title="Stop container"
                disabled={activeActionId === item.id}
              >
                {#if activeActionId === item.id && activeActionType === 'stop'}
                  <span class="btn-spinner"></span>
                  <span>Stopping...</span>
                {:else}
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor">
                    <rect x="4" y="4" width="16" height="16" rx="2"></rect>
                  </svg>
                  <span>Stop</span>
                {/if}
              </button>
            {:else}
              <!-- Start Button -->
              <button
                type="button"
                class="btn-act btn-start"
                class:loading={activeActionId === item.id && activeActionType === 'start'}
                onclick={(e) => handleStart(item.id, e)}
                title="Start container"
                disabled={activeActionId === item.id}
              >
                {#if activeActionId === item.id && activeActionType === 'start'}
                  <span class="btn-spinner"></span>
                  <span>Starting...</span>
                {:else}
                  <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor">
                    <polygon points="5 3 19 12 5 21 5 3"></polygon>
                  </svg>
                  <span>Start</span>
                {/if}
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .docker-preview-wrapper {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    gap: 8px;
  }

  .btn-icon-refresh {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    background: var(--island-card-bg, rgba(255, 255, 255, 0.08));
    color: var(--island-text-secondary, #9ca3af);
    cursor: pointer;
    padding: 0;
    transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease, transform 0.1s ease;
  }

  .btn-icon-refresh:hover:not(:disabled) {
    background: var(--island-card-hover, rgba(255, 255, 255, 0.16));
    border-color: var(--island-border-hover, rgba(255, 255, 255, 0.25));
    color: var(--island-text-primary, #ffffff);
  }

  .btn-icon-refresh:active:not(:disabled) {
    transform: scale(0.92);
  }

  .spinning {
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .controls-bar {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 2px 8px 4px 8px;
  }

  .filter-tabs {
    display: flex;
    gap: 6px;
    margin-bottom: 2px;
    overflow-x: auto;
    scrollbar-width: none;
  }

  .filter-chip {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.04);
    font-size: 11px;
    font-weight: 500;
    color: var(--island-text-secondary, #9ca3af);
    cursor: pointer;
    transition: all 0.15s ease;
    white-space: nowrap;
  }

  .filter-chip:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #ffffff;
  }

  .filter-chip.active {
    background: rgba(14, 165, 233, 0.22);
    border-color: #0ea5e9;
    color: #ffffff;
  }

  .chip-count {
    font-size: 9px;
    padding: 0 4px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.25);
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .running-dot {
    background: #10b981;
    box-shadow: 0 0 6px rgba(16, 185, 129, 0.6);
  }

  .stopped-dot {
    background: #64748b;
  }

  .search-box {
    position: relative;
    display: flex;
    align-items: center;
    width: 100%;
  }

  .search-icon {
    position: absolute;
    left: 8px;
    color: var(--island-text-muted, #6b7280);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: 5px 24px 5px 26px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(0, 0, 0, 0.25);
    color: #ffffff;
    font-size: 11px;
    box-sizing: border-box;
    outline: none;
    transition: border-color 0.15s ease;
  }

  .search-input:focus {
    border-color: #0ea5e9;
  }

  .search-input::placeholder {
    color: var(--island-text-muted, #6b7280);
  }

  .btn-clear-search {
    position: absolute;
    right: 6px;
    background: none;
    border: none;
    color: var(--island-text-muted, #6b7280);
    font-size: 14px;
    cursor: pointer;
    line-height: 1;
  }

  .containers-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    overflow-y: auto;
    max-height: 270px;
    padding-right: 2px;
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.15) transparent;
  }

  .container-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 7px 10px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.04);
    transition: background 0.15s ease;
    gap: 8px;
  }

  .container-card:hover {
    background: rgba(255, 255, 255, 0.07);
  }

  .container-left {
    display: flex;
    align-items: center;
    gap: 9px;
    min-width: 0;
    flex: 1;
    overflow: hidden;
  }

  .container-state-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #64748b;
    flex-shrink: 0;
  }

  .container-state-dot.is-running {
    background: #10b981;
    box-shadow: 0 0 8px rgba(16, 185, 129, 0.7);
  }

  .container-info-col {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
    overflow: hidden;
  }

  .container-title-row {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    width: 100%;
  }

  .container-name {
    font-size: 12px;
    font-weight: 600;
    color: #ffffff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
    flex-shrink: 1;
  }

  .ports-tag {
    font-family: monospace;
    font-size: 9px;
    padding: 1px 5px;
    border-radius: 4px;
    background: rgba(14, 165, 233, 0.15);
    color: #38bdf8;
    border: 1px solid rgba(14, 165, 233, 0.3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 110px;
    flex-shrink: 0;
  }

  .container-sub-row {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 10px;
    color: var(--island-text-muted, #6b7280);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
    width: 100%;
  }

  .image-name {
    color: var(--island-text-secondary, #9ca3af);
    font-family: monospace;
    font-size: 9.5px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
    flex-shrink: 1;
  }

  .status-text {
    flex-shrink: 0;
    white-space: nowrap;
  }

  .container-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .btn-act {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 7px;
    border-radius: 5px;
    font-size: 10px;
    font-weight: 600;
    cursor: pointer;
    border: none;
    transition: all 0.15s ease;
    white-space: nowrap;
  }

  .btn-act:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-act.loading {
    opacity: 0.9;
  }

  .btn-spinner {
    width: 9px;
    height: 9px;
    border: 1.5px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    flex-shrink: 0;
  }

  .btn-restart {
    background: rgba(245, 158, 11, 0.14);
    border: 1px solid rgba(245, 158, 11, 0.3);
    color: #fbbf24;
  }

  .btn-restart:hover {
    background: rgba(245, 158, 11, 0.3);
    color: #ffffff;
  }

  .btn-stop {
    background: rgba(239, 68, 68, 0.14);
    border: 1px solid rgba(239, 68, 68, 0.28);
    color: #f87171;
  }

  .btn-stop:hover {
    background: rgba(239, 68, 68, 0.3);
    color: #ffffff;
  }

  .btn-start {
    background: rgba(16, 185, 129, 0.16);
    border: 1px solid rgba(16, 185, 129, 0.3);
    color: #34d399;
  }

  .btn-start:hover {
    background: rgba(16, 185, 129, 0.3);
    color: #ffffff;
  }

  .empty-docker-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 24px 12px;
    text-align: center;
    color: var(--island-text-muted, #6b7280);
    gap: 4px;
  }

  .empty-icon {
    opacity: 0.5;
    margin-bottom: 4px;
  }

  .empty-title {
    font-size: 12px;
    font-weight: 600;
    color: #ffffff;
  }

  .empty-desc {
    font-size: 11px;
  }
</style>
