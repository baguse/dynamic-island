<script lang="ts">
  import IslandHeader from './IslandHeader.svelte';
  import MediaCompanion from './MediaCompanion.svelte';
  import AudioEqualizer from './AudioEqualizer.svelte';
  import { islandState } from '../state/island.svelte';

  const media = $derived(islandState.currentMedia);
  const showCompanion = $derived(islandState.settings.show_media_companion ?? true);

  function formatTime(ms: number | null | undefined): string {
    if (!ms || ms <= 0) return '0:00';
    const totalSeconds = Math.floor(ms / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  }

  const progressPercent = $derived(() => {
    if (!media?.position_ms || !media?.length_ms || media.length_ms <= 0) {
      return 0;
    }
    return Math.min(100, Math.max(0, (media.position_ms / media.length_ms) * 100));
  });

  function handlePlayPause(e: MouseEvent) {
    e.stopPropagation();
    islandState.togglePlayPause();
  }

  function handleNext(e: MouseEvent) {
    e.stopPropagation();
    islandState.nextTrack();
  }

  function handlePrevious(e: MouseEvent) {
    e.stopPropagation();
    islandState.previousTrack();
  }

  function handleBack(e: MouseEvent) {
    e.stopPropagation();
    islandState.openOverview();
  }
</script>

<div class="media-preview-wrapper">
  <IslandHeader title={media?.player_name ? `${media.player_name}` : 'Now Playing'}>
    <button
      type="button"
      class="companion-toggle-btn"
      class:active={showCompanion}
      onclick={() => islandState.updateSettings({ show_media_companion: !showCompanion })}
      title={showCompanion ? 'Hide Visual Companion' : 'Show Visual Companion'}
    >
      {showCompanion ? 'Visualizer: ON' : 'Visualizer: OFF'}
    </button>
  </IslandHeader>

  {#if media}
    <div class="track-row">
      <div class="artwork-container">
        {#if media.art_url}
          <img
            src={media.art_url}
            alt="Album Art"
            class="artwork-img"
            onerror={(e) => {
              (e.currentTarget as HTMLElement).style.display = 'none';
            }}
          />
        {/if}
        <div class="artwork-fallback" aria-hidden="true">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9 18V5l12-2v13"></path>
            <circle cx="6" cy="18" r="3"></circle>
            <circle cx="18" cy="16" r="3"></circle>
          </svg>
        </div>
      </div>

      <div class="track-info">
        <div class="track-title-row">
          <h4 class="track-title" title={media.title}>{media.title}</h4>
          <AudioEqualizer size="sm" isPlaying={islandState.isPlaying} />
        </div>
        <p class="track-artist" title={media.artist}>{media.artist}</p>
        {#if media.album}
          <p class="track-album" title={media.album}>{media.album}</p>
        {/if}
      </div>
    </div>

    <!-- Progress bar -->
    {#if media.length_ms}
      <div class="progress-section">
        <div class="progress-bar-bg">
          <div class="progress-bar-fill" style="width: {progressPercent()}%;"></div>
        </div>
        <div class="time-row">
          <span>{formatTime(media.position_ms)}</span>
          <span>{formatTime(media.length_ms)}</span>
        </div>
      </div>
    {/if}

    <!-- Playback controls -->
    <div class="controls-row">
      <button
        type="button"
        class="ctrl-btn"
        onclick={handlePrevious}
        aria-label="Previous track"
        title="Previous Track"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
          <polygon points="19 20 9 12 19 4 19 20"></polygon>
          <line x1="5" y1="19" x2="5" y2="5" stroke="currentColor" stroke-width="2" stroke-linecap="round"></line>
        </svg>
      </button>

      <button
        type="button"
        class="ctrl-btn play-btn"
        onclick={handlePlayPause}
        aria-label={islandState.isPlaying ? 'Pause' : 'Play'}
        title={islandState.isPlaying ? 'Pause' : 'Play'}
      >
        {#if islandState.isPlaying}
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <rect x="6" y="4" width="4" height="16" rx="1"></rect>
            <rect x="14" y="4" width="4" height="16" rx="1"></rect>
          </svg>
        {:else}
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <polygon points="5 3 19 12 5 21 5 3"></polygon>
          </svg>
        {/if}
      </button>

      <button
        type="button"
        class="ctrl-btn"
        onclick={handleNext}
        aria-label="Next track"
        title="Next Track"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
          <polygon points="5 4 15 12 5 20 5 4"></polygon>
          <line x1="19" y1="5" x2="19" y2="19" stroke="currentColor" stroke-width="2" stroke-linecap="round"></line>
        </svg>
      </button>
    </div>
  {:else}
    <!-- No Active Player State -->
    <div class="empty-media">
      <div class="empty-icon" aria-hidden="true">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
          <path d="M9 18V5l12-2v13"></path>
          <circle cx="6" cy="18" r="3"></circle>
          <circle cx="18" cy="16" r="3"></circle>
        </svg>
      </div>
      <p class="empty-title">No Media Active</p>
      <p class="empty-desc">
        Start playback in Spotify, VLC, or a browser to control it from the Island.
      </p>
      {#if !showCompanion}
        <button
          type="button"
          class="btn-back-overview"
          onclick={handleBack}
        >
          Back to Overview
        </button>
      {/if}
    </div>
  {/if}

  {#if showCompanion}
    <MediaCompanion />
  {/if}
</div>

<style>
  .companion-toggle-btn {
    height: 22px;
    padding: 0 8px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 6px;
    font-size: 10px;
    font-weight: 600;
    color: var(--island-text-secondary, #9ca3af);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .companion-toggle-btn.active {
    background: rgba(59, 130, 246, 0.2);
    border-color: rgba(59, 130, 246, 0.4);
    color: #93c5fd;
  }

  .companion-toggle-btn:hover {
    color: #ffffff;
    border-color: rgba(255, 255, 255, 0.25);
  }
  .media-preview-wrapper {
    width: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
  }

  .track-row {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-bottom: 12px;
  }

  .artwork-container {
    width: 52px;
    height: 52px;
    border-radius: 10px;
    position: relative;
    overflow: hidden;
    flex-shrink: 0;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    background: linear-gradient(135deg, #1e293b, #0f172a);
  }

  .artwork-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    position: absolute;
    top: 0;
    left: 0;
    z-index: 2;
  }

  .artwork-fallback {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #64748b;
    z-index: 1;
  }

  .track-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow: hidden;
  }

  .track-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    overflow: hidden;
  }

  .track-title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .track-artist {
    margin: 0;
    font-size: 12px;
    color: var(--island-text-secondary, #9ca3af);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .track-album {
    margin: 0;
    font-size: 11px;
    color: var(--island-text-muted, #6b7280);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .progress-section {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 10px;
  }

  .progress-bar-bg {
    width: 100%;
    height: 4px;
    background: rgba(255, 255, 255, 0.12);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background: #3b82f6;
    border-radius: 2px;
    transition: width 0.3s ease;
  }

  .time-row {
    display: flex;
    justify-content: space-between;
    font-size: 10px;
    color: var(--island-text-muted, #6b7280);
    font-variant-numeric: tabular-nums;
  }

  .controls-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding-top: 2px;
  }

  .ctrl-btn {
    background: none;
    border: none;
    color: var(--island-text-primary, #ffffff);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    transition: background 0.15s ease, transform 0.1s ease, color 0.15s ease;
  }

  .ctrl-btn:hover {
    background: rgba(255, 255, 255, 0.12);
  }

  .ctrl-btn:active {
    transform: scale(0.92);
  }

  .ctrl-btn.play-btn {
    background: #ffffff;
    color: #000000;
    width: 42px;
    height: 42px;
  }

  .ctrl-btn.play-btn:hover {
    background: #e2e8f0;
    transform: scale(1.05);
  }

  .ctrl-btn.play-btn:active {
    transform: scale(0.95);
  }

  /* Empty state */
  .empty-media {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 10px 0 4px 0;
  }

  .empty-icon {
    color: #64748b;
    margin-bottom: 8px;
  }

  .empty-title {
    margin: 0 0 4px 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
  }

  .empty-desc {
    margin: 0 0 12px 0;
    font-size: 11px;
    color: var(--island-text-secondary, #9ca3af);
    max-width: 280px;
    line-height: 1.3;
  }

  .btn-back-overview {
    padding: 6px 14px;
    font-size: 11px;
    font-weight: 500;
    border-radius: 6px;
    border: none;
    background: rgba(255, 255, 255, 0.08);
    color: var(--island-text-primary, #ffffff);
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .btn-back-overview:hover {
    background: rgba(255, 255, 255, 0.16);
  }
</style>
