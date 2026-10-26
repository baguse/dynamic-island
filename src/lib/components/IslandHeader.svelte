<script lang="ts">
  import { islandState } from '../state/island.svelte';

  interface Props {
    title?: string;
  }

  let { title = 'Dynamic Island' }: Props = $props();

  function handleCollapse(e: MouseEvent) {
    e.stopPropagation();
    islandState.collapse();
  }

  function handleBack(e: MouseEvent) {
    e.stopPropagation();
    islandState.openOverview();
  }

  function handleSettings(e: MouseEvent) {
    e.stopPropagation();
    islandState.openSettings();
  }
</script>

<header class="island-header">
  <div class="header-left">
    {#if islandState.mode !== 'expanded'}
      <button
        type="button"
        class="icon-btn back-btn"
        onclick={handleBack}
        title="Back to Overview"
        aria-label="Back to overview panel"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
      </button>
    {/if}
    <span class="status-indicator-dot" class:media-dot={islandState.mode === 'media'} aria-hidden="true"></span>
    <span class="header-title">{title}</span>
  </div>

  <div class="header-right">
    {#if islandState.mode === 'expanded'}
      <button
        type="button"
        class="icon-btn settings-btn"
        onclick={handleSettings}
        title="Settings & Preferences"
        aria-label="Open settings"
      >
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3"></circle>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
        </svg>
      </button>
    {/if}
    <button
      type="button"
      class="icon-btn collapse-btn"
      onclick={handleCollapse}
      title="Collapse Island"
      aria-label="Collapse island to compact pill"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="18 15 12 9 6 15"></polyline>
      </svg>
    </button>
  </div>
</header>

<div class="header-divider" role="separator" aria-orientation="horizontal"></div>

<style>
  .island-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 2px 2px 8px 2px;
    box-sizing: border-box;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 8px;
    overflow: hidden;
    min-height: 24px;
  }

  .status-indicator-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: var(--island-green, #10b981);
    box-shadow: 0 0 8px rgba(16, 185, 129, 0.6);
    flex-shrink: 0;
  }

  .status-indicator-dot.media-dot {
    background-color: var(--island-accent, #3b82f6);
    box-shadow: 0 0 8px rgba(59, 130, 246, 0.6);
  }

  .header-title {
    font-size: 13px;
    font-weight: 600;
    line-height: 1;
    color: var(--island-text-primary, #f9fafb);
    letter-spacing: -0.01em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .icon-btn {
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

  .icon-btn:hover {
    background: var(--island-card-hover, rgba(255, 255, 255, 0.16));
    border-color: var(--island-border-hover, rgba(255, 255, 255, 0.25));
    color: var(--island-text-primary, #ffffff);
  }

  .icon-btn:active {
    transform: scale(0.92);
  }

  .header-divider {
    width: 100%;
    height: 1px;
    background: var(--island-border, rgba(255, 255, 255, 0.12));
    margin-bottom: 12px;
  }
</style>
