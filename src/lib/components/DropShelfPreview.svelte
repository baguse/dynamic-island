<script lang="ts">
  import { startDrag } from '@crabnebula/tauri-plugin-drag';
  import IslandHeader from './IslandHeader.svelte';
  import { islandState } from '../state/island.svelte';
  import type { ShelfItem } from '../types';

  let isDragOver = $state(false);
  let copiedId = $state<string | null>(null);
  let copiedFileId = $state<string | null>(null);

  const shelf = $derived(islandState.shelf);

  async function handleClearShelf(e: MouseEvent) {
    e.stopPropagation();
    await islandState.clearShelf();
  }

  async function handleRemove(e: MouseEvent, id: string) {
    e.stopPropagation();
    await islandState.removeShelfItem(id);
  }

  async function handleOpenInFolder(e: MouseEvent, path: string | null | undefined) {
    e.stopPropagation();
    if (!path) return;
    await islandState.openShelfItemInFolder(path);
  }

  async function handleCopyFileForNemo(e: MouseEvent, path: string | null | undefined, id: string) {
    e.stopPropagation();
    if (!path) return;
    const ok = await islandState.copyShelfFileToFileManager(path);
    if (ok) {
      copiedFileId = id;
      setTimeout(() => {
        if (copiedFileId === id) copiedFileId = null;
      }, 2000);
    }
  }

  async function handleCopyContent(e: MouseEvent, item: ShelfItem) {
    e.stopPropagation();
    const toCopy = item.path || item.text_content || item.name;
    const success = await islandState.copyToClipboard(toCopy);
    if (success) {
      copiedId = item.id;
      setTimeout(() => {
        if (copiedId === item.id) copiedId = null;
      }, 1500);
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = 'copy';
    }
    isDragOver = true;
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    isDragOver = false;
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragOver = false;

    if (!e.dataTransfer) return;

    // Check files
    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      const paths: string[] = [];
      for (let i = 0; i < e.dataTransfer.files.length; i++) {
        const file = e.dataTransfer.files[i];
        // In Tauri/web, if file has a path property or name
        const p = (file as any).path;
        if (p) {
          paths.push(p);
        }
      }
      if (paths.length > 0) {
        await islandState.addShelfPaths(paths);
        return;
      }
    }

    // Check URL or text
    const text = e.dataTransfer.getData('text/plain') || e.dataTransfer.getData('text/uri-list');
    if (text) {
      await islandState.addShelfText(text);
    }
  }

  async function handleDragStart(e: DragEvent, item: ShelfItem) {
    if (item.path) {
      // Prevent WebKitGTK browser drag (which creates .desktop files) and trigger native OS GTK drag
      e.preventDefault();
      try {
        await startDrag({
          item: [item.path],
          icon: item.item_type === 'image' ? item.path : '',
        });
      } catch (err) {
        console.debug('Native startDrag note:', err);
      }
      return;
    }

    if (!e.dataTransfer) return;
    e.dataTransfer.effectAllowed = 'copyMove';

    if (item.text_content) {
      e.dataTransfer.setData('text/plain', item.text_content);
    } else {
      e.dataTransfer.setData('text/plain', item.name);
    }
  }

  function getItemIcon(item: ShelfItem): string {
    switch (item.item_type) {
      case 'image':
        return '🖼️';
      case 'code':
        return '💻';
      case 'archive':
        return '📦';
      case 'doc':
        return '📄';
      case 'url':
        return '🔗';
      case 'text':
        return '📝';
      default:
        return '📁';
    }
  }
</script>

<div class="shelf-preview-wrapper">
  <IslandHeader title="Desktop Drop Shelf">
    {#if shelf.length > 0}
      <button
        type="button"
        class="header-action-btn"
        onclick={handleClearShelf}
        title="Clear all parked shelf items"
      >
        Clear All
      </button>
    {/if}
  </IslandHeader>

  <!-- Drop Zone Banner -->
  <div
    class="drop-target-zone"
    class:drag-active={isDragOver}
    ondragover={handleDragOver}
    ondragleave={handleDragLeave}
    ondrop={handleDrop}
    role="region"
    aria-label="File drop area"
  >
    <div class="drop-icon-pulse">
      {#if isDragOver}
        📥
      {:else}
        📦
      {/if}
    </div>
    <div class="drop-hint-text">
      <strong>Drop files, images, or URLs here</strong>
      <span>Park temporarily, then drag out to editor or chat</span>
    </div>
  </div>

  <!-- Shelf Items Grid / List -->
  <div class="shelf-content custom-scroll">
    {#if shelf.length === 0}
      <div class="shelf-empty">
        <span class="empty-icon">🗂️</span>
        <div class="empty-title">Shelf is empty</div>
        <div class="empty-desc">
          Drag any file or screenshot from your desktop or Nemo onto the island to park it.
        </div>
      </div>
    {:else}
      <div class="shelf-grid" role="list">
        {#each shelf as item (item.id)}
          <div
            class="shelf-card"
            role="listitem"
            draggable="true"
            ondragstart={(e) => handleDragStart(e, item)}
            title="Drag out to paste path/file into chat or editor"
          >
            <div class="card-left">
              <div class="type-icon-box">
                {getItemIcon(item)}
              </div>
              <div class="item-info">
                <div class="item-name" title={item.name}>{item.name}</div>
                <div class="item-meta">
                  {#if item.size_formatted}
                    <span class="meta-tag">{item.size_formatted}</span>
                  {/if}
                  <span class="type-badge">{item.item_type}</span>
                </div>
              </div>
            </div>

            <div class="card-actions">
              {#if item.path}
                <button
                  type="button"
                  class="shelf-icon-btn file-copy-btn"
                  title="Copy real file to clipboard (Paste anywhere into Nemo with Ctrl+V)"
                  onclick={(e) => handleCopyFileForNemo(e, item.path, item.id)}
                >
                  {copiedFileId === item.id ? '✓' : '📑'}
                </button>
                <button
                  type="button"
                  class="shelf-icon-btn folder-btn"
                  title="Open enclosing folder in Nemo"
                  onclick={(e) => handleOpenInFolder(e, item.path)}
                >
                  📂
                </button>
              {/if}
              <button
                type="button"
                class="shelf-icon-btn copy-btn"
                title={item.path ? 'Copy file path text' : 'Copy content'}
                onclick={(e) => handleCopyContent(e, item)}
              >
                {copiedId === item.id ? '✓' : '📋'}
              </button>
              <button
                type="button"
                class="shelf-icon-btn delete-btn"
                title="Remove from shelf"
                onclick={(e) => handleRemove(e, item.id)}
              >
                ×
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .shelf-preview-wrapper {
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

  .drop-target-zone {
    margin: 8px 0;
    padding: 10px 14px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.03);
    border: 1.5px dashed rgba(255, 255, 255, 0.15);
    display: flex;
    align-items: center;
    gap: 12px;
    transition: all 0.25s ease;
    cursor: pointer;
  }

  .drop-target-zone.drag-active {
    background: rgba(59, 130, 246, 0.14);
    border-color: #3b82f6;
    box-shadow: 0 0 16px rgba(59, 130, 246, 0.25);
    transform: scale(1.01);
  }

  .drop-icon-pulse {
    font-size: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .drop-hint-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 11px;
  }

  .drop-hint-text strong {
    color: #f1f5f9;
    font-weight: 600;
  }

  .drop-hint-text span {
    color: var(--text-secondary, #94a3b8);
    font-size: 10px;
  }

  .shelf-content {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow-y: auto;
    max-height: 250px;
  }

  .shelf-grid {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding-bottom: 6px;
  }

  .shelf-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.07);
    cursor: grab;
    transition: all 0.2s ease;
  }

  .shelf-card:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.14);
    transform: translateY(-1px);
  }

  .shelf-card:active {
    cursor: grabbing;
  }

  .card-left {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    flex: 1;
  }

  .type-icon-box {
    width: 30px;
    height: 30px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.06);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 15px;
    flex-shrink: 0;
  }

  .item-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .item-name {
    font-size: 12px;
    font-weight: 500;
    color: #f8fafc;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 10px;
  }

  .meta-tag {
    color: var(--text-secondary, #94a3b8);
  }

  .type-badge {
    text-transform: uppercase;
    font-size: 8px;
    font-weight: 700;
    letter-spacing: 0.5px;
    padding: 1px 4px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.08);
    color: #cbd5e1;
  }

  .card-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .shelf-icon-btn {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    background: transparent;
    border: 1px solid transparent;
    cursor: pointer;
    font-size: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #cbd5e1;
    transition: all 0.2s ease;
  }

  .shelf-icon-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.15);
    color: #ffffff;
  }

  .delete-btn:hover {
    background: rgba(239, 68, 68, 0.2);
    border-color: rgba(239, 68, 68, 0.35);
    color: #fca5a5;
  }

  .shelf-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 24px 16px;
    text-align: center;
  }

  .empty-icon {
    font-size: 26px;
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
    max-width: 240px;
  }

  .custom-scroll::-webkit-scrollbar {
    width: 4px;
  }

  .custom-scroll::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.12);
    border-radius: 4px;
  }
</style>
