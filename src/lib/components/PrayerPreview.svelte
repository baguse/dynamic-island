<script lang="ts">
  import IslandHeader from './IslandHeader.svelte';
  import { islandState } from '../state/island.svelte';
  import type { PrayerTimeItem } from '../types';

  const schedule = $derived(islandState.prayerSchedule);
  const nextPrayer = $derived(islandState.nextPrayer);
  const currentPrayer = $derived(schedule?.current_prayer);
  const isLoading = $derived(islandState.isPrayerLoading);
  const settings = $derived(islandState.settings);

  let showSettingsModal = $state(false);
  let customCityInput = $state('');
  let isSavingSettings = $state(false);
  let audioTesting = $state(false);

  // Synchronize custom city input with current settings
  $effect(() => {
    if (settings.prayer_city) {
      customCityInput = settings.prayer_city;
    } else if (schedule?.city) {
      customCityInput = schedule.city;
    }
  });

  const CALCULATION_METHODS = [
    { id: 'KEMENAG', name: 'Kemenag RI (Indonesia)', desc: 'Fajr 20°, Isha 18°' },
    { id: 'MWL', name: 'Muslim World League', desc: 'Fajr 18°, Isha 17°' },
    { id: 'MAKKAH', name: 'Umm Al-Qura (Makkah)', desc: 'Fajr 18.5°, Isha +90m' },
    { id: 'EGYPT', name: 'Egyptian Authority', desc: 'Fajr 19.5°, Isha 17.5°' },
    { id: 'ISNA', name: 'ISNA (North America)', desc: 'Fajr 15°, Isha 15°' },
    { id: 'KARACHI', name: 'Univ. Islamic Sciences Karachi', desc: 'Fajr 18°, Isha 18°' },
  ];

  const AUDIO_MODES = [
    { id: 'chime', name: 'Melodic Chime', icon: '🔔', desc: 'Pleasant polyphonic bell chime' },
    { id: 'adhan_takbeer', name: 'Takbeer Motif', icon: '🕌', desc: 'Spiritual melodic adhan intro' },
    { id: 'beep', name: 'Soft Beep', icon: '🔉', desc: 'Subtle double notification tone' },
    { id: 'none', name: 'Silent / Banner Only', icon: '🔕', desc: 'No audio, banner notification only' },
  ];

  const PRE_REMINDER_OPTIONS = [
    { value: 0, label: 'At Exact Time' },
    { value: 5, label: '5 Minutes Before' },
    { value: 10, label: '10 Minutes Before' },
    { value: 15, label: '15 Minutes Before' },
  ];

  const POPULAR_CITIES = ['Surabaya', 'Jakarta', 'Bandung', 'Medan', 'Makkah', 'Madinah', 'Kuala Lumpur', 'London'];

  async function handleRefresh() {
    await islandState.refreshPrayerSchedule(true);
  }

  async function handleSaveSettings() {
    isSavingSettings = true;
    try {
      const updated = {
        ...settings,
        prayer_city: customCityInput.trim() ? customCityInput.trim() : null,
      };
      await islandState.saveSettings(updated);
      await islandState.refreshPrayerSchedule(true);
      showSettingsModal = false;
    } catch (err) {
      console.debug('Failed to save prayer settings:', err);
    } finally {
      isSavingSettings = false;
    }
  }

  async function handleSelectMethod(methodId: string) {
    const updated = {
      ...settings,
      prayer_calculation_method: methodId,
    };
    await islandState.saveSettings(updated);
    await islandState.refreshPrayerSchedule(true);
  }

  async function handleSelectAsr(school: 'standard' | 'hanafi') {
    const updated = {
      ...settings,
      prayer_asr_school: school,
    };
    await islandState.saveSettings(updated);
    await islandState.refreshPrayerSchedule(true);
  }

  async function handleSelectAudio(mode: 'chime' | 'beep' | 'adhan_takbeer' | 'system' | 'none') {
    const updated = {
      ...settings,
      prayer_audio_mode: mode,
    };
    await islandState.saveSettings(updated);
    islandState.playPrayerSound(mode);
  }

  async function handleSelectPreReminder(mins: number) {
    const updated = {
      ...settings,
      prayer_pre_reminder_mins: mins,
    };
    await islandState.saveSettings(updated);
  }

  async function handleToggleNotifications() {
    const updated = {
      ...settings,
      prayer_notifications_enabled: !settings.prayer_notifications_enabled,
    };
    await islandState.saveSettings(updated);
  }

  function handleTestAudio() {
    audioTesting = true;
    islandState.testPrayerAlert(nextPrayer?.name || 'Maghrib');
    setTimeout(() => {
      audioTesting = false;
    }, 2500);
  }

  function getPrayerIcon(name: string | undefined): string {
    switch (name) {
      case 'Imsak':
        return '🥣';
      case 'Fajr':
        return '🌄';
      case 'Sunrise':
        return '🌅';
      case 'Dhuhr':
        return '☀️';
      case 'Asr':
        return '🌤️';
      case 'Maghrib':
        return '🌇';
      case 'Isha':
        return '🌌';
      default:
        return '🕌';
    }
  }

  function getBearingCardinal(bearing: number): string {
    const directions = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW'];
    const index = Math.round(bearing / 45) % 8;
    return directions[index];
  }

  function handleKeydown(e: KeyboardEvent) {
    if (showSettingsModal) {
      if (e.key === 'Escape') {
        showSettingsModal = false;
        e.preventDefault();
      }
      return;
    }

    if (e.key === 'r' || e.key === 'R') {
      if (!isLoading) {
        handleRefresh();
      }
    } else if (e.key === 't' || e.key === 'T' || e.key === ' ') {
      handleTestAudio();
      e.preventDefault();
    } else if (e.key === 's' || e.key === 'S' || e.key === ',') {
      showSettingsModal = true;
      e.preventDefault();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="prayer-panel">
  <IslandHeader title="Islamic Prayer Reminder">
    <div class="header-hijri-badge" title="Islamic Hijri Date">
      <span class="crescent-icon" aria-hidden="true">🌙</span>
      <span class="hijri-text">{schedule?.date_hijri || 'Hijri Calendar'}</span>
    </div>
  </IslandHeader>

  <div class="prayer-body">
    <!-- Top Row: Hero Next Prayer Card & Qibla Compass -->
    <div class="prayer-top-grid">
      <!-- Hero Next Prayer Card -->
      <div class="hero-prayer-card" class:imminent={islandState.isPrayerImminent}>
        <!-- Ambient Decorative Glow -->
        <div class="hero-ambient-glow" aria-hidden="true"></div>

        <!-- Top Status Bar: Tag, Current Prayer & City Location -->
        <div class="hero-top-bar">
          <div class="hero-tag-group">
            <span class="hero-badge-tag">
              {#if islandState.isPrayerImminent}
                <span class="pulse-dot"></span> ⏳ Imminent Adhan
              {:else}
                <span class="star-icon">✦</span> Next Prayer
              {/if}
            </span>
            {#if currentPrayer}
              <span class="current-prayer-pill" title="Current active prayer window">
                <span class="current-dot"></span> Now: {currentPrayer}
              </span>
            {/if}
          </div>

          <div class="city-indicator" title="Calculation location">
            <span class="location-pin">📍</span>
            <span class="location-text">{schedule?.city || 'Local'}{#if schedule?.country}, {schedule.country}{/if}</span>
          </div>
        </div>

        <!-- Main Identity Centerpiece: Icon, Arabic & Roman Titles, Target Time -->
        <div class="hero-main-row">
          <div class="hero-prayer-ident">
            <div class="hero-icon-bubble" title={nextPrayer?.name}>
              <span class="hero-icon-glyph">{getPrayerIcon(nextPrayer?.name)}</span>
            </div>
            <div class="hero-name-stack">
              {#if nextPrayer?.arabic_name}
                <div class="hero-arabic-title" dir="rtl">{nextPrayer.arabic_name}</div>
              {/if}
              <div class="hero-en-row">
                <h2 class="hero-prayer-name">{nextPrayer?.name || 'Upcoming Prayer'}</h2>
                {#if nextPrayer?.label && nextPrayer.label !== nextPrayer.name}
                  <span class="hero-prayer-alias">({nextPrayer.label})</span>
                {/if}
              </div>
            </div>
          </div>

          <div class="hero-time-cluster">
            <span class="hero-time-caption">Adhan at</span>
            <div class="hero-time-display">
              {nextPrayer?.time || '--:--'}
            </div>
          </div>
        </div>

        <!-- Countdown Digit Blocks & Actions -->
        <div class="hero-bottom-section">
          <div class="hero-countdown-block">
            <span class="countdown-caption">Time Remaining</span>
            {#if nextPrayer && nextPrayer.seconds_remaining > 0}
              <div class="countdown-digit-row">
                <div class="digit-box">
                  <span class="digit-val">{String(nextPrayer.hours ?? 0).padStart(2, '0')}</span>
                  <span class="digit-lbl">hr</span>
                </div>
                <span class="digit-colon">:</span>
                <div class="digit-box">
                  <span class="digit-val">{String(nextPrayer.minutes ?? 0).padStart(2, '0')}</span>
                  <span class="digit-lbl">min</span>
                </div>
                <span class="digit-colon">:</span>
                <div class="digit-box highlight">
                  <span class="digit-val">{String(nextPrayer.seconds ?? 0).padStart(2, '0')}</span>
                  <span class="digit-lbl">sec</span>
                </div>
              </div>
            {:else}
              <div class="countdown-tomorrow-tag">
                {islandState.nextPrayerCountdownFormatted}
              </div>
            {/if}
          </div>

          <div class="hero-actions">
            <button
              type="button"
              class="hero-action-btn audio-btn"
              onclick={handleTestAudio}
              class:testing={audioTesting}
              title="Test Prayer Notification & Audio Chime (Shortcut: T or Space)"
            >
              <span class="btn-icon">{audioTesting ? '🔔' : '🔊'}</span>
              <span>{audioTesting ? 'Playing...' : 'Test Adhan'}</span>
              <kbd class="action-kbd">T</kbd>
            </button>
            <button
              type="button"
              class="hero-action-btn settings-btn"
              onclick={() => (showSettingsModal = !showSettingsModal)}
              title="Configure Prayer Times & Alerts (Shortcut: S)"
            >
              <span class="btn-icon">⚙️</span>
              <span>Settings</span>
              <kbd class="action-kbd">S</kbd>
            </button>
          </div>
        </div>
      </div>

      <!-- Qibla Compass Card -->
      <div class="qibla-card">
        <div class="qibla-header">
          <span class="qibla-title">🕋 Qibla Direction</span>
          <span class="qibla-deg">{schedule ? `${schedule.qibla_bearing}° ${getBearingCardinal(schedule.qibla_bearing)}` : '--°'}</span>
        </div>

        <div class="compass-dial-container">
          <div
            class="compass-pointer"
            style="transform: rotate({schedule?.qibla_bearing || 0}deg);"
            title="Direction towards Kaaba, Makkah ({schedule?.qibla_bearing || 0}° from True North)"
          >
            <div class="pointer-kaaba-icon">🕋</div>
            <div class="pointer-arrow"></div>
          </div>
          <div class="compass-cardinals">
            <span class="cardinal n">N</span>
            <span class="cardinal e">E</span>
            <span class="cardinal s">S</span>
            <span class="cardinal w">W</span>
          </div>
        </div>
        <span class="qibla-footnote">From true North towards Makkah</span>
      </div>
    </div>

    <!-- Daily Prayer Times List -->
    <div class="schedule-section">
      <div class="schedule-section-header">
        <span class="section-heading">Daily Schedule ({schedule?.date_gregorian || 'Today'})</span>
        <div class="section-actions">
          <button
            type="button"
            class="refresh-icon-btn"
            onclick={handleRefresh}
            disabled={isLoading}
            title="Refresh prayer times (Shortcut: R)"
          >
            <span class="refresh-icon" class:spinning={isLoading}>⟳</span>
            <span>Refresh</span>
            <kbd class="action-kbd small">R</kbd>
          </button>
        </div>
      </div>

      <div class="prayer-cards-grid">
        {#if schedule?.items}
          {#each schedule.items as item (item.name)}
            <div
              class="prayer-item-card"
              class:is-next={item.is_next}
              class:is-current={item.is_current}
              class:is-passed={item.is_passed && !item.is_current}
            >
              <div class="card-left">
                <div class="prayer-icon-wrap">
                  {#if item.name === 'Imsak'}
                    <span class="prayer-icon">🥣</span>
                  {:else if item.name === 'Fajr'}
                    <span class="prayer-icon">🌄</span>
                  {:else if item.name === 'Sunrise'}
                    <span class="prayer-icon">🌅</span>
                  {:else if item.name === 'Dhuhr'}
                    <span class="prayer-icon">☀️</span>
                  {:else if item.name === 'Asr'}
                    <span class="prayer-icon">🌤️</span>
                  {:else if item.name === 'Maghrib'}
                    <span class="prayer-icon">🌇</span>
                  {:else}
                    <span class="prayer-icon">🌌</span>
                  {/if}
                </div>
                <div class="prayer-names">
                  <span class="prayer-en-name">{item.label}</span>
                  <span class="prayer-ar-name">{item.arabic_name}</span>
                </div>
              </div>

              <div class="card-right">
                <span class="prayer-time-val">{item.time}</span>
                {#if item.is_next}
                  <span class="status-pill status-next">Next Up</span>
                {:else if item.is_current}
                  <span class="status-pill status-current">Current</span>
                {:else if item.is_passed}
                  <span class="status-pill status-passed">Passed</span>
                {:else}
                  <span class="status-pill status-upcoming">Upcoming</span>
                {/if}
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>

    <!-- Slide-in Settings Drawer / Modal -->
    {#if showSettingsModal}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div class="settings-drawer-backdrop" onclick={() => (showSettingsModal = false)} role="presentation">
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          class="settings-drawer"
          onclick={(e) => e.stopPropagation()}
          role="dialog"
          aria-modal="true"
          tabindex="-1"
        >
          <div class="drawer-header">
            <h3>Prayer Reminder Settings</h3>
            <button
              type="button"
              class="drawer-close-btn"
              onclick={() => (showSettingsModal = false)}
              aria-label="Close"
            >
              ✕
            </button>
          </div>

          <div class="drawer-content">
            <!-- Location Section -->
            <div class="settings-group">
              <label class="group-title" for="city-input">City Location</label>
              <div class="city-input-row">
                <input
                  id="city-input"
                  type="text"
                  bind:value={customCityInput}
                  placeholder="e.g. Surabaya, Jakarta, London..."
                  class="city-text-input"
                />
                <button
                  type="button"
                  class="save-city-btn"
                  onclick={handleSaveSettings}
                  disabled={isSavingSettings}
                >
                  {isSavingSettings ? 'Saving...' : 'Apply City'}
                </button>
              </div>
              <div class="popular-chips">
                {#each POPULAR_CITIES as c}
                  <button
                    type="button"
                    class="chip-btn"
                    class:active={customCityInput.toLowerCase() === c.toLowerCase()}
                    onclick={() => {
                      customCityInput = c;
                      handleSaveSettings();
                    }}
                  >
                    {c}
                  </button>
                {/each}
              </div>
            </div>

            <!-- Calculation Method -->
            <div class="settings-group">
              <span class="group-title">Calculation Convention</span>
              <div class="options-grid">
                {#each CALCULATION_METHODS as m}
                  <button
                    type="button"
                    class="option-card"
                    class:active={(settings.prayer_calculation_method || 'KEMENAG') === m.id}
                    onclick={() => handleSelectMethod(m.id)}
                  >
                    <div class="option-name">{m.name}</div>
                    <div class="option-desc">{m.desc}</div>
                  </button>
                {/each}
              </div>
            </div>

            <!-- Asr Jurisprudence -->
            <div class="settings-group">
              <span class="group-title">Asr Juristic Method</span>
              <div class="toggle-row">
                <button
                  type="button"
                  class="toggle-choice-btn"
                  class:active={(settings.prayer_asr_school || 'standard') === 'standard'}
                  onclick={() => handleSelectAsr('standard')}
                >
                  Standard (Shafi'i, Maliki, Hanbali)
                </button>
                <button
                  type="button"
                  class="toggle-choice-btn"
                  class:active={settings.prayer_asr_school === 'hanafi'}
                  onclick={() => handleSelectAsr('hanafi')}
                >
                  Hanafi (Shadow factor 2)
                </button>
              </div>
            </div>

            <!-- Pre-Adhan Reminder Warning -->
            <div class="settings-group">
              <span class="group-title">Pre-Adhan Reminder Alert</span>
              <div class="pill-options-row">
                {#each PRE_REMINDER_OPTIONS as opt}
                  <button
                    type="button"
                    class="pre-pill-btn"
                    class:active={(settings.prayer_pre_reminder_mins ?? 10) === opt.value}
                    onclick={() => handleSelectPreReminder(opt.value)}
                  >
                    {opt.label}
                  </button>
                {/each}
              </div>
            </div>

            <!-- Audio Alert Mode -->
            <div class="settings-group">
              <span class="group-title">Audio Notification Tone</span>
              <div class="audio-options-list">
                {#each AUDIO_MODES as a}
                  <button
                    type="button"
                    class="audio-option-item"
                    class:active={(settings.prayer_audio_mode || 'chime') === a.id}
                    onclick={() => handleSelectAudio(a.id as 'chime' | 'beep' | 'adhan_takbeer' | 'system' | 'none')}
                  >
                    <span class="audio-icon">{a.icon}</span>
                    <div class="audio-meta">
                      <span class="audio-name">{a.name}</span>
                      <span class="audio-desc">{a.desc}</span>
                    </div>
                  </button>
                {/each}
              </div>
            </div>

            <!-- Notification Toggle -->
            <div class="settings-group toggle-group">
              <div class="toggle-info">
                <span class="toggle-title">Show Desktop Notification Banner</span>
                <span class="toggle-sub">Pops a banner on the Dynamic Island at prayer time</span>
              </div>
              <button
                type="button"
                class="switch-btn"
                class:active={settings.prayer_notifications_enabled !== false}
                onclick={handleToggleNotifications}
                aria-pressed={settings.prayer_notifications_enabled !== false}
                aria-label="Toggle Desktop Notification Banner"
              >
                <span class="switch-thumb"></span>
              </button>
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .prayer-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    padding: 0 4px 6px;
    box-sizing: border-box;
    overflow-y: auto;
  }

  .header-hijri-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px;
    border-radius: 9999px;
    background: rgba(16, 185, 129, 0.12);
    border: 1px solid rgba(16, 185, 129, 0.25);
    color: #10b981;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.3px;
  }

  .crescent-icon {
    font-size: 11px;
  }

  .prayer-body {
    display: flex;
    flex-direction: column;
    gap: 16px;
    margin-top: 12px;
  }

  /* Top Grid: Hero Card + Qibla */
  .prayer-top-grid {
    display: grid;
    grid-template-columns: 1fr 190px;
    gap: 12px;
  }

  .hero-prayer-card {
    background: linear-gradient(135deg, rgba(13, 148, 136, 0.22) 0%, rgba(6, 78, 59, 0.45) 50%, rgba(15, 23, 42, 0.75) 100%);
    border: 1px solid rgba(45, 212, 191, 0.35);
    border-radius: 16px;
    padding: 16px 18px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 14px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.1);
    position: relative;
    overflow: hidden;
  }

  .hero-ambient-glow {
    position: absolute;
    top: -40px;
    right: -40px;
    width: 140px;
    height: 140px;
    background: radial-gradient(circle, rgba(45, 212, 191, 0.25) 0%, transparent 70%);
    border-radius: 9999px;
    pointer-events: none;
  }

  .hero-prayer-card.imminent {
    border-color: rgba(245, 158, 11, 0.7);
    background: linear-gradient(135deg, rgba(245, 158, 11, 0.25) 0%, rgba(180, 83, 9, 0.4) 50%, rgba(15, 23, 42, 0.8) 100%);
    box-shadow: 0 8px 32px rgba(245, 158, 11, 0.25), inset 0 1px 0 rgba(255, 255, 255, 0.15);
  }

  .hero-prayer-card.imminent .hero-ambient-glow {
    background: radial-gradient(circle, rgba(245, 158, 11, 0.3) 0%, transparent 70%);
  }

  /* Top Bar */
  .hero-top-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    position: relative;
    z-index: 1;
  }

  .hero-tag-group {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .hero-badge-tag {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: #2dd4bf;
    background: rgba(45, 212, 191, 0.18);
    border: 1px solid rgba(45, 212, 191, 0.3);
    padding: 3px 8px;
    border-radius: 9999px;
  }

  .hero-prayer-card.imminent .hero-badge-tag {
    color: #fbbf24;
    background: rgba(251, 191, 36, 0.2);
    border-color: rgba(251, 191, 36, 0.4);
  }

  .pulse-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #fbbf24;
    box-shadow: 0 0 8px #fbbf24;
    animation: pulse-glow 1.5s infinite;
  }

  @keyframes pulse-glow {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.4; transform: scale(0.85); }
  }

  .star-icon {
    font-size: 9px;
    color: #5eead4;
  }

  .current-prayer-pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 10px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.8);
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    padding: 3px 8px;
    border-radius: 9999px;
  }

  .current-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: #10b981;
    box-shadow: 0 0 6px #10b981;
  }

  .city-indicator {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: rgba(255, 255, 255, 0.7);
    font-weight: 500;
  }

  .location-pin {
    font-size: 11px;
  }

  .location-text {
    max-width: 140px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* Main Row */
  .hero-main-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    position: relative;
    z-index: 1;
    gap: 16px;
  }

  .hero-prayer-ident {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .hero-icon-bubble {
    width: 48px;
    height: 48px;
    border-radius: 14px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }

  .hero-icon-glyph {
    font-size: 24px;
  }

  .hero-name-stack {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .hero-arabic-title {
    font-size: 15px;
    font-weight: 700;
    color: #5eead4;
    letter-spacing: 0.5px;
    opacity: 0.9;
    font-family: 'Amiri', 'Traditional Arabic', 'Scheherazade New', serif;
  }

  .hero-prayer-card.imminent .hero-arabic-title {
    color: #fcd34d;
  }

  .hero-en-row {
    display: flex;
    align-items: baseline;
    gap: 6px;
  }

  .hero-prayer-name {
    margin: 0;
    font-size: 24px;
    font-weight: 800;
    letter-spacing: -0.4px;
    color: #f8fafc;
    line-height: 1.1;
  }

  .hero-prayer-alias {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.55);
    font-weight: 500;
  }

  .hero-time-cluster {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    flex-shrink: 0;
  }

  .hero-time-caption {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: rgba(255, 255, 255, 0.5);
  }

  .hero-time-display {
    font-size: 32px;
    font-weight: 900;
    font-variant-numeric: tabular-nums;
    color: #5eead4;
    letter-spacing: -0.5px;
    text-shadow: 0 0 18px rgba(94, 234, 212, 0.4);
    line-height: 1.1;
  }

  .hero-prayer-card.imminent .hero-time-display {
    color: #fcd34d;
    text-shadow: 0 0 18px rgba(252, 211, 77, 0.45);
  }

  /* Bottom Section */
  .hero-bottom-section {
    padding-top: 12px;
    border-top: 1px solid rgba(255, 255, 255, 0.09);
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    position: relative;
    z-index: 1;
    gap: 12px;
  }

  .hero-countdown-block {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .countdown-caption {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: rgba(255, 255, 255, 0.5);
  }

  .countdown-digit-row {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .digit-box {
    background: rgba(15, 23, 42, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 7px;
    padding: 3px 6px;
    display: inline-flex;
    align-items: baseline;
    gap: 2px;
  }

  .digit-box.highlight {
    border-color: rgba(45, 212, 191, 0.35);
  }

  .hero-prayer-card.imminent .digit-box.highlight {
    border-color: rgba(245, 158, 11, 0.45);
  }

  .digit-val {
    font-size: 16px;
    font-weight: 800;
    font-variant-numeric: tabular-nums;
    color: #f8fafc;
  }

  .digit-box.highlight .digit-val {
    color: #5eead4;
  }

  .hero-prayer-card.imminent .digit-box.highlight .digit-val {
    color: #fcd34d;
  }

  .digit-lbl {
    font-size: 9px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.45);
    text-transform: uppercase;
  }

  .digit-colon {
    font-size: 14px;
    font-weight: 700;
    color: rgba(255, 255, 255, 0.3);
  }

  .countdown-tomorrow-tag {
    font-size: 13px;
    font-weight: 700;
    color: #5eead4;
    background: rgba(45, 212, 191, 0.12);
    padding: 3px 8px;
    border-radius: 6px;
    border: 1px solid rgba(45, 212, 191, 0.25);
  }

  .hero-actions {
    display: flex;
    gap: 8px;
  }

  .hero-action-btn {
    border: none;
    border-radius: 8px;
    padding: 6px 11px;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    transition: all 0.15s ease;
  }

  .hero-action-btn .btn-icon {
    font-size: 12px;
  }

  .hero-action-btn.audio-btn {
    background: rgba(45, 212, 191, 0.18);
    color: #2dd4bf;
    border: 1px solid rgba(45, 212, 191, 0.35);
  }

  .hero-action-btn.audio-btn:hover {
    background: rgba(45, 212, 191, 0.3);
    color: #ffffff;
  }

  .hero-action-btn.audio-btn.testing {
    background: #14b8a6;
    color: #042f2e;
  }

  .hero-action-btn.settings-btn {
    background: rgba(255, 255, 255, 0.08);
    color: rgba(255, 255, 255, 0.85);
    border: 1px solid rgba(255, 255, 255, 0.14);
  }

  .hero-action-btn.settings-btn:hover {
    background: rgba(255, 255, 255, 0.16);
    color: #ffffff;
  }

  .action-kbd {
    display: inline-block;
    padding: 1px 5px;
    font-size: 9px;
    font-weight: 700;
    font-family: inherit;
    line-height: 1.2;
    color: rgba(255, 255, 255, 0.6);
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 4px;
    margin-left: 3px;
  }

  .action-kbd.small {
    padding: 0 4px;
    font-size: 8px;
  }

  /* Qibla Card */
  .qibla-card {
    background: rgba(15, 23, 42, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    padding: 14px 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: space-between;
    text-align: center;
  }

  .qibla-header {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .qibla-title {
    font-size: 12px;
    font-weight: 700;
    color: #e2e8f0;
  }

  .qibla-deg {
    font-size: 12px;
    font-weight: 800;
    color: #38bdf8;
  }

  .compass-dial-container {
    width: 90px;
    height: 90px;
    border-radius: 50%;
    border: 2px dashed rgba(255, 255, 255, 0.15);
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 8px 0;
  }

  .compass-cardinals {
    position: absolute;
    inset: 2px;
    pointer-events: none;
  }

  .cardinal {
    position: absolute;
    font-size: 9px;
    font-weight: 800;
    color: rgba(255, 255, 255, 0.4);
  }

  .cardinal.n { top: 1px; left: 50%; transform: translateX(-50%); color: #ef4444; }
  .cardinal.s { bottom: 1px; left: 50%; transform: translateX(-50%); }
  .cardinal.e { right: 2px; top: 50%; transform: translateY(-50%); }
  .cardinal.w { left: 2px; top: 50%; transform: translateY(-50%); }

  .compass-pointer {
    width: 100%;
    height: 100%;
    position: absolute;
    display: flex;
    flex-direction: column;
    align-items: center;
    transition: transform 0.6s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  .pointer-kaaba-icon {
    font-size: 14px;
    transform: translateY(-8px);
  }

  .pointer-arrow {
    width: 3px;
    height: 38px;
    background: linear-gradient(to top, transparent 0%, #38bdf8 100%);
    border-radius: 2px;
  }

  .qibla-footnote {
    font-size: 10px;
    color: rgba(255, 255, 255, 0.4);
  }

  /* Schedule Section */
  .schedule-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .schedule-section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 2px;
  }

  .section-heading {
    font-size: 13px;
    font-weight: 700;
    color: rgba(255, 255, 255, 0.7);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .refresh-icon-btn {
    background: transparent;
    border: none;
    color: rgba(255, 255, 255, 0.6);
    font-size: 12px;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 6px;
    transition: all 0.15s;
  }

  .refresh-icon-btn:hover {
    color: #f8fafc;
    background: rgba(255, 255, 255, 0.08);
  }

  .refresh-icon.spinning {
    display: inline-block;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .prayer-cards-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
    gap: 8px;
  }

  .prayer-item-card {
    background: rgba(30, 41, 59, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 10px;
    padding: 10px 14px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    transition: all 0.2s ease;
  }

  .prayer-item-card:hover {
    background: rgba(30, 41, 59, 0.7);
    border-color: rgba(255, 255, 255, 0.12);
  }

  .prayer-item-card.is-next {
    background: rgba(20, 184, 166, 0.15);
    border-color: rgba(45, 212, 191, 0.4);
    box-shadow: 0 0 12px rgba(45, 212, 191, 0.15);
  }

  .prayer-item-card.is-current {
    background: rgba(16, 185, 129, 0.12);
    border-color: rgba(16, 185, 129, 0.35);
  }

  .prayer-item-card.is-passed {
    opacity: 0.55;
  }

  .card-left {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .prayer-icon-wrap {
    font-size: 18px;
  }

  .prayer-names {
    display: flex;
    flex-direction: column;
  }

  .prayer-en-name {
    font-size: 13px;
    font-weight: 700;
    color: #f1f5f9;
  }

  .prayer-ar-name {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.45);
    font-family: serif;
  }

  .card-right {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 3px;
  }

  .prayer-time-val {
    font-size: 15px;
    font-weight: 800;
    font-variant-numeric: tabular-nums;
    color: #f8fafc;
  }

  .prayer-item-card.is-next .prayer-time-val {
    color: #2dd4bf;
  }

  .status-pill {
    font-size: 9px;
    font-weight: 700;
    padding: 1px 6px;
    border-radius: 9999px;
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .status-next {
    background: #2dd4bf;
    color: #042f2e;
  }

  .status-current {
    background: #10b981;
    color: #064e3b;
  }

  .status-passed {
    background: rgba(255, 255, 255, 0.08);
    color: rgba(255, 255, 255, 0.4);
  }

  .status-upcoming {
    background: rgba(255, 255, 255, 0.08);
    color: rgba(255, 255, 255, 0.6);
  }

  /* Slide-in Settings Drawer */
  .settings-drawer-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(4px);
    z-index: 9999;
    display: flex;
    justify-content: flex-end;
  }

  .settings-drawer {
    width: 380px;
    height: 100%;
    background: #0f172a;
    border-left: 1px solid rgba(255, 255, 255, 0.12);
    display: flex;
    flex-direction: column;
    padding: 18px;
    box-sizing: border-box;
    box-shadow: -8px 0 32px rgba(0, 0, 0, 0.5);
    overflow-y: auto;
  }

  .drawer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    padding-bottom: 12px;
    margin-bottom: 16px;
  }

  .drawer-header h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 800;
    color: #f8fafc;
  }

  .drawer-close-btn {
    background: transparent;
    border: none;
    color: rgba(255, 255, 255, 0.5);
    font-size: 16px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 6px;
  }

  .drawer-close-btn:hover {
    color: #f8fafc;
    background: rgba(255, 255, 255, 0.1);
  }

  .drawer-content {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .settings-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .group-title {
    font-size: 12px;
    font-weight: 700;
    color: rgba(255, 255, 255, 0.7);
    text-transform: uppercase;
    letter-spacing: 0.4px;
  }

  .city-input-row {
    display: flex;
    gap: 8px;
  }

  .city-text-input {
    flex: 1;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 8px;
    padding: 8px 12px;
    color: #f8fafc;
    font-size: 13px;
    outline: none;
  }

  .city-text-input:focus {
    border-color: #2dd4bf;
    background: rgba(255, 255, 255, 0.1);
  }

  .save-city-btn {
    background: #14b8a6;
    color: #042f2e;
    border: none;
    border-radius: 8px;
    padding: 0 14px;
    font-size: 12px;
    font-weight: 700;
    cursor: pointer;
    transition: opacity 0.15s;
  }

  .save-city-btn:hover {
    opacity: 0.9;
  }

  .popular-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 4px;
  }

  .chip-btn {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: rgba(255, 255, 255, 0.7);
    border-radius: 9999px;
    padding: 3px 10px;
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .chip-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #f8fafc;
  }

  .chip-btn.active {
    background: rgba(45, 212, 191, 0.2);
    border-color: #2dd4bf;
    color: #2dd4bf;
  }

  .options-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 6px;
  }

  .option-card {
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 8px 12px;
    text-align: left;
    cursor: pointer;
    transition: all 0.15s;
  }

  .option-card:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .option-card.active {
    background: rgba(45, 212, 191, 0.15);
    border-color: #2dd4bf;
  }

  .option-name {
    font-size: 12px;
    font-weight: 700;
    color: #f1f5f9;
  }

  .option-desc {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.45);
  }

  .toggle-row {
    display: flex;
    gap: 8px;
  }

  .toggle-choice-btn {
    flex: 1;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 8px 10px;
    color: rgba(255, 255, 255, 0.7);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .toggle-choice-btn.active {
    background: rgba(45, 212, 191, 0.2);
    border-color: #2dd4bf;
    color: #2dd4bf;
  }

  .pill-options-row {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 6px;
  }

  .pre-pill-btn {
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 7px 10px;
    color: rgba(255, 255, 255, 0.7);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .pre-pill-btn.active {
    background: rgba(45, 212, 191, 0.2);
    border-color: #2dd4bf;
    color: #2dd4bf;
  }

  .audio-options-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .audio-option-item {
    display: flex;
    align-items: center;
    gap: 12px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 8px 12px;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s;
  }

  .audio-option-item:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .audio-option-item.active {
    background: rgba(45, 212, 191, 0.15);
    border-color: #2dd4bf;
  }

  .audio-icon {
    font-size: 18px;
  }

  .audio-meta {
    display: flex;
    flex-direction: column;
  }

  .audio-name {
    font-size: 12px;
    font-weight: 700;
    color: #f1f5f9;
  }

  .audio-desc {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.45);
  }

  .toggle-group {
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    align-items: center;
    padding: 10px 0;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
  }

  .toggle-info {
    display: flex;
    flex-direction: column;
  }

  .toggle-title {
    font-size: 12px;
    font-weight: 700;
    color: #f1f5f9;
  }

  .toggle-sub {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.45);
  }

  .switch-btn {
    width: 44px;
    height: 24px;
    border-radius: 9999px;
    background: rgba(255, 255, 255, 0.15);
    border: none;
    cursor: pointer;
    position: relative;
    padding: 2px;
    transition: background 0.2s ease;
  }

  .switch-btn.active {
    background: #14b8a6;
  }

  .switch-thumb {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #fff;
    display: block;
    transition: transform 0.2s ease;
  }

  .switch-btn.active .switch-thumb {
    transform: translateX(20px);
  }
</style>
