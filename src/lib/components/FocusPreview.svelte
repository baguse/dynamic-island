<script lang="ts">
  import IslandHeader from './IslandHeader.svelte';
  import PixelMascot from './PixelMascot.svelte';
  import { islandState, FUNNY_FOCUS_QUOTES, FUNNY_FOCUS_EXCUSES } from '../state/island.svelte';

  const session = $derived(islandState.focusSession);
  const isActive = $derived(session.isActive);
  const isPaused = $derived(session.isPaused);
  const formattedTime = $derived(islandState.focusRemainingFormatted);
  const progress = $derived(islandState.focusProgress);

  let customMinutes = $state(25);
  let customLabel = $state('Locking In');
  let selectedPreset = $state(25);
  let showGiveUpConfirm = $state(false);
  let activeExcuse = $state('');
  let copiedExcuse = $state(false);
  let currentQuoteIndex = $state(0);
  let quoteTimer: ReturnType<typeof setInterval> | null = null;

  const currentFunnyQuote = $derived(FUNNY_FOCUS_QUOTES[currentQuoteIndex % FUNNY_FOCUS_QUOTES.length]);

  const productivityRank = $derived.by(() => {
    const count = session.completedCountToday;
    if (count === 0) return { title: 'Couch Potato', emoji: '🥔', desc: 'No sessions yet. Time to lock in!' };
    if (count === 1) return { title: 'Warm-up Rookie', emoji: '👶', desc: '1 session down. The engine is warm.' };
    if (count <= 3) return { title: 'Code Warrior', emoji: '⚔️', desc: 'Solid grind! Slaying bugs left and right.' };
    if (count <= 5) return { title: 'Keyboard Wizard', emoji: '🧙‍♂️', desc: 'Flow state unlocked. Magic in progress.' };
    return { title: 'Productivity Goblin', emoji: '👹', desc: 'Over 9000! Take a break before you ascend.' };
  });

  const PRESETS = [
    { mins: 15, name: 'Quick Sprint', tag: 'Speedrun', icon: '⚡', desc: 'Speedrun 1 bug' },
    { mins: 25, name: 'Standard Grind', tag: 'Pomodoro', icon: '🍅', desc: 'The holy canon' },
    { mins: 45, name: 'Monk Mode', tag: 'Monk Mode', icon: '🧘', desc: 'Transcend distraction' },
    { mins: 60, name: 'GigaChad', tag: 'Suffering', icon: '💀', desc: 'Pure coding grind' },
    { mins: 5,  name: 'Grass Break', tag: 'Chilling', icon: '🌱', desc: 'Hydrate & stretch' },
  ];

  $effect(() => {
    quoteTimer = setInterval(() => {
      currentQuoteIndex = (currentQuoteIndex + 1) % FUNNY_FOCUS_QUOTES.length;
    }, 5500);

    return () => {
      if (quoteTimer) clearInterval(quoteTimer);
    };
  });

  function handleStartPreset(mins: number, tag: string) {
    selectedPreset = mins;
    islandState.startFocus(mins, tag);
    showGiveUpConfirm = false;
  }

  function handleStartCustom() {
    islandState.startFocus(customMinutes, customLabel || 'Deep Work');
    showGiveUpConfirm = false;
  }

  function handleTogglePause() {
    if (isPaused) {
      islandState.resumeFocus();
    } else {
      islandState.pauseFocus();
    }
  }

  function handleAdd5Min() {
    islandState.extendFocus(5);
  }

  function handleGiveUp() {
    if (!showGiveUpConfirm) {
      showGiveUpConfirm = true;
      return;
    }
    islandState.stopFocus(true);
    showGiveUpConfirm = false;
  }

  function handleRollExcuse() {
    activeExcuse = islandState.getRandomExcuse();
    copiedExcuse = false;
  }

  async function handleCopyExcuse() {
    if (!activeExcuse) return;
    try {
      await navigator.clipboard.writeText(activeExcuse);
      copiedExcuse = true;
      setTimeout(() => {
        copiedExcuse = false;
      }, 2000);
    } catch (e) {
      console.debug('Failed to copy excuse:', e);
    }
  }

  // Calculate SVG Circle Stroke Offset
  const CIRCLE_RADIUS = 76;
  const CIRCLE_CIRCUMFERENCE = 2 * Math.PI * CIRCLE_RADIUS;
  const strokeDashoffset = $derived(
    CIRCLE_CIRCUMFERENCE - (progress / 100) * CIRCLE_CIRCUMFERENCE
  );
</script>

<div class="focus-panel">
  <IslandHeader title="Focus Mode">
    <span class="header-subtitle-tag">
      🔥 {session.completedCountToday} sessions • {session.totalMinutesToday}m locked in today
    </span>
  </IslandHeader>

  <div class="focus-layout">
    <!-- Left Column: Big Interactive Timer -->
    <div class="timer-section">
      <div class="timer-display-card" class:active={isActive} class:paused={isPaused}>
        <!-- Circular Progress Ring -->
        <div class="timer-ring-container">
          <svg class="timer-svg" width="180" height="180" viewBox="0 0 180 180">
            <!-- Background circle -->
            <circle
              class="timer-bg-circle"
              cx="90"
              cy="90"
              r={CIRCLE_RADIUS}
            />
            <!-- Animated Progress circle -->
            <circle
              class="timer-progress-circle"
              cx="90"
              cy="90"
              r={CIRCLE_RADIUS}
              style="stroke-dasharray: {CIRCLE_CIRCUMFERENCE}; stroke-dashoffset: {strokeDashoffset};"
            />
          </svg>

          <!-- Inside Ring Content -->
          <div class="ring-inner">
            <div class="ring-mascot-row">
              <PixelMascot size="sm" stateOverride={isActive ? 'typing' : undefined} />
              <span class="focus-flame-badge" class:pulsing={isActive && !isPaused}>
                {isActive ? (isPaused ? '⏸️' : '🔥') : '🎯'}
              </span>
            </div>

            <div class="timer-text-digits">
              {formattedTime}
            </div>

            <div class="timer-session-label">
              {isActive ? session.label : `${customMinutes}m session`}
            </div>
          </div>
        </div>

        <!-- Funny Live Quote / Subtitle -->
        <button
          type="button"
          class="funny-quote-pill"
          title="Click for new random thought"
          onclick={() => (currentQuoteIndex += 1)}
        >
          <span class="quote-text">{currentFunnyQuote}</span>
        </button>

        <!-- Primary Action Buttons -->
        <div class="timer-controls">
          {#if !isActive}
            <button
              type="button"
              class="action-btn start-btn"
              onclick={() => handleStartPreset(selectedPreset, 'Locking In')}
            >
              <span>🚀 Start Session</span>
            </button>
          {:else}
            <button
              type="button"
              class="action-btn pause-btn"
              onclick={handleTogglePause}
            >
              <span>{isPaused ? '▶ Resume' : '⏸ Pause'}</span>
            </button>

            <button
              type="button"
              class="action-btn boost-btn"
              onclick={handleAdd5Min}
              title="Add 5 more minutes ('just one more function')"
            >
              <span>+5m Boost</span>
            </button>

            {#if !showGiveUpConfirm}
              <button
                type="button"
                class="action-btn give-up-btn"
                onclick={() => (showGiveUpConfirm = true)}
                title="Give up and scroll social media"
              >
                <span>💀 Give Up</span>
              </button>
            {:else}
              <button
                type="button"
                class="action-btn confirm-give-up-btn"
                onclick={handleGiveUp}
              >
                <span>Confirm Abort 📱</span>
              </button>
            {/if}
          {/if}
        </div>
      </div>

      <!-- Excuse Generator Widget -->
      <div class="excuse-card">
        <div class="excuse-header">
          <span class="excuse-title">🛡️ Emergency Slack / Meeting Excuse</span>
          <button type="button" class="roll-excuse-btn" onclick={handleRollExcuse}>
            🎲 Roll Excuse
          </button>
        </div>

        {#if activeExcuse}
          <div class="excuse-body">
            <p class="excuse-text">"{activeExcuse}"</p>
            <button
              type="button"
              class="copy-excuse-btn"
              class:copied={copiedExcuse}
              onclick={handleCopyExcuse}
            >
              {copiedExcuse ? '✓ Copied!' : '📋 Copy to Clipboard'}
            </button>
          </div>
        {:else}
          <div class="excuse-placeholder">
            <span>Need an excuse to dodge pings? Click <b>Roll Excuse</b> above!</span>
          </div>
        {/if}
      </div>
    </div>

    <!-- Right Column: Presets & Stats -->
    <div class="presets-section">
      <div class="section-card">
        <span class="card-heading">⚡ Quick Session Presets</span>
        <div class="presets-list">
          {#each PRESETS as p}
            <button
              type="button"
              class="preset-item-btn"
              class:selected={selectedPreset === p.mins}
              onclick={() => handleStartPreset(p.mins, p.tag)}
            >
              <span class="preset-icon">{p.icon}</span>
              <div class="preset-info">
                <div class="preset-title-row">
                  <span class="preset-name">{p.name}</span>
                  <span class="preset-mins">{p.mins}m</span>
                </div>
                <span class="preset-desc">{p.desc}</span>
              </div>
            </button>
          {/each}
        </div>
      </div>

      <!-- Custom Timer Config -->
      <div class="section-card">
        <span class="card-heading">🛠️ Custom Timer</span>
        <div class="custom-timer-form">
          <div class="slider-row">
            <span class="slider-label">Duration: <b>{customMinutes} mins</b></span>
            <input
              type="range"
              min="1"
              max="120"
              step="1"
              class="custom-slider"
              bind:value={customMinutes}
            />
          </div>

          <div class="custom-input-row">
            <input
              type="text"
              class="label-input"
              placeholder="Tag (e.g. Bug Squashing)"
              bind:value={customLabel}
            />
            <button
              type="button"
              class="custom-start-btn"
              onclick={handleStartCustom}
            >
              Start Custom
            </button>
          </div>
        </div>
      </div>

      <!-- Today's Productivity Rank -->
      <div class="section-card rank-card">
        <div class="rank-header">
          <span class="rank-emoji">{productivityRank.emoji}</span>
          <div class="rank-text">
            <span class="rank-badge">Rank: {productivityRank.title}</span>
            <span class="rank-desc">{productivityRank.desc}</span>
          </div>
        </div>

        <div class="stats-grid">
          <div class="stat-box">
            <span class="stat-number">{session.completedCountToday}</span>
            <span class="stat-label">Sessions Won</span>
          </div>
          <div class="stat-box">
            <span class="stat-number">{session.totalMinutesToday}</span>
            <span class="stat-label">Total Minutes</span>
          </div>
          <div class="stat-box">
            <span class="stat-number">{Math.round(session.totalMinutesToday / 25)}</span>
            <span class="stat-label">🍅 Equivalent</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .focus-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    gap: 10px;
    box-sizing: border-box;
  }

  .header-subtitle-tag {
    font-size: 10.5px;
    color: var(--island-text-muted, #94a3b8);
    background: rgba(255, 255, 255, 0.08);
    padding: 2px 7px;
    border-radius: 999px;
  }

  .focus-layout {
    display: grid;
    grid-template-columns: 1.15fr 1fr;
    gap: 12px;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding-right: 2px;
  }

  .timer-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .timer-display-card {
    background: rgba(255, 255, 255, 0.035);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    position: relative;
    transition: all 0.25s ease;
  }

  .timer-display-card.active {
    border-color: rgba(245, 158, 11, 0.4);
    box-shadow: 0 0 20px rgba(245, 158, 11, 0.12);
  }

  .timer-ring-container {
    position: relative;
    width: 180px;
    height: 180px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .timer-svg {
    position: absolute;
    top: 0;
    left: 0;
    transform: rotate(-90deg);
  }

  .timer-bg-circle {
    fill: none;
    stroke: rgba(255, 255, 255, 0.08);
    stroke-width: 6;
  }

  .timer-progress-circle {
    fill: none;
    stroke: var(--island-accent, #f59e0b);
    stroke-width: 7;
    stroke-linecap: round;
    transition: stroke-dashoffset 0.5s ease;
  }

  .ring-inner {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    z-index: 2;
  }

  .ring-mascot-row {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .focus-flame-badge {
    font-size: 14px;
    transition: transform 0.2s ease;
  }

  .focus-flame-badge.pulsing {
    animation: flameBounce 1.5s infinite;
  }

  @keyframes flameBounce {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.25); }
  }

  .timer-text-digits {
    font-size: 32px;
    font-weight: 800;
    color: #ffffff;
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.5px;
    line-height: 1.1;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  }

  .timer-session-label {
    font-size: 10.5px;
    color: var(--island-text-muted, #94a3b8);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.6px;
  }

  .funny-quote-pill {
    background: rgba(245, 158, 11, 0.08);
    border: 1px solid rgba(245, 158, 11, 0.2);
    border-radius: 999px;
    padding: 4px 12px;
    font-size: 10.5px;
    color: #fbbf24;
    cursor: pointer;
    text-align: center;
    max-width: 90%;
    transition: all 0.15s ease;
  }

  .funny-quote-pill:hover {
    background: rgba(245, 158, 11, 0.15);
    border-color: rgba(245, 158, 11, 0.35);
  }

  .quote-text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .timer-controls {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    justify-content: center;
  }

  .action-btn {
    border: none;
    border-radius: 7px;
    font-size: 11px;
    font-weight: 600;
    padding: 7px 14px;
    cursor: pointer;
    transition: all 0.15s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
  }

  .start-btn {
    background: #f59e0b;
    color: #000000;
    font-size: 12px;
    padding: 8px 18px;
  }

  .start-btn:hover {
    background: #fbbf24;
    transform: translateY(-1px);
  }

  .pause-btn {
    background: rgba(255, 255, 255, 0.1);
    color: #ffffff;
    border: 1px solid rgba(255, 255, 255, 0.15);
  }

  .pause-btn:hover {
    background: rgba(255, 255, 255, 0.18);
  }

  .boost-btn {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
    border: 1px solid rgba(16, 185, 129, 0.3);
  }

  .boost-btn:hover {
    background: rgba(16, 185, 129, 0.25);
  }

  .give-up-btn {
    background: rgba(239, 68, 68, 0.1);
    color: #f87171;
    border: 1px solid rgba(239, 68, 68, 0.25);
  }

  .give-up-btn:hover {
    background: rgba(239, 68, 68, 0.2);
  }

  .confirm-give-up-btn {
    background: #ef4444;
    color: #ffffff;
    animation: pulse 1s infinite;
  }

  .excuse-card {
    background: rgba(255, 255, 255, 0.035);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .excuse-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .excuse-title {
    font-size: 10.5px;
    font-weight: 600;
    color: #94a3b8;
  }

  .roll-excuse-btn {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: #ffffff;
    font-size: 10px;
    padding: 3px 8px;
    border-radius: 5px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .roll-excuse-btn:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  .excuse-body {
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: rgba(0, 0, 0, 0.25);
    padding: 8px 10px;
    border-radius: 6px;
    border-left: 2px solid #f59e0b;
  }

  .excuse-text {
    margin: 0;
    font-size: 11px;
    color: #e2e8f0;
    font-style: italic;
    line-height: 1.35;
  }

  .copy-excuse-btn {
    align-self: flex-end;
    background: transparent;
    border: none;
    color: #94a3b8;
    font-size: 10px;
    cursor: pointer;
    padding: 2px 4px;
    transition: all 0.15s;
  }

  .copy-excuse-btn:hover {
    color: #ffffff;
  }

  .copy-excuse-btn.copied {
    color: #10b981;
    font-weight: 600;
  }

  .excuse-placeholder {
    font-size: 10.5px;
    color: #64748b;
    text-align: center;
    padding: 6px;
  }

  .presets-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .section-card {
    background: rgba(255, 255, 255, 0.035);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .card-heading {
    font-size: 11px;
    font-weight: 600;
    color: #cbd5e1;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .presets-list {
    display: grid;
    grid-template-columns: 1fr;
    gap: 6px;
  }

  .preset-item-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 7px 10px;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
  }

  .preset-item-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .preset-item-btn.selected {
    background: rgba(245, 158, 11, 0.12);
    border-color: rgba(245, 158, 11, 0.35);
  }

  .preset-icon {
    font-size: 16px;
  }

  .preset-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .preset-title-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .preset-name {
    font-size: 11px;
    font-weight: 600;
    color: #ffffff;
  }

  .preset-mins {
    font-size: 10px;
    color: #f59e0b;
    font-weight: 700;
  }

  .preset-desc {
    font-size: 9.5px;
    color: #94a3b8;
  }

  .custom-timer-form {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .slider-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .slider-label {
    font-size: 10.5px;
    color: #94a3b8;
  }

  .custom-slider {
    width: 100%;
    accent-color: #f59e0b;
    cursor: pointer;
  }

  .custom-input-row {
    display: flex;
    gap: 6px;
  }

  .label-input {
    flex: 1;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 6px;
    padding: 5px 8px;
    font-size: 11px;
    color: #ffffff;
    outline: none;
  }

  .label-input:focus {
    border-color: #f59e0b;
  }

  .custom-start-btn {
    background: rgba(245, 158, 11, 0.2);
    border: 1px solid rgba(245, 158, 11, 0.4);
    color: #fbbf24;
    font-size: 10.5px;
    font-weight: 600;
    border-radius: 6px;
    padding: 5px 10px;
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.15s ease;
  }

  .custom-start-btn:hover {
    background: #f59e0b;
    color: #000;
  }

  .rank-card {
    background: rgba(245, 158, 11, 0.05);
    border-color: rgba(245, 158, 11, 0.15);
  }

  .rank-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .rank-emoji {
    font-size: 24px;
  }

  .rank-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .rank-badge {
    font-size: 11px;
    font-weight: 700;
    color: #f59e0b;
  }

  .rank-desc {
    font-size: 9.5px;
    color: #94a3b8;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
    margin-top: 4px;
  }

  .stat-box {
    background: rgba(0, 0, 0, 0.25);
    border-radius: 6px;
    padding: 6px;
    text-align: center;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stat-number {
    font-size: 14px;
    font-weight: 800;
    color: #ffffff;
  }

  .stat-label {
    font-size: 9px;
    color: #94a3b8;
  }
</style>
