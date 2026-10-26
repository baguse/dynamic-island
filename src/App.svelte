<script lang="ts">
  import Island from './lib/components/Island.svelte';
  import { islandState } from './lib/state/island.svelte';
  import { onMount, onDestroy } from 'svelte';

  $effect(() => {
    const theme = islandState.settings.theme || 'dark';
    document.documentElement.setAttribute('data-theme', theme);
  });

  onMount(() => {
    // Initial sync with native window positioning
    islandState.syncNativeWindow();
    // Initialize background MPRIS and notification services
    islandState.initServices();
  });

  onDestroy(() => {
    islandState.destroyServices();
  });
</script>

<main class="island-viewport">
  <Island />
</main>

<style>
  .island-viewport {
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    background: transparent;
    box-sizing: border-box;
  }
</style>
