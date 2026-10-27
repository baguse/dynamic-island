<script lang="ts">
  import { onMount } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { islandState } from '../state/island.svelte';

  const media = $derived(islandState.currentMedia);
  const isPlaying = $derived(islandState.isPlaying);
  const companionStyle = $derived(islandState.settings.media_companion_style || 'vinyl');
  const savedPath = $derived(islandState.settings.media_companion_path);

  let fileInputRef: HTMLInputElement | null = $state(null);
  let videoRef: HTMLVideoElement | null = $state(null);
  let isDragOver = $state(false);
  let customUrl = $state<string | null>(null);
  let customType = $state<'video' | 'image' | null>(null);
  let customName = $state<string | null>(null);
  let isMuted = $state(true);
  let isLooping = $state(true);
  let isVideoPlaying = $state(true);
  let objectFitMode = $state<'contain' | 'cover'>('cover');

  // IndexedDB persistence helper for offline / session persistence of selected files
  const DB_NAME = 'dynamic-island-media';
  const DB_STORE = 'companion';

  function openMediaDB(): Promise<IDBDatabase> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, 1);
      request.onupgradeneeded = () => {
        const db = request.result;
        if (!db.objectStoreNames.contains(DB_STORE)) {
          db.createObjectStore(DB_STORE);
        }
      };
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }

  async function saveBlobToDB(blob: Blob, name: string, type: 'video' | 'image') {
    try {
      const db = await openMediaDB();
      const tx = db.transaction(DB_STORE, 'readwrite');
      tx.objectStore(DB_STORE).put({ blob, name, type }, 'active_companion');
    } catch (e) {
      console.debug('IndexedDB save note:', e);
    }
  }

  async function loadBlobFromDB(): Promise<{ blob: Blob; name: string; type: 'video' | 'image' } | null> {
    try {
      const db = await openMediaDB();
      return new Promise((resolve) => {
        const tx = db.transaction(DB_STORE, 'readonly');
        const req = tx.objectStore(DB_STORE).get('active_companion');
        req.onsuccess = () => resolve(req.result || null);
        req.onerror = () => resolve(null);
      });
    } catch {
      return null;
    }
  }

  async function clearBlobFromDB() {
    try {
      const db = await openMediaDB();
      const tx = db.transaction(DB_STORE, 'readwrite');
      tx.objectStore(DB_STORE).delete('active_companion');
    } catch (e) {
      console.debug('IndexedDB clear note:', e);
    }
  }

  onMount(async () => {
    // 1. If a path was saved in settings, attempt to resolve via Tauri asset protocol
    if (savedPath) {
      try {
        const ext = savedPath.split('.').pop()?.toLowerCase();
        const isVid = ext === 'mp4' || ext === 'webm' || ext === 'mkv' || ext === 'm4v' || ext === 'mov';
        customType = isVid ? 'video' : 'image';
        customUrl = convertFileSrc(savedPath);
        customName = savedPath.split('/').pop() || 'Media File';
        return;
      } catch (err) {
        console.debug('convertFileSrc note:', err);
      }
    }

    // 2. Fallback to cached Blob in IndexedDB
    const cached = await loadBlobFromDB();
    if (cached && cached.blob) {
      customType = cached.type;
      customName = cached.name;
      customUrl = URL.createObjectURL(cached.blob);
    }
  });

  function handleTriggerUpload(e: MouseEvent) {
    e.stopPropagation();
    fileInputRef?.click();
  }

  function handleFileSelected(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      applyFile(target.files[0]);
    }
  }

  function applyFile(file: File) {
    const rawPath = (file as unknown as { path?: string }).path;
    const name = file.name;
    const ext = name.split('.').pop()?.toLowerCase();
    const isVid = file.type.startsWith('video/') || ext === 'mp4' || ext === 'webm' || ext === 'mov';
    
    customType = isVid ? 'video' : 'image';
    customName = name;

    if (rawPath) {
      // Use persistent Tauri asset url
      customUrl = convertFileSrc(rawPath);
      islandState.updateSettings({ media_companion_path: rawPath });
    } else {
      // Use Blob URL and store in IndexedDB
      if (customUrl && customUrl.startsWith('blob:')) {
        URL.revokeObjectURL(customUrl);
      }
      customUrl = URL.createObjectURL(file);
      saveBlobToDB(file, name, customType);
      islandState.updateSettings({ media_companion_path: null });
    }
  }

  async function handleResetToVisualizer(e: MouseEvent) {
    e.stopPropagation();
    if (customUrl && customUrl.startsWith('blob:')) {
      URL.revokeObjectURL(customUrl);
    }
    customUrl = null;
    customType = null;
    customName = null;
    await clearBlobFromDB();
    await islandState.updateSettings({ media_companion_path: null });
  }

  function handleToggleStyle(e: MouseEvent) {
    e.stopPropagation();
    const next = companionStyle === 'vinyl' ? 'cassette' : 'vinyl';
    islandState.updateSettings({ media_companion_style: next });
  }

  function handleToggleMute(e: MouseEvent) {
    e.stopPropagation();
    isMuted = !isMuted;
  }

  function handleToggleVideoPlayback(e: MouseEvent) {
    e.stopPropagation();
    if (videoRef) {
      if (videoRef.paused) {
        videoRef.play().catch(() => {});
        isVideoPlaying = true;
      } else {
        videoRef.pause();
        isVideoPlaying = false;
      }
    } else {
      isVideoPlaying = !isVideoPlaying;
    }
  }

  function handleToggleLoop(e: MouseEvent) {
    e.stopPropagation();
    isLooping = !isLooping;
    if (videoRef) {
      videoRef.loop = isLooping;
    }
  }

  function handleVideoTimeUpdate(e: Event) {
    const v = e.currentTarget as HTMLVideoElement;
    if (isLooping && v.duration > 0) {
      // Seamlessly catch end-of-stream freeze in Linux WebKitGTK / GStreamer:
      if (v.currentTime >= v.duration - 0.08) {
        v.currentTime = 0;
        v.play().catch(() => {});
      }
    }
  }

  function handleVideoEnded(e: Event) {
    const v = e.currentTarget as HTMLVideoElement;
    if (isLooping) {
      v.currentTime = 0;
      v.play().catch(() => {});
    }
  }

  function handleToggleFitMode(e: MouseEvent) {
    e.stopPropagation();
    objectFitMode = objectFitMode === 'cover' ? 'contain' : 'cover';
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

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragOver = false;
    if (e.dataTransfer && e.dataTransfer.files.length > 0) {
      const file = e.dataTransfer.files[0];
      applyFile(file);
    }
  }

  function handleTurntableClick(e: MouseEvent) {
    e.stopPropagation();
    islandState.togglePlayPause();
  }
</script>

<div
  class="media-companion-card"
  class:is-drag-over={isDragOver}
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
  role="region"
  aria-label="Media Visual Companion"
>
  <!-- Hidden File Input for Video/GIF/Image selection -->
  <input
    type="file"
    accept="video/mp4,video/webm,video/ogg,image/gif,image/png,image/jpeg,image/webp,image/svg+xml"
    bind:this={fileInputRef}
    onchange={handleFileSelected}
    style="display: none;"
  />

  <!-- Top Bar Controls -->
  <div
    class="companion-topbar"
    class:overlay-hover={!!customUrl}
  >
    <div class="topbar-left">
      {#if customUrl}
        <span class="companion-badge">
          {#if customType === 'video'}🎬 Video{:else}🖼️ GIF/Image{/if}
        </span>
        <span class="companion-filename" title={customName || 'Media File'}>
          {customName || 'Media File'}
        </span>
      {:else if companionStyle === 'vinyl'}
        <span class="companion-badge">💿 Vinyl Turntable</span>
        <span class="companion-hint">Interactive 33 RPM</span>
      {:else}
        <span class="companion-badge">📼 Retro Cassette</span>
        <span class="companion-hint">Stereo High-Bias</span>
      {/if}
    </div>

    <div class="topbar-right">
      {#if customUrl}
        {#if customType === 'video'}
          <button
            type="button"
            class="topbar-btn"
            class:active-btn={isLooping}
            onclick={handleToggleLoop}
            title={isLooping ? 'Looping is ON (click to disable)' : 'Looping is OFF (click to enable)'}
          >
            🔁 {isLooping ? 'Loop: ON' : 'Loop: OFF'}
          </button>

          <button
            type="button"
            class="topbar-btn"
            onclick={handleToggleMute}
            title={isMuted ? 'Unmute Audio' : 'Mute Audio'}
          >
            {#if isMuted}
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="1" y1="1" x2="23" y2="23"></line>
                <path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6"></path>
                <path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2a7 7 0 0 1-.11 1.23"></path>
              </svg>
            {:else}
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
                <path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"></path>
              </svg>
            {/if}
          </button>
        {/if}

        <button
          type="button"
          class="topbar-btn"
          onclick={handleToggleFitMode}
          title={objectFitMode === 'cover' ? 'Switch to Contain' : 'Switch to Fill/Cover'}
        >
          {objectFitMode === 'cover' ? '🔲 Fit' : '🔳 Fill'}
        </button>

        <button
          type="button"
          class="topbar-btn btn-reset"
          onclick={handleResetToVisualizer}
          title="Back to Turntable / Cassette"
        >
          ✕ Reset
        </button>
      {:else}
        <!-- Switch between Vinyl and Cassette style -->
        <button
          type="button"
          class="topbar-btn"
          onclick={handleToggleStyle}
          title={companionStyle === 'vinyl' ? 'Switch to Cassette Tape' : 'Switch to Vinyl Turntable'}
        >
          {companionStyle === 'vinyl' ? '📼 Cassette' : '💿 Vinyl'}
        </button>
      {/if}

      <!-- Load Custom File button -->
      <button
        type="button"
        class="topbar-btn btn-load"
        onclick={handleTriggerUpload}
        title="Load local Video, GIF, or Image"
      >
        📁 {customUrl ? 'Change' : 'Load Video/GIF'}
      </button>
    </div>
  </div>

  <!-- Stage Canvas -->
  <div class="companion-stage">
    {#if customUrl}
      <!-- Custom User Video / GIF / Image Player -->
      {#if customType === 'video'}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <video
          bind:this={videoRef}
          class="media-video"
          class:fit-cover={objectFitMode === 'cover'}
          class:fit-contain={objectFitMode === 'contain'}
          src={customUrl}
          autoplay
          loop={isLooping}
          muted={isMuted}
          playsinline
          preload="auto"
          controls={false}
          onclick={handleToggleVideoPlayback}
          onplay={() => { isVideoPlaying = true; }}
          onpause={() => { isVideoPlaying = false; }}
          ontimeupdate={handleVideoTimeUpdate}
          onended={handleVideoEnded}
        ></video>
      {:else}
        <img
          class="media-image"
          class:fit-cover={objectFitMode === 'cover'}
          class:fit-contain={objectFitMode === 'contain'}
          src={customUrl}
          alt={customName || 'Visual Companion'}
        />
      {/if}

      <!-- Subtle overlay hover controls for custom media -->
      <div class="video-overlay-controls">
        {#if customType === 'video'}
          <button
            type="button"
            class="overlay-pill-btn"
            onclick={handleToggleVideoPlayback}
          >
            {isVideoPlaying ? '⏸ Pause Video' : '▶ Play Video'}
          </button>
        {/if}
      </div>
    {:else if companionStyle === 'vinyl'}
      <!-- Interactive Vinyl Turntable Display -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="turntable-deck"
        onclick={handleTurntableClick}
        title={isPlaying ? 'Click to Pause MPRIS' : 'Click to Play MPRIS'}
      >
        <!-- Metallic Turntable Platter Rim -->
        <div class="turntable-platter">
          <!-- Spinning Vinyl Disc -->
          <div
            class="vinyl-disc"
            class:spinning={isPlaying}
          >
            <!-- Vinyl Grooves Rings -->
            <div class="vinyl-groove-ring ring-1"></div>
            <div class="vinyl-groove-ring ring-2"></div>
            <div class="vinyl-groove-ring ring-3"></div>

            <!-- Center Artwork Label -->
            <div class="vinyl-center-label">
              {#if media?.art_url}
                <img
                  src={media.art_url}
                  alt="Track Label"
                  class="vinyl-artwork-img"
                  onerror={(e) => {
                    (e.currentTarget as HTMLElement).style.display = 'none';
                  }}
                />
              {/if}
              <div class="vinyl-label-fallback" aria-hidden="true">
                <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M9 18V5l12-2v13"></path>
                  <circle cx="6" cy="18" r="3"></circle>
                  <circle cx="18" cy="16" r="3"></circle>
                </svg>
              </div>
              <div class="vinyl-spindle-hole"></div>
            </div>
          </div>
        </div>

        <!-- Realistic Mechanical Tonearm -->
        <div class="tonearm-assembly" class:playing={isPlaying}>
          <div class="tonearm-base"></div>
          <div class="tonearm-pivot"></div>
          <div class="tonearm-arm"></div>
          <div class="tonearm-cartridge"></div>
        </div>

        <!-- Turntable Status Info -->
        <div class="turntable-meta">
          <span class="meta-status" class:active={isPlaying}>
            <span class="status-dot"></span>
            {isPlaying ? '33 RPM • ACTIVE' : 'STOPPED'}
          </span>
          {#if media?.title}
            <span class="meta-title">{media.title}</span>
          {/if}
        </div>
      </div>
    {:else}
      <!-- Retro Cassette Tape Display -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="cassette-deck"
        onclick={handleTurntableClick}
        title={isPlaying ? 'Click to Pause MPRIS' : 'Click to Play MPRIS'}
      >
        <div class="cassette-shell">
          <!-- Cassette Screws -->
          <span class="screw screw-tl"></span>
          <span class="screw screw-tr"></span>
          <span class="screw screw-bl"></span>
          <span class="screw screw-br"></span>

          <!-- Cassette Label Section -->
          <div class="cassette-label">
            <div class="label-header">
              <span class="label-side">A</span>
              <span class="label-brand">DYNAMIC AUDIO • CHROME 90</span>
              <span class="label-noise">NR: ON</span>
            </div>
            <div class="label-song-info">
              <span class="song-title">{media?.title || 'Ready for Playback'}</span>
              <span class="song-artist">{media?.artist || 'Dynamic Island Player'}</span>
            </div>
          </div>

          <!-- Cassette Window with Spinning Tape Spools -->
          <div class="cassette-window">
            <!-- Left Spool -->
            <div class="tape-spool left-spool" class:spinning={isPlaying}>
              <div class="spool-gear"></div>
              <div class="spool-teeth"></div>
            </div>

            <!-- Tape bridge & tape counter -->
            <div class="tape-counter">
              <span class="tape-bridge"></span>
              <span class="counter-num">{isPlaying ? 'PLAY' : 'PAUSE'}</span>
            </div>

            <!-- Right Spool -->
            <div class="tape-spool right-spool" class:spinning={isPlaying}>
              <div class="spool-gear"></div>
              <div class="spool-teeth"></div>
            </div>
          </div>
        </div>
      </div>
    {/if}

    <!-- Drop Zone Prompt Overlay (shown on drag over) -->
    {#if isDragOver}
      <div class="drop-overlay">
        <span class="drop-icon">📥</span>
        <span class="drop-text">Drop Video, GIF, or Image here!</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .media-companion-card {
    position: relative;
    width: 100%;
    height: 220px;
    background: rgba(15, 17, 21, 0.75);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    margin-top: 14px;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.05), 0 8px 24px rgba(0, 0, 0, 0.4);
    transition: border-color 0.2s ease, box-shadow 0.2s ease;
  }

  .media-companion-card.is-drag-over {
    border-color: #3b82f6 !important;
    box-shadow: 0 0 24px rgba(59, 130, 246, 0.4) !important;
  }

  /* Top Bar */
  .companion-topbar {
    height: 32px;
    padding: 0 12px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: rgba(0, 0, 0, 0.35);
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    flex-shrink: 0;
    z-index: 10;
    transition: opacity 0.22s cubic-bezier(0.2, 0.9, 0.3, 1), transform 0.22s cubic-bezier(0.2, 0.9, 0.3, 1);
  }

  /* Overlay mode when video or custom media is loaded: completely hidden until hover */
  .companion-topbar.overlay-hover {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 38px;
    background: linear-gradient(180deg, rgba(0, 0, 0, 0.88) 0%, rgba(0, 0, 0, 0.5) 65%, transparent 100%);
    border-bottom: none;
    opacity: 0;
    transform: translateY(-8px);
    pointer-events: none;
  }

  .media-companion-card:hover .companion-topbar.overlay-hover,
  .media-companion-card:focus-within .companion-topbar.overlay-hover {
    opacity: 1;
    transform: translateY(0);
    pointer-events: auto;
  }

  .topbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    overflow: hidden;
  }

  .companion-badge {
    font-size: 10.5px;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
    letter-spacing: 0.02em;
    white-space: nowrap;
  }

  .companion-hint {
    font-size: 10px;
    color: var(--island-text-muted, #6b7280);
    white-space: nowrap;
  }

  .companion-filename {
    font-size: 10px;
    color: var(--island-text-secondary, #9ca3af);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 140px;
  }

  .topbar-right {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .topbar-btn {
    height: 22px;
    padding: 0 8px;
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    font-size: 10px;
    font-weight: 500;
    color: var(--island-text-secondary, #cbd5e1);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
  }

  .topbar-btn:hover {
    background: rgba(255, 255, 255, 0.14);
    color: #ffffff;
    border-color: rgba(255, 255, 255, 0.2);
  }

  .topbar-btn.active-btn {
    background: rgba(59, 130, 246, 0.25);
    border-color: rgba(59, 130, 246, 0.5);
    color: #93c5fd;
  }

  .topbar-btn.btn-load {
    background: rgba(59, 130, 246, 0.18);
    border-color: rgba(59, 130, 246, 0.35);
    color: #93c5fd;
  }

  .topbar-btn.btn-load:hover {
    background: rgba(59, 130, 246, 0.3);
    color: #ffffff;
  }

  .topbar-btn.btn-reset {
    color: #f87171;
    border-color: rgba(239, 68, 68, 0.3);
  }

  .topbar-btn.btn-reset:hover {
    background: rgba(239, 68, 68, 0.2);
    color: #fca5a5;
  }

  /* Stage Canvas */
  .companion-stage {
    position: relative;
    width: 100%;
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  /* Video & Image Player */
  .media-video {
    width: 100%;
    height: 100%;
    display: block;
    cursor: pointer;
  }

  .media-image {
    width: 100%;
    height: 100%;
    display: block;
  }

  .fit-cover {
    object-fit: cover;
  }

  .fit-contain {
    object-fit: contain;
    background: #000000;
  }

  .video-overlay-controls {
    position: absolute;
    bottom: 8px;
    right: 8px;
    display: flex;
    gap: 6px;
    opacity: 0;
    transform: translateY(6px);
    transition: opacity 0.2s ease, transform 0.2s ease;
    z-index: 5;
    pointer-events: none;
  }

  .media-companion-card:hover .video-overlay-controls,
  .media-companion-card:focus-within .video-overlay-controls {
    opacity: 1;
    transform: translateY(0);
    pointer-events: auto;
  }

  .overlay-pill-btn {
    padding: 4px 10px;
    font-size: 10px;
    font-weight: 600;
    background: rgba(0, 0, 0, 0.7);
    color: #ffffff;
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 9999px;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .overlay-pill-btn:hover {
    background: rgba(0, 0, 0, 0.9);
  }

  /* ========================================================================= */
  /* Vinyl Turntable Styling                                                   */
  /* ========================================================================= */
  .turntable-deck {
    position: relative;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: radial-gradient(circle at 45% 50%, #171920, #0a0b0e);
    cursor: pointer;
    user-select: none;
  }

  .turntable-platter {
    position: relative;
    width: 168px;
    height: 168px;
    border-radius: 50%;
    background: radial-gradient(circle, #252830 0%, #12141a 100%);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.8), inset 0 0 10px rgba(0, 0, 0, 0.9);
    display: flex;
    align-items: center;
    justify-content: center;
    border: 2px solid rgba(255, 255, 255, 0.1);
  }

  .vinyl-disc {
    position: relative;
    width: 156px;
    height: 156px;
    border-radius: 50%;
    background:
      repeating-radial-gradient(
        circle,
        #111113 0px,
        #111113 2px,
        #19191d 3px,
        #111113 4px
      ),
      conic-gradient(
        from 0deg,
        rgba(255, 255, 255, 0.15) 0deg,
        transparent 60deg,
        rgba(255, 255, 255, 0.12) 180deg,
        transparent 240deg,
        rgba(255, 255, 255, 0.15) 360deg
      );
    box-shadow: 0 2px 14px rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    will-change: transform;
    animation: vinylSpin 3s linear infinite;
    animation-play-state: paused;
  }

  .vinyl-disc.spinning {
    animation-play-state: running;
  }

  @keyframes vinylSpin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .vinyl-groove-ring {
    position: absolute;
    border-radius: 50%;
    border: 1px dashed rgba(255, 255, 255, 0.05);
    pointer-events: none;
  }

  .ring-1 {
    width: 130px;
    height: 130px;
  }

  .ring-2 {
    width: 104px;
    height: 104px;
  }

  .ring-3 {
    width: 78px;
    height: 78px;
  }

  .vinyl-center-label {
    position: relative;
    width: 54px;
    height: 54px;
    border-radius: 50%;
    background: #1e293b;
    border: 2px solid #f59e0b;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.6);
    z-index: 2;
  }

  .vinyl-artwork-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .vinyl-label-fallback {
    color: #f59e0b;
  }

  .vinyl-spindle-hole {
    position: absolute;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #000000;
    border: 2px solid #cbd5e1;
    z-index: 4;
  }

  /* Mechanical Tonearm */
  .tonearm-assembly {
    position: absolute;
    top: 14px;
    right: 48px;
    width: 80px;
    height: 130px;
    pointer-events: none;
    transition: transform 0.6s cubic-bezier(0.34, 1.56, 0.64, 1);
    transform-origin: 65px 15px;
    transform: rotate(-18deg);
  }

  .tonearm-assembly.playing {
    transform: rotate(14deg);
  }

  .tonearm-base {
    position: absolute;
    top: 5px;
    right: 5px;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: radial-gradient(circle, #64748b, #1e293b);
    border: 1px solid rgba(255, 255, 255, 0.2);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.6);
  }

  .tonearm-pivot {
    position: absolute;
    top: 12px;
    right: 12px;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #cbd5e1;
  }

  .tonearm-arm {
    position: absolute;
    top: 16px;
    right: 14px;
    width: 3px;
    height: 85px;
    background: linear-gradient(to right, #94a3b8, #e2e8f0, #64748b);
    border-radius: 2px;
    box-shadow: 1px 2px 4px rgba(0, 0, 0, 0.4);
    transform-origin: top center;
    transform: rotate(12deg);
  }

  .tonearm-cartridge {
    position: absolute;
    bottom: 24px;
    left: 42px;
    width: 10px;
    height: 16px;
    background: #f59e0b;
    border-radius: 2px;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.6);
    transform: rotate(18deg);
  }

  .turntable-meta {
    position: absolute;
    bottom: 8px;
    left: 14px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-width: 220px;
  }

  .meta-status {
    font-size: 9.5px;
    font-weight: 700;
    color: var(--island-text-muted, #6b7280);
    display: flex;
    align-items: center;
    gap: 5px;
    letter-spacing: 0.05em;
  }

  .meta-status.active {
    color: #10b981;
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
  }

  .meta-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* ========================================================================= */
  /* Retro Cassette Tape Styling                                               */
  /* ========================================================================= */
  .cassette-deck {
    position: relative;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: radial-gradient(circle at 50% 50%, #1e1e24, #0b0c0e);
    cursor: pointer;
    user-select: none;
  }

  .cassette-shell {
    position: relative;
    width: 280px;
    height: 145px;
    background: linear-gradient(135deg, #2a2d34, #18191d);
    border-radius: 12px;
    border: 2px solid rgba(255, 255, 255, 0.12);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.7), inset 0 1px 2px rgba(255, 255, 255, 0.15);
    display: flex;
    flex-direction: column;
    padding: 10px 14px;
    box-sizing: border-box;
  }

  .screw {
    position: absolute;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: #64748b;
    border: 1px solid rgba(0, 0, 0, 0.6);
  }

  .screw-tl { top: 6px; left: 6px; }
  .screw-tr { top: 6px; right: 6px; }
  .screw-bl { bottom: 6px; left: 6px; }
  .screw-br { bottom: 6px; right: 6px; }

  .cassette-label {
    width: 100%;
    background: #e2e8f0;
    border-radius: 6px;
    padding: 6px 10px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 3px;
    color: #0f172a;
    box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.2);
  }

  .label-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 8px;
    font-weight: 800;
    letter-spacing: 0.05em;
    color: #475569;
    border-bottom: 1px solid #cbd5e1;
    padding-bottom: 2px;
  }

  .label-side {
    font-size: 11px;
    font-weight: 900;
    color: #dc2626;
  }

  .label-song-info {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .song-title {
    font-size: 11px;
    font-weight: 700;
    color: #0f172a;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .song-artist {
    font-size: 9.5px;
    font-weight: 500;
    color: #64748b;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .cassette-window {
    width: 100%;
    flex: 1;
    background: rgba(0, 0, 0, 0.45);
    border-radius: 6px;
    margin-top: 8px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    display: flex;
    align-items: center;
    justify-content: space-around;
    padding: 4px 16px;
    box-sizing: border-box;
    box-shadow: inset 0 2px 8px rgba(0, 0, 0, 0.8);
  }

  .tape-spool {
    position: relative;
    width: 42px;
    height: 42px;
    border-radius: 50%;
    background: #ffffff;
    border: 2px solid #94a3b8;
    display: flex;
    align-items: center;
    justify-content: center;
    will-change: transform;
    animation: reelSpin 2s linear infinite;
    animation-play-state: paused;
  }

  .tape-spool.spinning {
    animation-play-state: running;
  }

  @keyframes reelSpin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .spool-gear {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #0f172a;
  }

  .spool-teeth {
    position: absolute;
    width: 26px;
    height: 26px;
    border: 2px dashed #94a3b8;
    border-radius: 50%;
  }

  .tape-counter {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }

  .tape-bridge {
    width: 48px;
    height: 4px;
    background: #451a03;
    border-radius: 2px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.6);
  }

  .counter-num {
    font-size: 8.5px;
    font-weight: 800;
    color: #94a3b8;
    letter-spacing: 0.08em;
  }

  /* Drop Overlay */
  .drop-overlay {
    position: absolute;
    inset: 0;
    background: rgba(15, 23, 42, 0.92);
    backdrop-filter: blur(8px);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    z-index: 20;
    border: 2px dashed #3b82f6;
  }

  .drop-icon {
    font-size: 28px;
  }

  .drop-text {
    font-size: 12px;
    font-weight: 600;
    color: #93c5fd;
  }
</style>
