<script lang="ts">
  import IslandHeader from './IslandHeader.svelte';
  import { islandState } from '../state/island.svelte';

  const agent = $derived(islandState.agent);

  function handleApprove(e: MouseEvent) {
    e.stopPropagation();
    islandState.approveAgentReview();
  }

  function handleReject(e: MouseEvent) {
    e.stopPropagation();
    islandState.rejectAgentReview();
  }

  function handleBack(e: MouseEvent) {
    e.stopPropagation();
    islandState.openOverview();
  }
</script>

<div class="agent-preview-wrapper">
  <IslandHeader title="AI Agent Companion" />

  <!-- Main Agent Status Card -->
  <div class="agent-card" class:review={agent.state === 'review'} class:working={agent.state === 'working'} class:thinking={agent.state === 'thinking'}>
    <div
      class="agent-avatar"
      class:avatar-thinking={agent.state === 'thinking'}
      class:avatar-working={agent.state === 'working'}
      class:avatar-review={agent.state === 'review'}
      class:avatar-idle={agent.state === 'idle'}
    >
      {#if agent.state === 'thinking'}
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10"></circle>
          <path d="M12 6v6l4 2"></path>
        </svg>
      {:else if agent.state === 'working'}
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"></path>
        </svg>
      {:else if agent.state === 'review'}
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
          <polyline points="14 2 14 8 20 8"></polyline>
          <path d="m9 15 2 2 4-4"></path>
        </svg>
      {:else}
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 2a2 2 0 0 1 2 2v2a2 2 0 0 1-2 2 2 2 0 0 1-2-2V4a2 2 0 0 1 2-2z"></path>
          <rect x="4" y="8" width="16" height="12" rx="4"></rect>
          <circle cx="9" cy="14" r="1.5"></circle>
          <circle cx="15" cy="14" r="1.5"></circle>
        </svg>
      {/if}
    </div>

    <div class="agent-info">
      <div class="agent-title-row">
        <span class="agent-name">{agent.name}</span>
        <span class="agent-state-badge state-{agent.state}">{agent.state}</span>
      </div>

      {#if agent.task}
        <span class="agent-task-title">{agent.task}</span>
      {/if}

      <p class="agent-status-text">{agent.message}</p>

      {#if agent.progress !== null && agent.progress !== undefined}
        <div class="progress-bar-container">
          <div class="progress-bar-fill" style="width: {agent.progress}%"></div>
          <span class="progress-label">{agent.progress}%</span>
        </div>
      {/if}

      {#if agent.changed_files && agent.changed_files.length > 0}
        <div class="files-container">
          <span class="files-header">Files modified ({agent.changed_files.length}):</span>
          <div class="files-list">
            {#each agent.changed_files as file}
              <span class="file-chip" title={file}>
                {file.split('/').pop()}
              </span>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Review Action Buttons -->
  {#if agent.state === 'review'}
    <div class="review-actions-bar">
      <button type="button" class="btn btn-approve" onclick={handleApprove}>
        ✓ Approve & Apply Changes
      </button>
      <button type="button" class="btn btn-reject" onclick={handleReject}>
        ✕ Discard
      </button>
    </div>
  {/if}

  <div class="bottom-bar">
    <button type="button" class="back-link" onclick={handleBack}>
      ← Back to Overview
    </button>
    <span class="ipc-hint">IPC: dynamic-island.sock</span>
  </div>
</div>

<style>
  .agent-preview-wrapper {
    width: 100%;
    max-height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.15) transparent;
    display: flex;
    flex-direction: column;
    gap: 10px;
    box-sizing: border-box;
    animation: fadeIn 0.2s ease-out;
  }

  .agent-preview-wrapper::-webkit-scrollbar {
    width: 4px;
  }

  .agent-preview-wrapper::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.15);
    border-radius: 4px;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .agent-card {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    background: var(--island-card-bg, rgba(255, 255, 255, 0.05));
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    border-radius: 12px;
    padding: 12px;
    transition: border-color 0.2s ease, background 0.2s ease;
  }

  .agent-card.thinking {
    border-color: rgba(168, 85, 247, 0.3);
    background: rgba(168, 85, 247, 0.06);
  }

  .agent-card.working {
    border-color: rgba(59, 130, 246, 0.3);
    background: rgba(59, 130, 246, 0.06);
  }

  .agent-card.review {
    border-color: rgba(245, 158, 11, 0.35);
    background: rgba(245, 158, 11, 0.08);
  }

  .agent-avatar {
    width: 38px;
    height: 38px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #ffffff;
    flex-shrink: 0;
  }

  .avatar-idle {
    background: linear-gradient(135deg, #4b5563 0%, #374151 100%);
  }

  .avatar-thinking {
    background: linear-gradient(135deg, #a855f7 0%, #7c3aed 100%);
    animation: pulseThinking 1.5s infinite alternate;
  }

  .avatar-working {
    background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  }

  .avatar-review {
    background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
    box-shadow: 0 0 10px rgba(245, 158, 11, 0.4);
  }

  @keyframes pulseThinking {
    0% { transform: scale(1); filter: brightness(1); }
    100% { transform: scale(1.08); filter: brightness(1.3); }
  }

  .agent-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
    overflow: hidden;
  }

  .agent-title-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .agent-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
  }

  .agent-state-badge {
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 2px 7px;
    border-radius: 9999px;
  }

  .state-idle {
    background: rgba(255, 255, 255, 0.08);
    color: var(--island-text-secondary, #9ca3af);
  }

  .state-thinking {
    background: rgba(168, 85, 247, 0.2);
    color: #c084fc;
    border: 1px solid rgba(168, 85, 247, 0.35);
  }

  .state-working {
    background: rgba(59, 130, 246, 0.2);
    color: #93c5fd;
    border: 1px solid rgba(59, 130, 246, 0.35);
  }

  .state-review {
    background: rgba(245, 158, 11, 0.25);
    color: #fcd34d;
    border: 1px solid rgba(245, 158, 11, 0.45);
  }

  .agent-task-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--island-accent, #3b82f6);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .agent-status-text {
    margin: 0;
    font-size: 11.5px;
    color: var(--island-text-secondary, #9ca3af);
    line-height: 1.35;
  }

  .progress-bar-container {
    width: 100%;
    height: 6px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 9999px;
    position: relative;
    overflow: hidden;
    margin-top: 4px;
    display: flex;
    align-items: center;
  }

  .progress-bar-fill {
    height: 100%;
    background: linear-gradient(90deg, #3b82f6, #60a5fa);
    border-radius: 9999px;
    transition: width 0.3s ease;
  }

  .progress-label {
    position: absolute;
    right: 4px;
    font-size: 8px;
    font-weight: 700;
    color: #ffffff;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.8);
  }

  /* Changed files */
  .files-container {
    margin-top: 4px;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .files-header {
    font-size: 10px;
    color: var(--island-text-muted, #6b7280);
  }

  .files-list {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .file-chip {
    font-size: 9.5px;
    padding: 1px 5px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.08);
    color: #d1d5db;
    font-family: monospace;
  }

  /* Review Actions Bar */
  .review-actions-bar {
    display: flex;
    gap: 8px;
  }

  .btn {
    padding: 7px 12px;
    border-radius: 8px;
    font-size: 11px;
    font-weight: 600;
    border: none;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    transition: background 0.15s ease, transform 0.1s ease;
  }

  .btn:active {
    transform: scale(0.98);
  }

  .btn-approve {
    background: #10b981;
    color: #ffffff;
    flex: 1;
  }

  .btn-approve:hover {
    background: #059669;
  }

  .btn-reject {
    background: rgba(239, 68, 68, 0.2);
    color: #fca5a5;
    border: 1px solid rgba(239, 68, 68, 0.3);
    flex: 0 0 80px;
  }

  .btn-reject:hover {
    background: rgba(239, 68, 68, 0.35);
  }

  /* Bottom bar */
  .bottom-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 2px;
  }

  .back-link {
    background: none;
    border: none;
    color: var(--island-text-secondary, #9ca3af);
    font-size: 11px;
    cursor: pointer;
    font-family: inherit;
    padding: 2px 6px;
    border-radius: 4px;
    transition: color 0.15s ease;
  }

  .back-link:hover {
    color: #ffffff;
  }

  .ipc-hint {
    font-size: 9px;
    color: var(--island-text-muted, #6b7280);
    font-family: monospace;
  }
</style>