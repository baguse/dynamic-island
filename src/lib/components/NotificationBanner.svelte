<script lang="ts">
  import { islandState } from '../state/island.svelte';

  const notif = $derived(islandState.activeNotification);

  function handleBannerClick() {
    if (notif?.id?.startsWith('agent-review') || islandState.agent.state === 'review') {
      islandState.openAgent();
    } else {
      islandState.dismissNotification();
    }
  }

  function handleDismiss(e?: MouseEvent) {
    if (e) e.stopPropagation();
    islandState.dismissNotification();
  }
</script>

<div class="notification-banner" role="status" aria-live="polite">
  <div class="banner-body">
    <button
      type="button"
      class="banner-content-btn"
      onclick={handleBannerClick}
      aria-label="Notification: {notif?.title || 'Alert'}."
    >
      <div
        class="icon-container"
        class:terminal={notif?.icon === 'terminal'}
        class:system={notif?.icon === 'system'}
      >
        {#if notif?.icon === 'terminal'}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="4 17 10 11 4 5"></polyline>
            <line x1="12" y1="19" x2="20" y2="19"></line>
          </svg>
        {:else if notif?.icon === 'system'}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"></path>
            <path d="M13.73 21a2 2 0 0 1-3.46 0"></path>
          </svg>
        {:else}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
          </svg>
        {/if}
      </div>

      <div class="text-content">
        <div class="app-row">
          <span class="app-name">{notif?.app_name || 'Notification'}</span>
          <span class="now-badge">Just now</span>
        </div>
        <h4 class="notif-title">{notif?.title || 'Alert'}</h4>
        <p class="notif-message">{notif?.message || ''}</p>
      </div>
    </button>

    <button
      type="button"
      class="dismiss-btn"
      onclick={(e) => {
        e.stopPropagation();
        handleDismiss();
      }}
      aria-label="Dismiss notification"
      title="Dismiss"
    >
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <line x1="18" y1="6" x2="6" y2="18"></line>
        <line x1="6" y1="6" x2="18" y2="18"></line>
      </svg>
    </button>
  </div>

  <!-- Countdown Progress Bar -->
  <div class="progress-track" aria-hidden="true">
    <div
      class="progress-fill"
      style="animation-duration: {notif?.duration_ms || 4500}ms;"
    ></div>
  </div>
</div>

<style>
  .notification-banner {
    width: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    position: relative;
    overflow: hidden;
  }

  .banner-body {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 2px 2px 8px 2px;
    gap: 8px;
  }

  .banner-content-btn {
    display: flex;
    align-items: center;
    gap: 12px;
    background: none;
    border: none;
    padding: 0;
    margin: 0;
    color: inherit;
    font-family: inherit;
    cursor: pointer;
    flex: 1;
    overflow: hidden;
    text-align: left;
  }

  .icon-container {
    width: 36px;
    height: 36px;
    border-radius: 10px;
    background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
    color: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    box-shadow: 0 4px 12px rgba(59, 130, 246, 0.4);
  }

  .icon-container.terminal {
    background: linear-gradient(135deg, #10b981 0%, #059669 100%);
    box-shadow: 0 4px 12px rgba(16, 185, 129, 0.4);
  }

  .icon-container.system {
    background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
    box-shadow: 0 4px 12px rgba(245, 158, 11, 0.4);
  }

  .text-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    overflow: hidden;
  }

  .app-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .app-name {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--island-text-muted, #9ca3af);
  }

  .now-badge {
    font-size: 9px;
    color: #6b7280;
  }

  .notif-title {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--island-text-primary, #ffffff);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .notif-message {
    margin: 0;
    font-size: 11px;
    color: var(--island-text-secondary, #9ca3af);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dismiss-btn {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 1px solid var(--island-card-border, rgba(255, 255, 255, 0.08));
    background: var(--island-card-bg, rgba(255, 255, 255, 0.08));
    color: var(--island-text-secondary, #9ca3af);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    flex-shrink: 0;
    transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
  }

  .dismiss-btn:hover {
    background: var(--island-card-hover, rgba(255, 255, 255, 0.16));
    border-color: var(--island-border-hover, rgba(255, 255, 255, 0.25));
    color: var(--island-text-primary, #ffffff);
  }

  .progress-track {
    width: 100%;
    height: 2px;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 1px;
    overflow: hidden;
    margin-top: 4px;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #3b82f6, #60a5fa);
    width: 100%;
    transform-origin: left;
    animation: countdown linear forwards;
  }

  @keyframes countdown {
    from {
      transform: scaleX(1);
    }
    to {
      transform: scaleX(0);
    }
  }
</style>
