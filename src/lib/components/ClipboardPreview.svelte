<script lang="ts">
  import { onMount } from 'svelte';
  import IslandHeader from './IslandHeader.svelte';
  import { islandState } from '../state/island.svelte';
  import type { ClipboardItem, SnippetItem } from '../types';

  let activeTab = $state<'history' | 'snippets' | 'add'>('history');
  let searchQuery = $state('');
  let copiedId = $state<string | null>(null);
  let isRefreshing = $state(false);

  onMount(async () => {
    await islandState.refreshClipboard();
  });

  // New snippet form state
  let newTitle = $state('');
  let newContent = $state('');
  let newCategory = $state('General');

  const clipboard = $derived(islandState.clipboard);
  const history = $derived(clipboard.history);
  const snippets = $derived(clipboard.snippets);

  const filteredHistory = $derived.by(() => {
    if (!searchQuery.trim()) return history;
    const q = searchQuery.toLowerCase().trim();
    return history.filter(
      (item) => item.content.toLowerCase().includes(q) || item.preview.toLowerCase().includes(q)
    );
  });

  const filteredSnippets = $derived.by(() => {
    if (!searchQuery.trim()) return snippets;
    const q = searchQuery.toLowerCase().trim();
    return snippets.filter(
      (s) =>
        s.title.toLowerCase().includes(q) ||
        s.content.toLowerCase().includes(q) ||
        s.category.toLowerCase().includes(q)
    );
  });

  async function handleCopy(id: string, text: string) {
    const success = await islandState.copyToClipboard(text);
    if (success) {
      copiedId = id;
      setTimeout(() => {
        if (copiedId === id) copiedId = null;
      }, 1500);
    }
  }

  async function handleDeleteHistory(e: MouseEvent, id: string) {
    e.stopPropagation();
    await islandState.deleteClipboardItem(id);
  }

  async function handleClearHistory(e: MouseEvent) {
    e.stopPropagation();
    await islandState.clearClipboardHistory();
  }

  async function handleDeleteSnippet(e: MouseEvent, id: string) {
    e.stopPropagation();
    await islandState.deleteSnippet(id);
  }

  async function handlePinAsSnippet(e: MouseEvent, item: ClipboardItem) {
    e.stopPropagation();
    const title = item.preview.slice(0, 30);
    await islandState.addSnippet(title, item.content, 'Pinned');
    activeTab = 'snippets';
  }

  async function handleCreateSnippet(e: SubmitEvent) {
    e.preventDefault();
    if (!newContent.trim()) return;
    const title = newTitle.trim() || newContent.trim().slice(0, 24);
    await islandState.addSnippet(title, newContent.trim(), newCategory.trim() || 'General');
    newTitle = '';
    newContent = '';
    newCategory = 'General';
    activeTab = 'snippets';
  }

  function formatTime(timestamp: number): string {
    const now = Math.floor(Date.now() / 1000);
    const diff = Math.max(0, now - timestamp);
    if (diff < 60) return `${diff}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    return `${Math.floor(diff / 3600)}h ago`;
  }
  async function handleRefresh(e: MouseEvent) {
    e.stopPropagation();
    isRefreshing = true;
    await islandState.refreshClipboard();
    setTimeout(() => {
      isRefreshing = false;
    }, 350);
  }
</script>

<div class="clipboard-preview-wrapper">
  <IslandHeader title="Clipboard & Snippets">
    <button
      type="button"
      class="btn-icon-refresh"
      onclick={handleRefresh}
      title="Refresh clipboard history"
      aria-label="Refresh clipboard history"
      disabled={isRefreshing}
    >
      <svg class:spinning={isRefreshing} width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="23 4 23 10 17 10"></polyline>
        <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
      </svg>
    </button>
    {#if activeTab === 'history' && history.length > 0}
      <button
        type="button"
        class="header-action-btn"
        onclick={handleClearHistory}
        title="Clear clipboard history"
      >
        Clear
      </button>
    {/if}
  </IslandHeader>

  <!-- Navigation Tabs -->
  <div class="controls-bar">
    <div class="tab-chips">
      <button
        type="button"
        class="tab-chip"
        class:active={activeTab === 'history'}
        onclick={() => (activeTab = 'history')}
      >
        History <span class="chip-count">{history.length}</span>
      </button>
      <button
        type="button"
        class="tab-chip"
        class:active={activeTab === 'snippets'}
        onclick={() => (activeTab = 'snippets')}
      >
        Snippets <span class="chip-count">{snippets.length}</span>
      </button>
      <button
        type="button"
        class="tab-chip"
        class:active={activeTab === 'add'}
        onclick={() => (activeTab = 'add')}
      >
        + New
      </button>
    </div>

    {#if activeTab !== 'add'}
      <div class="search-box">
        <input
          type="text"
          placeholder="Search..."
          bind:value={searchQuery}
          class="search-input"
        />
        {#if searchQuery}
          <button
            type="button"
            class="clear-search-btn"
            onclick={() => (searchQuery = '')}
          >
            ×
          </button>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Content List Area -->
  <div class="clipboard-content custom-scroll">
    {#if activeTab === 'history'}
      {#if filteredHistory.length === 0}
        <div class="empty-state">
          <span class="empty-icon">📋</span>
          <div class="empty-title">
            {searchQuery ? 'No matching clipboard items' : 'Clipboard history is empty'}
          </div>
          <div class="empty-desc">Copy text anywhere on your desktop to automatically collect it here.</div>
        </div>
      {:else}
        <div class="items-list">
          {#each filteredHistory as item (item.id)}
            <div
              class="clip-card"
              class:copied={copiedId === item.id}
              role="button"
              tabindex="0"
              onclick={() => handleCopy(item.id, item.content)}
              onkeydown={(e) => { if (e.key === 'Enter') handleCopy(item.id, item.content); }}
            >
              <div class="clip-main">
                <div class="clip-preview">{item.preview}</div>
                <div class="clip-meta">
                  <span>{item.char_count} chars</span>
                  {#if item.line_count > 1}
                    <span>• {item.line_count} lines</span>
                  {/if}
                  <span>• {formatTime(item.timestamp)}</span>
                </div>
              </div>

              <div class="clip-actions">
                {#if copiedId === item.id}
                  <span class="copied-badge">Copied!</span>
                {:else}
                  <button
                    type="button"
                    class="action-icon-btn pin-btn"
                    title="Pin as snippet"
                    onclick={(e) => handlePinAsSnippet(e, item)}
                  >
                    📌
                  </button>
                  <button
                    type="button"
                    class="action-icon-btn delete-btn"
                    title="Delete item"
                    onclick={(e) => handleDeleteHistory(e, item.id)}
                  >
                    🗑️
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    {:else if activeTab === 'snippets'}
      {#if filteredSnippets.length === 0}
        <div class="empty-state">
          <span class="empty-icon">📌</span>
          <div class="empty-title">
            {searchQuery ? 'No matching snippets' : 'No pinned snippets yet'}
          </div>
          <div class="empty-desc">Save recurring regexes, tokens, CLI commands, or boilerplate snippets.</div>
        </div>
      {:else}
        <div class="items-list">
          {#each filteredSnippets as snippet (snippet.id)}
            <div
              class="clip-card"
              class:copied={copiedId === snippet.id}
              role="button"
              tabindex="0"
              onclick={() => handleCopy(snippet.id, snippet.content)}
              onkeydown={(e) => { if (e.key === 'Enter') handleCopy(snippet.id, snippet.content); }}
            >
              <div class="clip-main">
                <div class="snippet-title-row">
                  <span class="snippet-title">{snippet.title}</span>
                  <span class="snippet-tag">{snippet.category}</span>
                </div>
                <div class="clip-preview snippet-body">{snippet.content}</div>
              </div>

              <div class="clip-actions">
                {#if copiedId === snippet.id}
                  <span class="copied-badge">Copied!</span>
                {:else}
                  <button
                    type="button"
                    class="action-icon-btn delete-btn"
                    title="Delete snippet"
                    onclick={(e) => handleDeleteSnippet(e, snippet.id)}
                  >
                    🗑️
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    {:else}
      <!-- Add Snippet Form -->
      <form class="add-snippet-form" onsubmit={handleCreateSnippet}>
        <div class="form-row">
          <label for="snippet-title" class="form-label">Title / Label</label>
          <input
            id="snippet-title"
            type="text"
            bind:value={newTitle}
            placeholder="e.g. Docker Prune, Auth Bearer, Regex"
            class="form-input"
          />
        </div>

        <div class="form-row">
          <label for="snippet-category" class="form-label">Category</label>
          <select id="snippet-category" bind:value={newCategory} class="form-select">
            <option value="General">General</option>
            <option value="Commands">Commands & Shell</option>
            <option value="Tokens">Tokens & Keys</option>
            <option value="Regex">Regex</option>
            <option value="Code">Code Snippet</option>
          </select>
        </div>

        <div class="form-row">
          <label for="snippet-content" class="form-label">Content</label>
          <textarea
            id="snippet-content"
            bind:value={newContent}
            rows="4"
            placeholder="Paste or write snippet text here..."
            class="form-textarea custom-scroll"
            required
          ></textarea>
        </div>

        <button type="submit" class="submit-snippet-btn" disabled={!newContent.trim()}>
          Save Pinned Snippet
        </button>
      </form>
    {/if}
  </div>
</div>

<style>
  .clipboard-preview-wrapper {
    display: flex;
    flex-direction: column;
    height: 100%;
    color: var(--text-primary, #ffffff);
    box-sizing: border-box;
  }

  .header-action-btn {
    font-size: 11px;
    padding: 3px 8px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: #cbd5e1;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .header-action-btn:hover {
    background: rgba(239, 68, 68, 0.2);
    border-color: rgba(239, 68, 68, 0.4);
    color: #fca5a5;
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

  .controls-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin: 8px 0 10px 0;
  }

  .tab-chips {
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .tab-chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 9px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 500;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    color: var(--text-secondary, #94a3b8);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .tab-chip:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #f8fafc;
  }

  .tab-chip.active {
    background: rgba(59, 130, 246, 0.15);
    border-color: rgba(59, 130, 246, 0.35);
    color: #60a5fa;
  }

  .chip-count {
    font-size: 9px;
    padding: 1px 4px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.1);
  }

  .search-box {
    position: relative;
    max-width: 130px;
  }

  .search-input {
    width: 100%;
    box-sizing: border-box;
    padding: 4px 20px 4px 8px;
    font-size: 11px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #ffffff;
    outline: none;
  }

  .search-input:focus {
    border-color: #3b82f6;
  }

  .clear-search-btn {
    position: absolute;
    right: 5px;
    top: 50%;
    transform: translateY(-50%);
    background: transparent;
    border: none;
    color: #94a3b8;
    cursor: pointer;
    font-size: 12px;
  }

  .clipboard-content {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow-y: auto;
    max-height: 310px;
  }

  .items-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding-bottom: 8px;
  }

  .clip-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: left;
  }

  .clip-card:hover {
    background: rgba(255, 255, 255, 0.07);
    border-color: rgba(255, 255, 255, 0.12);
  }

  .clip-card.copied {
    border-color: #22c55e;
    background: rgba(34, 197, 94, 0.08);
  }

  .clip-main {
    flex: 1;
    min-width: 0;
  }

  .clip-preview {
    font-size: 12px;
    font-family: monospace;
    color: #f1f5f9;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .clip-meta {
    font-size: 10px;
    color: var(--text-secondary, #94a3b8);
    margin-top: 3px;
  }

  .snippet-title-row {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 2px;
  }

  .snippet-title {
    font-size: 12px;
    font-weight: 600;
    color: #f8fafc;
  }

  .snippet-tag {
    font-size: 9px;
    font-weight: 600;
    padding: 1px 5px;
    border-radius: 4px;
    background: rgba(139, 92, 246, 0.18);
    color: #c084fc;
    border: 1px solid rgba(139, 92, 246, 0.3);
  }

  .snippet-body {
    color: #94a3b8;
    font-size: 11px;
  }

  .clip-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .copied-badge {
    font-size: 10px;
    font-weight: 600;
    color: #4ade80;
    padding: 2px 6px;
    border-radius: 4px;
    background: rgba(34, 197, 94, 0.15);
  }

  .action-icon-btn {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    background: transparent;
    border: 1px solid transparent;
    cursor: pointer;
    font-size: 11px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
  }

  .action-icon-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .delete-btn:hover {
    background: rgba(239, 68, 68, 0.18);
    border-color: rgba(239, 68, 68, 0.3);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 30px 16px;
    text-align: center;
  }

  .empty-icon {
    font-size: 28px;
    margin-bottom: 6px;
  }

  .empty-title {
    font-size: 13px;
    font-weight: 600;
    color: #f1f5f9;
  }

  .empty-desc {
    font-size: 11px;
    color: #94a3b8;
    margin-top: 4px;
    max-width: 260px;
  }

  /* Add Snippet Form */
  .add-snippet-form {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 4px 2px;
  }

  .form-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .form-label {
    font-size: 11px;
    font-weight: 500;
    color: #94a3b8;
  }

  .form-input,
  .form-select,
  .form-textarea {
    width: 100%;
    box-sizing: border-box;
    padding: 6px 10px;
    font-size: 12px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #ffffff;
    outline: none;
    font-family: inherit;
  }

  .form-textarea {
    font-family: monospace;
    resize: none;
  }

  .form-input:focus,
  .form-select:focus,
  .form-textarea:focus {
    border-color: #3b82f6;
  }

  .submit-snippet-btn {
    padding: 8px 14px;
    border-radius: 8px;
    font-size: 12px;
    font-weight: 600;
    background: #3b82f6;
    border: none;
    color: #ffffff;
    cursor: pointer;
    transition: all 0.2s ease;
    margin-top: 4px;
  }

  .submit-snippet-btn:hover:not(:disabled) {
    background: #2563eb;
  }

  .submit-snippet-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .custom-scroll::-webkit-scrollbar {
    width: 4px;
  }

  .custom-scroll::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.12);
    border-radius: 4px;
  }
</style>
