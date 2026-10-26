<script lang="ts">
  import IslandHeader from './IslandHeader.svelte';
  import { islandState } from '../state/island.svelte';

  const editor = $derived(islandState.editor);

  function getLanguageColor(lang?: string | null): string {
    if (!lang) return '#94a3b8';
    const l = lang.toLowerCase();
    switch (l) {
      case 'typescript':
      case 'ts':
        return '#3b82f6';
      case 'javascript':
      case 'js':
        return '#eab308';
      case 'svelte':
        return '#ff3e00';
      case 'rust':
      case 'rs':
        return '#f97316';
      case 'python':
      case 'py':
        return '#06b6d4';
      case 'html':
        return '#ef4444';
      case 'css':
        return '#0ea5e9';
      case 'json':
        return '#10b981';
      case 'markdown':
      case 'md':
        return '#8b5cf6';
      default:
        return '#6366f1';
    }
  }

  function getRelativeTime(timestampSec?: number): string {
    if (!timestampSec) return 'Active now';
    const diff = Math.floor(Date.now() / 1000 - timestampSec);
    if (diff < 5) return 'Just now';
    if (diff < 60) return `${diff}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    return `${Math.floor(diff / 3600)}h ago`;
  }
</script>

<div class="editor-preview-wrapper">
  <IslandHeader title="Editor Presence" />

  {#if !editor || !editor.file_name}
    <div class="empty-editor-card">
      <div class="empty-icon">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="16 18 22 12 16 6"></polyline>
          <polyline points="8 6 2 12 8 18"></polyline>
        </svg>
      </div>
      <h3 class="empty-title">No Active Editor Connected</h3>
      <p class="empty-desc">
        Install the Dynamic Island extension in Cursor or VS Code to broadcast your active file, line position, and workspace status live to the Dynamic Island.
      </p>
    </div>
  {:else}
    <!-- Main Editor Card -->
    <div class="editor-card">
      <div class="editor-top">
        <div class="editor-icon" style="border-color: {getLanguageColor(editor.language)}44;">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="{getLanguageColor(editor.language)}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="16 18 22 12 16 6"></polyline>
            <polyline points="8 6 2 12 8 18"></polyline>
          </svg>
        </div>
        <div class="editor-info">
          <div class="editor-header-row">
            <span class="editor-name">{editor.editor || 'Code Editor'}</span>
            <span class="state-badge state-{editor.state}">{editor.state}</span>
          </div>
          {#if editor.workspace}
            <div class="workspace-row">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
              </svg>
              <span class="workspace-name">{editor.workspace}</span>
            </div>
          {/if}
        </div>
      </div>

      <!-- File Details -->
      <div class="file-details-card">
        <div class="file-main-row">
          <div class="file-title-group">
            <span class="file-name">{editor.file_name}</span>
            {#if editor.language}
              <span class="lang-tag" style="color: {getLanguageColor(editor.language)}; border-color: {getLanguageColor(editor.language)}55;">
                {editor.language}
              </span>
            {/if}
          </div>
          {#if editor.line !== null && editor.line !== undefined}
            <span class="cursor-pos">
              Ln {editor.line}{editor.column ? `, Col ${editor.column}` : ''}
            </span>
          {/if}
        </div>

        {#if editor.file_path && editor.file_path !== editor.file_name}
          <div class="file-path-row" title={editor.file_path}>
            <span class="file-path">{editor.file_path}</span>
          </div>
        {/if}
      </div>

      <!-- Footer / Meta -->
      <div class="editor-footer">
        {#if editor.git_branch}
          <div class="footer-meta branch-meta">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="6" y1="3" x2="6" y2="15"></line>
              <circle cx="18" cy="6" r="3"></circle>
              <circle cx="6" cy="18" r="3"></circle>
              <path d="M18 9a9 9 0 0 1-9 9"></path>
            </svg>
            <span>{editor.git_branch}</span>
          </div>
        {/if}
        <div class="footer-meta time-meta">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <polyline points="12 6 12 12 16 14"></polyline>
          </svg>
          <span>{getRelativeTime(editor.updated_at)}</span>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .editor-preview-wrapper {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    padding: 2px 4px;
    gap: 10px;
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(3px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .empty-editor-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 24px 16px;
    background: var(--island-card-bg, rgba(255, 255, 255, 0.04));
    border: 1px dashed var(--island-card-border, rgba(255, 255, 255, 0.12));
    border-radius: 12px;
    gap: 8px;
  }

  .empty-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: var(--island-card-bg, rgba(255, 255, 255, 0.06));
    color: var(--island-text-secondary, #9ca3af);
    margin-bottom: 4px;
  }

  .empty-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
    margin: 0;
  }

  .empty-desc {
    font-size: 11px;
    line-height: 1.45;
    color: var(--island-text-secondary, #9ca3af);
    margin: 0;
    max-width: 300px;
  }

  .editor-card {
    display: flex;
    flex-direction: column;
    background: var(--island-card-bg, rgba(255, 255, 255, 0.05));
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.1));
    border-radius: 12px;
    padding: 12px;
    gap: 10px;
  }

  .editor-top {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .editor-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 10px;
    background: var(--island-card-bg, rgba(255, 255, 255, 0.06));
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.1));
    flex-shrink: 0;
  }

  .editor-info {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
    gap: 3px;
  }

  .editor-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
  }

  .editor-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .state-badge {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    padding: 2px 7px;
    border-radius: 12px;
  }

  .state-badge.state-editing {
    background: rgba(16, 185, 129, 0.18);
    color: #10b981;
    border: 1px solid rgba(16, 185, 129, 0.35);
  }

  .state-badge.state-debugging {
    background: rgba(245, 158, 11, 0.18);
    color: #f59e0b;
    border: 1px solid rgba(245, 158, 11, 0.35);
  }

  .state-badge.state-viewing,
  .state-badge.state-idle {
    background: rgba(156, 163, 175, 0.15);
    color: #9ca3af;
    border: 1px solid rgba(156, 163, 175, 0.25);
  }

  .workspace-row {
    display: flex;
    align-items: center;
    gap: 5px;
    color: var(--island-text-secondary, #9ca3af);
    font-size: 11px;
  }

  .workspace-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-details-card {
    display: flex;
    flex-direction: column;
    background: var(--island-card-bg, rgba(255, 255, 255, 0.04));
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    border-radius: 8px;
    padding: 8px 10px;
    gap: 4px;
  }

  .file-main-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .file-title-group {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .file-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .lang-tag {
    font-size: 9px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 1px 5px;
    border-radius: 4px;
    border: 1px solid;
    background: rgba(0, 0, 0, 0.2);
  }

  .cursor-pos {
    font-size: 11px;
    font-family: monospace;
    color: var(--island-text-secondary, #9ca3af);
    flex-shrink: 0;
  }

  .file-path-row {
    font-size: 10px;
    color: var(--island-text-secondary, #6b7280);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .editor-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 2px;
    font-size: 11px;
    color: var(--island-text-secondary, #9ca3af);
  }

  .footer-meta {
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .branch-meta {
    color: var(--island-green, #10b981);
  }
</style>
