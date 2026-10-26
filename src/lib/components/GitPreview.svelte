<script lang="ts">
  import IslandHeader from './IslandHeader.svelte';
  import { islandState } from '../state/island.svelte';

  const git = $derived(islandState.git);
  let isRefreshing = $state(false);

  async function handleRefresh(e: MouseEvent) {
    e.stopPropagation();
    isRefreshing = true;
    await islandState.refreshGitStatus();
    setTimeout(() => {
      isRefreshing = false;
    }, 400);
  }

  function handleBack(e: MouseEvent) {
    e.stopPropagation();
    islandState.openOverview();
  }
</script>

<div class="git-preview-wrapper">
  <IslandHeader title="Git Repository" />

  {#if !git.is_repo}
    <div class="empty-repo-card">
      <div class="empty-icon">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10"></circle>
          <line x1="12" y1="8" x2="12" y2="12"></line>
          <line x1="12" y1="16" x2="12.01" y2="16"></line>
        </svg>
      </div>
      <h3 class="empty-title">No Active Git Repository</h3>
      <p class="empty-desc">
        Dynamic Island will automatically track Git status when you open a Git workspace.
      </p>
      <button type="button" class="btn btn-refresh" onclick={handleRefresh} disabled={isRefreshing}>
        <svg class:spinning={isRefreshing} width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="23 4 23 10 17 10"></polyline>
          <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
        </svg>
        {isRefreshing ? 'Checking...' : 'Check for Repository'}
      </button>
    </div>
  {:else}
    <!-- Main Repo Card -->
    <div class="repo-card">
      <div class="repo-top">
        <div class="repo-icon">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="6" y1="3" x2="6" y2="15"></line>
            <circle cx="18" cy="6" r="3"></circle>
            <circle cx="6" cy="18" r="3"></circle>
            <path d="M18 9a9 9 0 0 1-9 9"></path>
          </svg>
        </div>
        <div class="repo-info">
          <div class="repo-header-row">
            <span class="repo-name">{git.repo_name || 'Repository'}</span>
            <span class="branch-badge">
              <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <line x1="6" y1="3" x2="6" y2="15"></line>
                <circle cx="18" cy="6" r="3"></circle>
                <circle cx="6" cy="18" r="3"></circle>
                <path d="M18 9a9 9 0 0 1-9 9"></path>
              </svg>
              {git.branch || 'detached'}
            </span>
          </div>
          {#if git.repo_path}
            <div class="repo-path-row">
              <span class="repo-path" title={git.repo_path}>{git.repo_path}</span>
              {#if islandState.editor?.editor}
                <span class="editor-sync-badge" title="Tracked from active {islandState.editor.editor} workspace">
                  ⚡ {islandState.editor.editor}
                </span>
              {/if}
            </div>
          {/if}
        </div>
        <button
          type="button"
          class="btn-icon-action"
          onclick={handleRefresh}
          title="Refresh Git status"
          aria-label="Refresh Git status"
          disabled={isRefreshing}
        >
          <svg class:spinning={isRefreshing} width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 4 23 10 17 10"></polyline>
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
          </svg>
        </button>
      </div>

      <!-- Sync Indicators -->
      <div class="sync-row">
        {#if git.ahead > 0 || git.behind > 0}
          <div class="sync-pill" class:ahead={git.ahead > 0}>
            <span>↑ {git.ahead} to push</span>
          </div>
          <div class="sync-pill" class:behind={git.behind > 0}>
            <span>↓ {git.behind} to pull</span>
          </div>
        {:else}
          <div class="sync-pill synced">
            <span>✓ Up to date with remote</span>
          </div>
        {/if}
      </div>
    </div>

    <!-- Working Tree Metrics Grid -->
    <div class="status-grid">
      <div class="status-metric staged" class:has-items={git.staged_count > 0}>
        <span class="metric-count">{git.staged_count}</span>
        <span class="metric-label">Staged</span>
      </div>
      <div class="status-metric unstaged" class:has-items={git.unstaged_count > 0}>
        <span class="metric-count">{git.unstaged_count}</span>
        <span class="metric-label">Modified</span>
      </div>
      <div class="status-metric untracked" class:has-items={git.untracked_count > 0}>
        <span class="metric-count">{git.untracked_count}</span>
        <span class="metric-label">Untracked</span>
      </div>
    </div>

    <!-- Changed Files Section -->
    {#if git.changed_files.length > 0}
      <div class="files-card">
        <div class="files-header">
          <span>Changed Files ({git.changed_files.length})</span>
        </div>
        <div class="files-list">
          {#each git.changed_files.slice(0, 8) as file}
            <div class="file-item">
              <span
                class="status-tag"
                class:status-staged={file.is_staged}
                class:status-mod={!file.is_staged && file.status !== '??'}
                class:status-untracked={file.status === '??'}
              >
                {file.status}
              </span>
              <span class="file-name" title={file.path}>
                {file.path}
              </span>
            </div>
          {/each}
          {#if git.changed_files.length > 8}
            <div class="files-more">
              +{git.changed_files.length - 8} more files...
            </div>
          {/if}
        </div>
      </div>
    {:else}
      <div class="clean-tree-card">
        <span class="clean-icon">✓</span>
        <span class="clean-text">Working tree clean — nothing to commit</span>
      </div>
    {/if}

    <!-- Recent Commit Info -->
    {#if git.head_commit}
      <div class="commit-card">
        <div class="commit-header">
          <span class="commit-label">Latest Commit</span>
          <span class="commit-sha">#{git.head_commit.short_hash}</span>
        </div>
        <p class="commit-message" title={git.head_commit.message}>
          {git.head_commit.message}
        </p>
        <div class="commit-meta">
          <span>{git.head_commit.author}</span>
          <span>•</span>
          <span>{git.head_commit.relative_time}</span>
        </div>
      </div>
    {/if}
  {/if}

  <div class="bottom-bar">
    <button type="button" class="back-link" onclick={handleBack}>
      ← Back to Overview
    </button>
    <span class="ipc-hint">Git Phase 1 HUD</span>
  </div>
</div>

<style>
  .git-preview-wrapper {
    width: 100%;
    max-height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: thin;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-bottom: 4px;
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  /* Repo Card */
  .repo-card {
    background: var(--island-card-bg, rgba(255, 255, 255, 0.05));
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    border-radius: 12px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .repo-top {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .repo-icon {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    background: rgba(59, 130, 246, 0.15);
    color: var(--island-accent, #3b82f6);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .repo-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .repo-header-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .repo-name {
    font-size: 13px;
    font-weight: 700;
    color: var(--island-text-primary, #ffffff);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .branch-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    font-weight: 600;
    color: var(--island-accent, #3b82f6);
    background: rgba(59, 130, 246, 0.12);
    border: 1px solid rgba(59, 130, 246, 0.25);
    padding: 1px 7px;
    border-radius: 9999px;
    max-width: 140px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .repo-path-row {
    display: flex;
    align-items: center;
    gap: 6px;
    overflow: hidden;
  }

  .editor-sync-badge {
    font-size: 9px;
    font-weight: 600;
    color: var(--island-cyan, #06b6d4);
    background: rgba(6, 182, 212, 0.12);
    border: 1px solid rgba(6, 182, 212, 0.25);
    padding: 0px 5px;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .repo-path {
    font-size: 10px;
    color: var(--island-text-muted, #6b7280);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .btn-icon-action {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.1));
    background: var(--island-card-bg, rgba(255, 255, 255, 0.08));
    color: var(--island-text-secondary, #9ca3af);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: background 0.15s ease, color 0.15s ease;
  }

  .btn-icon-action:hover {
    background: var(--island-card-hover, rgba(255, 255, 255, 0.16));
    color: var(--island-text-primary, #ffffff);
  }

  .spinning {
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  /* Sync Row */
  .sync-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .sync-pill {
    font-size: 10.5px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--island-text-secondary, #9ca3af);
  }

  .sync-pill.ahead {
    background: rgba(245, 158, 11, 0.16);
    color: var(--island-amber, #f59e0b);
    border: 1px solid rgba(245, 158, 11, 0.25);
  }

  .sync-pill.behind {
    background: rgba(239, 68, 68, 0.16);
    color: var(--island-red, #ef4444);
    border: 1px solid rgba(239, 68, 68, 0.25);
  }

  .sync-pill.synced {
    color: var(--island-green, #10b981);
    background: rgba(16, 185, 129, 0.1);
  }

  /* Status Metric Grid */
  .status-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
  }

  .status-metric {
    background: var(--island-card-bg, rgba(255, 255, 255, 0.05));
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    border-radius: 10px;
    padding: 8px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    text-align: center;
  }

  .status-metric.staged.has-items {
    border-color: rgba(16, 185, 129, 0.35);
    background: rgba(16, 185, 129, 0.08);
  }

  .status-metric.unstaged.has-items {
    border-color: rgba(245, 158, 11, 0.35);
    background: rgba(245, 158, 11, 0.08);
  }

  .status-metric.untracked.has-items {
    border-color: rgba(59, 130, 246, 0.35);
    background: rgba(59, 130, 246, 0.08);
  }

  .metric-count {
    font-size: 16px;
    font-weight: 700;
    color: var(--island-text-primary, #ffffff);
  }

  .status-metric.staged .metric-count {
    color: var(--island-green, #10b981);
  }

  .status-metric.unstaged .metric-count {
    color: var(--island-amber, #f59e0b);
  }

  .status-metric.untracked .metric-count {
    color: var(--island-accent, #3b82f6);
  }

  .metric-label {
    font-size: 10px;
    font-weight: 500;
    color: var(--island-text-muted, #6b7280);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  /* Changed Files List */
  .files-card {
    background: var(--island-card-bg, rgba(255, 255, 255, 0.05));
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    border-radius: 10px;
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .files-header {
    font-size: 11px;
    font-weight: 600;
    color: var(--island-text-secondary, #9ca3af);
  }

  .files-list {
    display: flex;
    flex-direction: column;
    gap: 3px;
    max-height: 120px;
    overflow-y: auto;
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    padding: 2px 0;
  }

  .status-tag {
    font-size: 9px;
    font-weight: 700;
    padding: 1px 4px;
    border-radius: 4px;
    font-family: monospace;
    min-width: 14px;
    text-align: center;
  }

  .status-tag.status-staged {
    background: rgba(16, 185, 129, 0.2);
    color: var(--island-green, #10b981);
  }

  .status-tag.status-mod {
    background: rgba(245, 158, 11, 0.2);
    color: var(--island-amber, #f59e0b);
  }

  .status-tag.status-untracked {
    background: rgba(107, 114, 128, 0.2);
    color: var(--island-text-secondary, #9ca3af);
  }

  .file-name {
    color: var(--island-text-primary, #ffffff);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    direction: rtl;
    text-align: left;
  }

  .files-more {
    font-size: 10px;
    color: var(--island-text-muted, #6b7280);
    font-style: italic;
    padding-top: 2px;
  }

  .clean-tree-card {
    display: flex;
    align-items: center;
    gap: 8px;
    background: rgba(16, 185, 129, 0.08);
    border: 1px solid rgba(16, 185, 129, 0.2);
    border-radius: 10px;
    padding: 8px 12px;
    color: var(--island-green, #10b981);
  }

  .clean-icon {
    font-weight: 700;
    font-size: 14px;
  }

  .clean-text {
    font-size: 11.5px;
    font-weight: 500;
  }

  /* Latest Commit Card */
  .commit-card {
    background: var(--island-card-bg, rgba(255, 255, 255, 0.04));
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.07));
    border-radius: 10px;
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .commit-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .commit-label {
    font-size: 10px;
    font-weight: 600;
    color: var(--island-text-muted, #6b7280);
    text-transform: uppercase;
  }

  .commit-sha {
    font-size: 10px;
    font-family: monospace;
    font-weight: 600;
    color: var(--island-accent, #3b82f6);
    background: rgba(59, 130, 246, 0.12);
    padding: 1px 4px;
    border-radius: 4px;
  }

  .commit-message {
    margin: 0;
    font-size: 11.5px;
    color: var(--island-text-primary, #ffffff);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .commit-meta {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 10px;
    color: var(--island-text-secondary, #9ca3af);
  }

  /* Empty Repo Card */
  .empty-repo-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 24px 16px;
    text-align: center;
    gap: 8px;
    background: var(--island-card-bg, rgba(255, 255, 255, 0.05));
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    border-radius: 12px;
  }

  .empty-icon {
    color: var(--island-text-muted, #6b7280);
  }

  .empty-title {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
  }

  .empty-desc {
    margin: 0;
    font-size: 11px;
    color: var(--island-text-secondary, #9ca3af);
    max-width: 260px;
    line-height: 1.4;
  }

  .btn {
    border-radius: 8px;
    padding: 6px 12px;
    font-size: 11.5px;
    font-weight: 600;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    transition: all 0.15s ease;
  }

  .btn-refresh {
    background: var(--island-card-hover, rgba(255, 255, 255, 0.12));
    border: 1px solid var(--island-border, rgba(255, 255, 255, 0.18));
    color: var(--island-text-primary, #ffffff);
    margin-top: 4px;
  }

  .btn-refresh:hover:not(:disabled) {
    background: var(--island-accent, #3b82f6);
    border-color: var(--island-accent, #3b82f6);
  }

  /* Bottom Bar */
  .bottom-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 4px;
    border-top: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.06));
    margin-top: 2px;
  }

  .back-link {
    font-size: 11px;
    font-weight: 500;
    color: var(--island-text-secondary, #9ca3af);
    background: none;
    border: none;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 4px;
    transition: color 0.15s ease;
  }

  .back-link:hover {
    color: var(--island-accent, #3b82f6);
  }

  .ipc-hint {
    font-size: 10px;
    color: var(--island-text-muted, #6b7280);
    font-family: monospace;
  }
</style>
