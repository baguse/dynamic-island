<script lang="ts">
  import IslandHeader from './IslandHeader.svelte';
  import { islandState } from '../state/island.svelte';
  import type { PortInfo } from '../types';

  let selectedCategory = $state<'all' | 'web' | 'database' | 'service'>('all');
  let searchQuery = $state('');
  let isRefreshing = $state(false);
  let killingPid = $state<number | null>(null);

  const ports = $derived(islandState.ports);

  const filteredPorts = $derived.by(() => {
    return ports.filter((p) => {
      // Category filter
      if (selectedCategory !== 'all' && p.category !== selectedCategory) {
        return false;
      }
      // Search filter
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase().trim();
        const matchesPort = p.port.toString().includes(q);
        const matchesProc = p.process?.toLowerCase().includes(q);
        const matchesContainer = p.container?.toLowerCase().includes(q);
        return matchesPort || matchesProc || matchesContainer;
      }
      return true;
    });
  });

  const webCount = $derived(ports.filter((p) => p.category === 'web').length);
  const dbCount = $derived(ports.filter((p) => p.category === 'database').length);
  const serviceCount = $derived(ports.filter((p) => p.category === 'service').length);

  async function handleRefresh(e: MouseEvent) {
    e.stopPropagation();
    isRefreshing = true;
    await islandState.refreshPorts();
    setTimeout(() => {
      isRefreshing = false;
    }, 350);
  }

  async function handleOpen(port: number) {
    await islandState.openBrowser(`http://localhost:${port}`);
  }

  async function handleKill(pid: number, e: MouseEvent) {
    e.stopPropagation();
    killingPid = pid;
    await islandState.killPortProcess(pid);
    killingPid = null;
  }

  function getCategoryColor(cat: string): string {
    switch (cat) {
      case 'web':
        return '#3b82f6';
      case 'database':
        return '#10b981';
      case 'service':
        return '#8b5cf6';
      case 'ide':
        return '#ec4899';
      default:
        return '#64748b';
    }
  }
</script>

<div class="ports-preview-wrapper">
  <IslandHeader title="Localhost & Ports">
    <button
      type="button"
      class="btn-icon-refresh"
      onclick={handleRefresh}
      title="Refresh listening ports"
      aria-label="Refresh listening ports"
      disabled={isRefreshing}
    >
      <svg class:spinning={isRefreshing} width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="23 4 23 10 17 10"></polyline>
        <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
      </svg>
    </button>
  </IslandHeader>

  <!-- Filter & Search Controls -->
  <div class="controls-bar">
    <div class="category-tabs">
      <button
        type="button"
        class="cat-chip"
        class:active={selectedCategory === 'all'}
        onclick={() => (selectedCategory = 'all')}
      >
        All <span class="chip-count">{ports.length}</span>
      </button>
      <button
        type="button"
        class="cat-chip"
        class:active={selectedCategory === 'web'}
        onclick={() => (selectedCategory = 'web')}
      >
        <span class="status-dot web-dot"></span>
        Web <span class="chip-count">{webCount}</span>
      </button>
      <button
        type="button"
        class="cat-chip"
        class:active={selectedCategory === 'database'}
        onclick={() => (selectedCategory = 'database')}
      >
        <span class="status-dot db-dot"></span>
        DB <span class="chip-count">{dbCount}</span>
      </button>
      <button
        type="button"
        class="cat-chip"
        class:active={selectedCategory === 'service'}
        onclick={() => (selectedCategory = 'service')}
      >
        <span class="status-dot service-dot"></span>
        Services <span class="chip-count">{serviceCount}</span>
      </button>
    </div>

    <div class="search-box">
      <svg class="search-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="11" cy="11" r="8"></circle>
        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
      </svg>
      <input
        type="text"
        placeholder="Filter port, process, or container..."
        bind:value={searchQuery}
        class="search-input"
      />
      {#if searchQuery}
        <button type="button" class="btn-clear-search" onclick={() => (searchQuery = '')}>×</button>
      {/if}
    </div>
  </div>

  <!-- Ports List -->
  {#if filteredPorts.length === 0}
    <div class="empty-ports-card">
      <div class="empty-icon">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10"></circle>
          <line x1="12" y1="8" x2="12" y2="12"></line>
          <line x1="12" y1="16" x2="12.01" y2="16"></line>
        </svg>
      </div>
      <span class="empty-title">No matching ports found</span>
      <span class="empty-desc">No active listening ports match your filter or search.</span>
    </div>
  {:else}
    <div class="ports-list">
      {#each filteredPorts as item (item.port)}
        <div class="port-card">
          <div class="port-left">
            <span
              class="port-badge"
              style="border-color: {getCategoryColor(item.category)}55; background: {getCategoryColor(item.category)}18; color: {getCategoryColor(item.category)};"
            >
              :{item.port}
            </span>

            <div class="port-info-col">
              <div class="port-title-row">
                <span class="port-label">
                  {#if item.container}
                    <span class="docker-pill" title="Docker Container">🐳 {item.container}</span>
                  {:else if item.process}
                    <span class="proc-name">{item.process}</span>
                  {:else}
                    <span class="proc-system">System Service</span>
                  {/if}
                </span>

                {#if item.pid}
                  <span class="pid-badge">PID {item.pid}</span>
                {/if}
              </div>

              <span class="port-sub">
                {item.ip}:{item.port} • <span class="cat-tag">{item.category}</span>
              </span>
            </div>
          </div>

          <div class="port-actions">
            {#if item.is_openable}
              <button
                type="button"
                class="btn-act btn-open"
                onclick={() => handleOpen(item.port)}
                title="Open http://localhost:{item.port} in browser"
              >
                <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                  <polyline points="15 3 21 3 21 9"></polyline>
                  <line x1="10" y1="14" x2="21" y2="3"></line>
                </svg>
                <span>Open</span>
              </button>
            {/if}

            {#if item.pid}
              <button
                type="button"
                class="btn-act btn-kill"
                class:loading={killingPid === item.pid}
                onclick={(e) => handleKill(item.pid!, e)}
                title="Kill process {item.process || ''} (PID {item.pid}) to free port :{item.port}"
                disabled={killingPid === item.pid}
              >
                {#if killingPid === item.pid}
                  <span class="btn-spinner"></span>
                  <span>Killing...</span>
                {:else}
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="18" y1="6" x2="6" y2="18"></line>
                    <line x1="6" y1="6" x2="18" y2="18"></line>
                  </svg>
                  <span>Kill</span>
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
  .ports-preview-wrapper {
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

  .category-tabs {
    display: flex;
    gap: 6px;
    margin-bottom: 2px;
    overflow-x: auto;
    scrollbar-width: none;
  }

  .cat-chip {
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

  .cat-chip:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #ffffff;
  }

  .cat-chip.active {
    background: rgba(59, 130, 246, 0.2);
    border-color: #3b82f6;
    color: #ffffff;
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .web-dot {
    background: #3b82f6;
    box-shadow: 0 0 6px rgba(59, 130, 246, 0.6);
  }

  .db-dot {
    background: #10b981;
    box-shadow: 0 0 6px rgba(16, 185, 129, 0.6);
  }

  .service-dot {
    background: #8b5cf6;
    box-shadow: 0 0 6px rgba(139, 92, 246, 0.6);
  }

  .chip-count {
    font-size: 9px;
    padding: 0 4px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.25);
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
    border-color: var(--island-accent, #3b82f6);
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

  .ports-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    overflow-y: auto;
    max-height: 250px;
    padding-right: 2px;
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.15) transparent;
  }

  .port-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.04);
    transition: background 0.15s ease;
    gap: 8px;
  }

  .port-card:hover {
    background: rgba(255, 255, 255, 0.07);
  }

  .port-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
    overflow: hidden;
  }

  .port-badge {
    font-family: monospace;
    font-size: 12px;
    font-weight: 700;
    padding: 3px 6px;
    border-radius: 6px;
    border: 1px solid;
    flex-shrink: 0;
  }

  .port-info-col {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
    overflow: hidden;
  }

  .port-title-row {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    width: 100%;
  }

  .port-label {
    font-size: 12px;
    font-weight: 600;
    color: #ffffff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
    flex-shrink: 1;
  }

  .docker-pill {
    color: #38bdf8;
  }

  .proc-name {
    color: #f3f4f6;
  }

  .proc-system {
    color: #94a3b8;
    font-weight: 400;
  }

  .pid-badge {
    font-size: 9px;
    padding: 1px 4px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.08);
    color: var(--island-text-secondary, #9ca3af);
    font-family: monospace;
  }

  .port-sub {
    font-size: 10px;
    color: var(--island-text-muted, #6b7280);
  }

  .cat-tag {
    text-transform: capitalize;
  }

  .port-actions {
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

  .btn-open {
    background: rgba(59, 130, 246, 0.18);
    border: 1px solid rgba(59, 130, 246, 0.35);
    color: #60a5fa;
  }

  .btn-open:hover {
    background: rgba(59, 130, 246, 0.35);
    color: #ffffff;
  }

  .btn-kill {
    background: rgba(239, 68, 68, 0.14);
    border: 1px solid rgba(239, 68, 68, 0.28);
    color: #f87171;
  }

  .btn-kill:hover:not(:disabled) {
    background: rgba(239, 68, 68, 0.32);
    color: #ffffff;
  }

  .btn-kill:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .empty-ports-card {
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
