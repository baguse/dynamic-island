<script lang="ts">
  import { islandState } from '../state/island.svelte';
  import type { MascotState, MascotType } from '../types';

  interface Props {
    size?: 'sm' | 'md' | 'lg';
    stateOverride?: MascotState;
    mascotTypeOverride?: MascotType;
    showTooltip?: boolean;
  }

  let { size = 'sm', stateOverride, mascotTypeOverride, showTooltip = true }: Props = $props();

  const settings = $derived(islandState.settings);
  const isPlaying = $derived(islandState.isPlaying);
  const hasActiveBuild = $derived(islandState.hasActiveBuild);
  const hasActiveAgent = $derived(islandState.hasActiveAgent);
  const isBatteryLow = $derived(islandState.isBatteryLow);
  const telemetry = $derived(islandState.telemetry);

  const activeMascot = $derived<MascotType>(mascotTypeOverride ?? (settings.mascot_type as MascotType) ?? 'cat');

  let isPetting = $state(false);
  let petTimer: ReturnType<typeof setTimeout> | null = null;
  let heartBursts = $state<Array<{ id: number; x: number; y: number }>>([]);
  let burstCounter = 0;

  // Derive current reactive mascot mood
  const currentMood = $derived.by<MascotState>(() => {
    if (stateOverride) return stateOverride;
    if (isPetting) return 'petting';
    if (hasActiveBuild || (telemetry && telemetry.cpu_percent > 55)) return 'typing';
    if (hasActiveAgent) return 'agent';
    if (isPlaying) return 'music';
    if (isBatteryLow) return 'alert';
    return 'sleeping';
  });

  const moodLabel = $derived.by<string>(() => {
    switch (activeMascot) {
      case 'duck':
        switch (currentMood) {
          case 'petting': return 'Happy Quack! (Petted ♥)';
          case 'typing': return 'Frantically coding with wings! ⚡';
          case 'agent': return 'Detective Duck inspecting with cyber-visor... 🤖';
          case 'music': return 'Vibing with cool sunglasses! 🦆🕶️';
          case 'alert': return 'Alert! Low battery or system errors! 🚨';
          case 'sleeping': return 'Floating and snoozing peacefully (Zzz)... 😴';
        }
      case 'robot':
        switch (currentMood) {
          case 'petting': return 'Beep Boop! Heart LEDs overload! [♥_♥]';
          case 'typing': return 'Overclocked turbo compilation active! ⚡';
          case 'agent': return 'Deep neural network scan in progress... 🤖';
          case 'music': return 'Equalizer synth dance mode engaged! 📻';
          case 'alert': return 'Warning! System threshold exceeded! 🚨';
          case 'sleeping': return 'Standby & power-saving mode (010)... 🔋';
        }
      case 'fox':
        switch (currentMood) {
          case 'petting': return 'Joyful Kitsune yip! Tail wagging! ♥';
          case 'typing': return 'Bongo Fox rapid paw typing! ⚡';
          case 'agent': return 'Channeling cyber spirit wisps... 🦊';
          case 'music': return 'Dancing with fluffy tail swishing! 🎶';
          case 'alert': return 'Alert! Perked ears & startled fox! 🚨';
          case 'sleeping': return 'Curled up under fluffy warm tail (Zzz)... 😴';
        }
      default: // cat
        switch (currentMood) {
          case 'petting': return 'Purring happily! (Petted ♥)';
          case 'typing': return 'Typing code furiously on laptop! ⚡';
          case 'agent': return 'Thinking & inspecting with cyber-visor... 🤖';
          case 'music': return 'Jamming to the beat with neon headphones! 🎧';
          case 'alert': return 'Alert! Low battery or system errors! 🚨';
          case 'sleeping': return 'Napping peacefully (Zzz)... 😴';
        }
    }
  });

  const mascotName = $derived.by<string>(() => {
    switch (activeMascot) {
      case 'duck': return 'Chill Duck';
      case 'robot': return 'Retro Bot';
      case 'fox': return 'Kitsune Fox';
      default: return 'Cyber-Cat';
    }
  });

  function handlePet(e: MouseEvent) {
    if (settings.mascot_interactive === false) return;
    e.stopPropagation();

    isPetting = true;
    const burstId = burstCounter++;
    heartBursts = [...heartBursts, { id: burstId, x: Math.random() * 12 - 6, y: -8 }];

    setTimeout(() => {
      heartBursts = heartBursts.filter(h => h.id !== burstId);
    }, 1200);

    if (petTimer) clearTimeout(petTimer);
    petTimer = setTimeout(() => {
      isPetting = false;
    }, 2200);
  }
</script>

{#if settings.mascot_enabled !== false}
  <button
    type="button"
    class="pixel-mascot-container size-{size} mood-{currentMood} mascot-{activeMascot}"
    onclick={handlePet}
    title={showTooltip ? `${mascotName}: ${moodLabel} • Click to pet!` : undefined}
    aria-label="Desktop {mascotName} companion"
  >
    <!-- Floating Petting Hearts Burst -->
    {#each heartBursts as burst (burst.id)}
      <span class="pixel-particle pixel-heart" style="--burst-x: {burst.x}px">♥</span>
    {/each}

    <!-- Mood 1: Sleeping Zzz Particles -->
    {#if currentMood === 'sleeping'}
      <span class="pixel-particle pixel-z z-1">z</span>
      <span class="pixel-particle pixel-z z-2">Z</span>
      <span class="pixel-particle pixel-z z-3">z</span>
    {/if}

    <!-- Mood 2: Music Notes Particles -->
    {#if currentMood === 'music'}
      <span class="pixel-particle music-note note-1">♪</span>
      <span class="pixel-particle music-note note-2">♫</span>
    {/if}

    <!-- Mood 3: Typing Sparks Particles -->
    {#if currentMood === 'typing'}
      <span class="pixel-particle spark spark-1">⚡</span>
      <span class="pixel-particle spark spark-2">✦</span>
    {/if}

    <!-- Mood 4: AI Agent Thought Gear Particle -->
    {#if currentMood === 'agent'}
      <span class="pixel-particle agent-bubble">💡</span>
    {/if}

    <!-- SVG Pixel Graphics (ViewBox 0 0 28 28) -->
    <svg
      class="mascot-svg"
      viewBox="0 0 28 28"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      shape-rendering="crispEdges"
    >
      <!-- Ground Shadow -->
      <ellipse cx="14" cy="25.5" rx="10" ry="2" fill="rgba(0,0,0,0.4)" />

      {#if activeMascot === 'duck'}
        <!-- ======================================================== -->
        <!-- ==================== 1. CHILL DUCK ===================== -->
        <!-- ======================================================== -->

        <!-- Calm Water Ripple in Sleeping state -->
        {#if currentMood === 'sleeping'}
          <ellipse cx="14" cy="25" rx="8.5" ry="2" fill="none" stroke="#38bdf8" stroke-width="0.8" class="water-ripple" />
        {/if}

        <!-- Webbed Orange Feet -->
        <ellipse cx="10" cy="24.5" rx="3.5" ry="1.5" fill="#f97316" stroke="#c2410c" stroke-width="0.75" />
        <ellipse cx="18" cy="24.5" rx="3.5" ry="1.5" fill="#f97316" stroke="#c2410c" stroke-width="0.75" />

        <!-- Perky Duck Tail -->
        <path d="M6 17 L2 14 L5 19 Z" fill="#facc15" stroke="#0f172a" stroke-width="0.9" class="duck-tail" />

        <!-- Plump Sunny Yellow Duck Body -->
        <g class="duck-body-group">
          <ellipse cx="14" cy="18" rx="8.5" ry="6.5" fill="#facc15" stroke="#0f172a" stroke-width="0.9" />
          <ellipse cx="13" cy="19" rx="5.5" ry="4" fill="#fde047" />
        </g>

        <!-- Duck Head Group -->
        <g class="duck-head-group">
          <path d="M14 4.5 Q15 1.5 16 4.5" stroke="#eab308" stroke-width="1.8" fill="none" stroke-linecap="round" />
          <circle cx="15" cy="10.5" r="6.5" fill="#facc15" stroke="#0f172a" stroke-width="0.9" />
          <circle cx="12" cy="12" r="1.5" fill="#fde047" />

          {#if currentMood === 'sleeping'}
            <path d="M12 9.5 Q14 11 16 9.5" stroke="#0f172a" stroke-width="1.6" fill="none" stroke-linecap="round" />
            <path d="M17 10.5 Q24 9.5 25 12 Q24 14.5 17 13.5 Z" fill="#f97316" stroke="#c2410c" stroke-width="0.8" />
            <ellipse cx="19" cy="11.5" rx="0.5" ry="0.4" fill="#c2410c" />
            <circle cx="11" cy="12.5" r="1.2" fill="#fbcfe8" />
          {:else if currentMood === 'music'}
            <g class="duck-sunglasses">
              <rect x="10" y="8" width="6.5" height="4.5" rx="1" fill="#0f172a" />
              <rect x="17" y="8" width="5.5" height="4.5" rx="1" fill="#0f172a" />
              <line x1="15.5" y1="9.5" x2="18" y2="9.5" stroke="#0f172a" stroke-width="1.4" />
              <line x1="11" y1="9" x2="13" y2="9" stroke="#ffffff" stroke-width="0.8" />
              <line x1="18" y1="9" x2="20" y2="9" stroke="#ffffff" stroke-width="0.8" />
            </g>
            <path d="M17 11.5 Q24 10.5 25 13 Q24 15 17 14 Z" fill="#f97316" stroke="#c2410c" stroke-width="0.8" />
          {:else if currentMood === 'typing'}
            <circle cx="15" cy="10.5" r="2.4" fill="#0ea5e9" stroke="#0f172a" stroke-width="0.8" />
            <circle cx="15.5" cy="10" r="0.9" fill="#ffffff" />
            <path d="M17 11 Q24 10 25 12.5 Q24 14.5 17 13.5 Z" fill="#f97316" stroke="#c2410c" stroke-width="0.8" />
          {:else if currentMood === 'agent'}
            <circle cx="15" cy="10.5" r="3.2" fill="rgba(14, 165, 233, 0.4)" stroke="#0284c7" stroke-width="1" />
            <line x1="12" y1="10.5" x2="18" y2="10.5" stroke="#38bdf8" stroke-width="1.2" class="visor-laser" />
            <line x1="15" y1="4.5" x2="15" y2="1.5" stroke="#3b82f6" stroke-width="1.2" />
            <circle cx="15" cy="1.5" r="1.4" fill="#38bdf8" class="antenna-beacon" />
            <path d="M17 11 Q24 10 25 12.5 Q24 14.5 17 13.5 Z" fill="#f97316" stroke="#c2410c" stroke-width="0.8" />
          {:else if currentMood === 'petting'}
            <path d="M13 9 L15.5 11 L13 13" stroke="#0f172a" stroke-width="1.8" fill="none" stroke-linecap="round" />
            <circle cx="11.5" cy="13" r="1.8" fill="#fb7185" />
            <path d="M17 11 Q24 10 25 13 Q24 15.5 17 14 Z" fill="#f97316" stroke="#c2410c" stroke-width="0.8" />
          {:else if currentMood === 'alert'}
            <circle cx="15" cy="10" r="2.8" fill="#ffffff" stroke="#0f172a" stroke-width="1" />
            <circle cx="15" cy="10" r="1.1" fill="#0f172a" />
            <ellipse cx="23" cy="8" rx="1.4" ry="2.2" fill="#38bdf8" class="sweat-drop" />
            <ellipse cx="21" cy="12.5" rx="4" ry="2.5" fill="#f97316" stroke="#c2410c" stroke-width="0.8" />
            <ellipse cx="21.5" cy="12.5" rx="2" ry="1.2" fill="#0f172a" />
          {:else}
            <circle cx="15" cy="10.5" r="2.2" fill="#0284c7" stroke="#0f172a" stroke-width="0.8" />
            <circle cx="15.5" cy="10" r="0.8" fill="#ffffff" />
            <path d="M17 11 Q24 10 25 12.5 Q24 14.5 17 13.5 Z" fill="#f97316" stroke="#c2410c" stroke-width="0.8" />
          {/if}
        </g>

        <!-- WINGS & FOREGROUND LAPTOP FOR DUCK -->
        {#if currentMood === 'typing'}
          <g class="laptop-group">
            <polygon points="5,21 23,21 25,26 3,26" fill="#334155" stroke="#0f172a" stroke-width="0.9" />
            <rect x="6" y="22" width="16" height="2.5" fill="#38bdf8" class="glowing-keys" />
            <rect x="7" y="14" width="14" height="7.5" rx="1" fill="#022c22" stroke="#10b981" stroke-width="0.9" />
            <text x="8.5" y="19" fill="#22c55e" font-size="4.5" font-family="monospace" font-weight="bold">&gt;_</text>
          </g>
          <ellipse cx="7" cy="19.5" rx="3.5" ry="2.5" fill="#eab308" stroke="#0f172a" stroke-width="0.8" class="paw-left" />
          <ellipse cx="21" cy="19.5" rx="3.5" ry="2.5" fill="#eab308" stroke="#0f172a" stroke-width="0.8" class="paw-right" />
        {:else if currentMood === 'music'}
          <ellipse cx="10" cy="18" rx="4.5" ry="3.5" fill="#eab308" stroke="#0f172a" stroke-width="0.8" class="duck-dancing-wing" />
        {:else}
          <ellipse cx="11.5" cy="18" rx="5" ry="3.5" fill="#eab308" stroke="#0f172a" stroke-width="0.8" />
        {/if}

      {:else if activeMascot === 'robot'}
        <!-- ======================================================== -->
        <!-- ==================== 2. RETRO BOT ====================== -->
        <!-- ======================================================== -->

        <!-- Rubber Caterpillar Treads -->
        <rect x="6" y="23.5" width="16" height="3" rx="1.5" fill="#334155" stroke="#0f172a" stroke-width="0.8" />
        <circle cx="8.5" cy="25" r="0.9" fill="#94a3b8" />
        <circle cx="12" cy="25" r="0.9" fill="#94a3b8" />
        <circle cx="16" cy="25" r="0.9" fill="#94a3b8" />
        <circle cx="19.5" cy="25" r="0.9" fill="#94a3b8" />

        <!-- Robot Torso / Chassis -->
        <g class="bot-body-group">
          <rect x="7" y="15" width="14" height="9" rx="2" fill="#cbd5e1" stroke="#0f172a" stroke-width="0.9" />
          <!-- Chest Power Gauge / Battery Slot -->
          <rect x="9.5" y="17.5" width="9" height="4.5" rx="1" fill="#0f172a" />
          {#if currentMood === 'sleeping'}
            <!-- Battery Standby Indicator -->
            <rect x="11" y="19" width="3" height="1.5" fill="#f59e0b" />
          {:else if currentMood === 'alert'}
            <rect x="11" y="19" width="6" height="1.5" fill="#ef4444" />
          {:else}
            <!-- Pulsing Energy Core -->
            <rect x="10.5" y="19" width="2" height="1.5" fill="#10b981" />
            <rect x="13" y="19" width="2" height="1.5" fill="#10b981" />
            <rect x="15.5" y="19" width="2" height="1.5" fill="#38bdf8" />
          {/if}
        </g>

        <!-- Robot Head & CRT Screen -->
        <g class="bot-head-group">
          <!-- Antenna Stem & Beacon -->
          <line x1="14" y1="6" x2="14" y2="2" stroke="#475569" stroke-width="1.4" />
          <circle cx="14" cy="2" r="1.6" fill={currentMood === 'alert' ? '#ef4444' : currentMood === 'sleeping' ? '#f59e0b' : '#38bdf8'} class="antenna-beacon" />

          <!-- Head Outer Chassis (Curved Monitor Casing) -->
          <rect x="5" y="6" width="18" height="10" rx="3" fill="#e2e8f0" stroke="#0f172a" stroke-width="1" />
          <!-- CRT Screen Faceplate -->
          <rect x="7" y="8" width="14" height="6.5" rx="1.5" fill="#0f172a" />

          <!-- CRT Screen Displays by Mood -->
          {#if currentMood === 'sleeping'}
            <!-- Power Save Line: [ - - ] -->
            <line x1="9" y1="11.2" x2="12" y2="11.2" stroke="#f59e0b" stroke-width="1.4" stroke-linecap="round" />
            <line x1="16" y1="11.2" x2="19" y2="11.2" stroke="#f59e0b" stroke-width="1.4" stroke-linecap="round" />

          {:else if currentMood === 'music'}
            <!-- Live Bouncing Equalizer Bars on CRT Screen -->
            <rect x="8.5" y="11" width="1.6" height="2.5" fill="#22c55e" class="bot-eq-1" />
            <rect x="11" y="9.5" width="1.6" height="4" fill="#38bdf8" class="bot-eq-2" />
            <rect x="13.5" y="8.8" width="1.6" height="4.8" fill="#c084fc" class="bot-eq-3" />
            <rect x="16" y="9.8" width="1.6" height="3.8" fill="#38bdf8" class="bot-eq-2" />
            <rect x="18.5" y="11.2" width="1.6" height="2.2" fill="#22c55e" class="bot-eq-1" />

          {:else if currentMood === 'typing'}
            <!-- Turbo Code Eyes: [ > _ ] -->
            <text x="8" y="13.2" fill="#22c55e" font-size="4.5" font-family="monospace" font-weight="bold">&gt;_</text>
            <circle cx="18" cy="11.2" r="1.2" fill="#38bdf8" />

          {:else if currentMood === 'agent'}
            <!-- Futuristic Neural Scanner Visor -->
            <line x1="8" y1="11.2" x2="20" y2="11.2" stroke="#38bdf8" stroke-width="1.6" class="visor-laser" />

          {:else if currentMood === 'petting'}
            <!-- Heart Eyes: [ ♥ ♥ ] -->
            <text x="7.8" y="13" fill="#f43f5e" font-size="4.8" font-weight="bold">♥</text>
            <text x="15.8" y="13" fill="#f43f5e" font-size="4.8" font-weight="bold">♥</text>

          {:else if currentMood === 'alert'}
            <!-- Red Warning: [ ! ] -->
            <text x="12" y="13.5" fill="#ef4444" font-size="5" font-family="monospace" font-weight="bold">!</text>

          {:else}
            <!-- Default Glowing Cyan Pixel Eyes -->
            <rect x="9" y="10" width="2.5" height="2.5" rx="0.5" fill="#38bdf8" />
            <rect x="16.5" y="10" width="2.5" height="2.5" rx="0.5" fill="#38bdf8" />
          {/if}
        </g>

        <!-- Mechanical Arms & Foreground Laptop -->
        {#if currentMood === 'typing'}
          <g class="laptop-group">
            <polygon points="5,21 23,21 25,26 3,26" fill="#334155" stroke="#0f172a" stroke-width="0.9" />
            <rect x="6" y="22" width="16" height="2.5" fill="#38bdf8" class="glowing-keys" />
            <rect x="7" y="14" width="14" height="7.5" rx="1" fill="#022c22" stroke="#10b981" stroke-width="0.9" />
            <text x="8.5" y="19" fill="#22c55e" font-size="4.5" font-family="monospace" font-weight="bold">&gt;_</text>
          </g>
          <!-- Bongo Mechanical Claws -->
          <rect x="5.5" y="18" width="3.5" height="3" rx="1" fill="#94a3b8" stroke="#0f172a" stroke-width="0.8" class="paw-left" />
          <rect x="19" y="18" width="3.5" height="3" rx="1" fill="#94a3b8" stroke="#0f172a" stroke-width="0.8" class="paw-right" />
        {:else}
          <!-- Resting Joint Arms -->
          <rect x="4" y="16.5" width="3" height="5.5" rx="1.5" fill="#94a3b8" stroke="#0f172a" stroke-width="0.8" />
          <rect x="21" y="16.5" width="3" height="5.5" rx="1.5" fill="#94a3b8" stroke="#0f172a" stroke-width="0.8" />
        {/if}

      {:else if activeMascot === 'fox'}
        <!-- ======================================================== -->
        <!-- ==================== 3. KITSUNE FOX ==================== -->
        <!-- ======================================================== -->

        <!-- Magnificent Fluffy Fox Tail -->
        <g class="fox-tail-group">
          {#if currentMood === 'sleeping'}
            <!-- Curled around body -->
            <path d="M7 23 Q14 26 21 23 Q25 20 22 17 Q19 19 14 21 Z" fill="#ea580c" stroke="#0f172a" stroke-width="0.9" />
            <polygon points="6,21 3,24 8,24" fill="#ffffff" />
          {:else}
            <!-- Upright Swishing Fluffy Tail with White Tip -->
            <path d="M19 19 Q28 13 26 7 Q21 6 18 13 Z" fill="#ea580c" stroke="#0f172a" stroke-width="0.9" class="fox-tail" />
            <path d="M26 7 Q23 6 22 9 Q25 8 26 7 Z" fill="#ffffff" />
            <!-- Glowing Blue Spirit Flame Wisp on Tail Tip -->
            <circle cx="25" cy="6" r="1.4" fill="#38bdf8" opacity="0.8" class="spirit-flame" />
          {/if}
        </g>

        <!-- Autumn Orange Body -->
        <g class="fox-body-group">
          <rect x="7" y="16" width="14" height="9" rx="2.5" fill="#ea580c" stroke="#0f172a" stroke-width="0.9" />
          <!-- White Chest Ruff / Bib -->
          <polygon points="10,16 18,16 14,21" fill="#ffffff" />
          <!-- Shimenawa Rope Collar with Cyan Magatama Bead -->
          <rect x="8" y="16.5" width="12" height="1.6" fill="#dc2626" />
          <circle cx="14" cy="18.5" r="1.4" fill="#38bdf8" stroke="#0284c7" stroke-width="0.5" />
        </g>

        <!-- Fox Head Group -->
        <g class="fox-head-group">
          <!-- Large Pointy Ears with Black Tips and Pink Fluff -->
          <polygon points="4,9 7,1 11,9" fill="#ea580c" stroke="#0f172a" stroke-width="0.9" />
          <polygon points="6,4 7,1 8,4" fill="#0f172a" />
          <polygon points="6,8 7,5 9,8" fill="#f472b6" />

          <polygon points="17,9 21,1 24,9" fill="#ea580c" stroke="#0f172a" stroke-width="0.9" />
          <polygon points="20,4 21,1 22,4" fill="#0f172a" />
          <polygon points="19,8 21,5 22,8" fill="#f472b6" />

          <!-- Fox Head Base -->
          <rect x="5.5" y="7.5" width="17" height="10" rx="2.5" fill="#ea580c" stroke="#0f172a" stroke-width="0.9" />
          <!-- White Muzzle Mask -->
          <path d="M9 13.5 L14 11 L19 13.5 L14 17.5 Z" fill="#ffffff" stroke="#0f172a" stroke-width="0.6" />
          <!-- Black Fox Nose -->
          <circle cx="14" cy="15.8" r="0.8" fill="#0f172a" />

          <!-- Fox Whiskers -->
          <line x1="2" y1="13" x2="5" y2="13.5" stroke="#0f172a" stroke-width="0.8" />
          <line x1="23" y1="13.5" x2="26" y2="13" stroke="#0f172a" stroke-width="0.8" />

          <!-- EXPRESSIONS BY MOOD -->
          {#if currentMood === 'sleeping'}
            <!-- Serene Sleepy Smile Eyes: ^ ^ -->
            <path d="M8 12 Q10 13.5 12 12" stroke="#0f172a" stroke-width="1.6" fill="none" stroke-linecap="round" />
            <path d="M16 12 Q18 13.5 20 12" stroke="#0f172a" stroke-width="1.6" fill="none" stroke-linecap="round" />
            <circle cx="7.5" cy="14" r="1.2" fill="#fbcfe8" />
            <circle cx="20.5" cy="14" r="1.2" fill="#fbcfe8" />

          {:else if currentMood === 'music'}
            <!-- Dancing Fox Eyes & Neon Headphones -->
            <path d="M8 13.5 L10 11 L12 13.5" stroke="#0f172a" stroke-width="1.8" fill="none" stroke-linecap="round" />
            <path d="M16 13.5 L18 11 L20 13.5" stroke="#0f172a" stroke-width="1.8" fill="none" stroke-linecap="round" />
            <!-- Headband & Glowing Earcups -->
            <path d="M3.5 7.5 Q14 0 24.5 7.5" stroke="#c084fc" stroke-width="2.4" fill="none" stroke-linecap="round" />
            <rect x="2" y="7" width="4" height="6.5" rx="1.5" fill="#06b6d4" class="neon-earcup" />
            <rect x="22" y="7" width="4" height="6.5" rx="1.5" fill="#06b6d4" class="neon-earcup" />

          {:else if currentMood === 'typing'}
            <!-- Focused Cunning Eyes -->
            <ellipse cx="10" cy="12" rx="2.2" ry="2.5" fill="#0ea5e9" stroke="#0f172a" stroke-width="0.8" />
            <circle cx="10.5" cy="11.5" r="0.9" fill="#ffffff" />
            <ellipse cx="18" cy="12" rx="2.2" ry="2.5" fill="#0ea5e9" stroke="#0f172a" stroke-width="0.8" />
            <circle cx="18.5" cy="11.5" r="0.9" fill="#ffffff" />

          {:else if currentMood === 'agent'}
            <!-- Cyber Monocle & Floating Kitsune Spirit Orb -->
            <rect x="7" y="10" width="14" height="5" rx="1.5" fill="rgba(14, 165, 233, 0.45)" stroke="#0284c7" stroke-width="0.9" />
            <line x1="8" y1="12.5" x2="20" y2="12.5" stroke="#38bdf8" stroke-width="1.2" class="visor-laser" />

          {:else if currentMood === 'petting'}
            <!-- Joyful Squint: > < -->
            <path d="M8 10.5 L10.5 12.5 L8 14.5" stroke="#0f172a" stroke-width="1.8" fill="none" stroke-linecap="round" />
            <path d="M20 10.5 L17.5 12.5 L20 14.5" stroke="#0f172a" stroke-width="1.8" fill="none" stroke-linecap="round" />
            <circle cx="7" cy="14" r="1.8" fill="#fb7185" />
            <circle cx="21" cy="14" r="1.8" fill="#fb7185" />

          {:else if currentMood === 'alert'}
            <circle cx="10" cy="12" r="2.6" fill="#ffffff" stroke="#0f172a" stroke-width="1" />
            <circle cx="10" cy="12" r="1" fill="#0f172a" />
            <circle cx="18" cy="12" r="2.6" fill="#ffffff" stroke="#0f172a" stroke-width="1" />
            <circle cx="18" cy="12" r="1" fill="#0f172a" />
            <ellipse cx="23.5" cy="9.5" rx="1.4" ry="2.2" fill="#38bdf8" class="sweat-drop" />

          {:else}
            <!-- Default Anime Fox Eyes -->
            <ellipse cx="10" cy="12" rx="2" ry="2.4" fill="#0284c7" stroke="#0f172a" stroke-width="0.8" />
            <circle cx="10.5" cy="11.5" r="0.8" fill="#ffffff" />
            <ellipse cx="18" cy="12" rx="2" ry="2.4" fill="#0284c7" stroke="#0f172a" stroke-width="0.8" />
            <circle cx="18.5" cy="11.5" r="0.8" fill="#ffffff" />
          {/if}
        </g>

        <!-- Fox Paws & Laptop -->
        {#if currentMood === 'typing'}
          <g class="laptop-group">
            <polygon points="5,21 23,21 25,26 3,26" fill="#334155" stroke="#0f172a" stroke-width="0.9" />
            <rect x="6" y="22" width="16" height="2.5" fill="#38bdf8" class="glowing-keys" />
            <rect x="7" y="14" width="14" height="7.5" rx="1" fill="#022c22" stroke="#10b981" stroke-width="0.9" />
            <text x="8.5" y="19" fill="#22c55e" font-size="4.5" font-family="monospace" font-weight="bold">&gt;_</text>
          </g>
          <!-- Black Sock Fox Paws alternating Bongo tapping -->
          <ellipse cx="7.5" cy="19.5" rx="2.5" ry="3.5" fill="#0f172a" stroke="#334155" stroke-width="0.8" class="paw-left" />
          <ellipse cx="20.5" cy="19.5" rx="2.5" ry="3.5" fill="#0f172a" stroke="#334155" stroke-width="0.8" class="paw-right" />
        {:else}
          <!-- Resting Black Fox Socks Paws with White Toes -->
          <ellipse cx="8.5" cy="23.5" rx="2.6" ry="1.8" fill="#0f172a" stroke="#334155" stroke-width="0.8" />
          <ellipse cx="19.5" cy="23.5" rx="2.6" ry="1.8" fill="#0f172a" stroke="#334155" stroke-width="0.8" />
        {/if}

      {:else}
        <!-- ======================================================== -->
        <!-- ==================== 4. CYBER-CAT ====================== -->
        <!-- ======================================================== -->

        <!-- Tail -->
        <g class="cat-tail-group">
          {#if currentMood === 'sleeping'}
            <rect x="21" y="20" width="4" height="3" fill="#cbd5e1" stroke="#0f172a" stroke-width="0.75" />
            <rect x="23" y="17" width="3" height="4" fill="#f8fafc" stroke="#0f172a" stroke-width="0.75" />
            <rect x="22" y="15" width="3" height="3" fill="#38bdf8" />
          {:else}
            <rect x="21" y="17" width="3.5" height="5" fill="#cbd5e1" stroke="#0f172a" stroke-width="0.75" />
            <rect x="23" y="13" width="3.5" height="5" fill="#f8fafc" stroke="#0f172a" stroke-width="0.75" />
            <rect x="24" y="9" width="3" height="4.5" fill="#38bdf8" stroke="#0f172a" stroke-width="0.75" />
          {/if}
        </g>

        <!-- Main Body -->
        <g class="cat-body-group">
          <rect x="6" y="16" width="16" height="9" rx="2" fill="#f8fafc" stroke="#0f172a" stroke-width="0.9" />
          <rect x="9" y="17" width="10" height="7" fill="#e2e8f0" />
        </g>

        <!-- Cat Head Group -->
        <g class="cat-head-group">
          <!-- Ears -->
          <polygon points="4,9 7,2 11,9" fill="#f8fafc" stroke="#0f172a" stroke-width="0.9" />
          <polygon points="6,8 7,4 9,8" fill="#f472b6" />
          <polygon points="17,9 21,2 24,9" fill="#f8fafc" stroke="#0f172a" stroke-width="0.9" />
          <polygon points="19,8 21,4 22,8" fill="#f472b6" />

          <!-- Head Base Box -->
          <rect x="5" y="7" width="18" height="11" rx="2.5" fill="#f8fafc" stroke="#0f172a" stroke-width="0.9" />
          <!-- Forehead Patch -->
          <rect x="10" y="7" width="8" height="2.5" fill="#38bdf8" />

          <!-- Collar & Bell -->
          <rect x="7" y="17" width="14" height="2" fill="#10b981" stroke="#065f46" stroke-width="0.5" />
          <circle cx="14" cy="18" r="1.6" fill="#fbbf24" stroke="#b45309" stroke-width="0.5" />

          <!-- Whiskers -->
          <line x1="2" y1="12" x2="5" y2="12.5" stroke="#94a3b8" stroke-width="0.9" />
          <line x1="2" y1="14" x2="5" y2="14" stroke="#94a3b8" stroke-width="0.9" />
          <line x1="23" y1="12.5" x2="26" y2="12" stroke="#94a3b8" stroke-width="0.9" />
          <line x1="23" y1="14" x2="26" y2="14" stroke="#94a3b8" stroke-width="0.9" />

          <!-- Nose -->
          <polygon points="13,13.5 15,13.5 14,14.5" fill="#f472b6" />

          <!-- EXPRESSIONS BY MOOD -->
          {#if currentMood === 'sleeping'}
            <path d="M7.5 12 Q9.5 13.5 11.5 12" stroke="#0f172a" stroke-width="1.6" fill="none" stroke-linecap="round" />
            <path d="M16.5 12 Q18.5 13.5 20.5 12" stroke="#0f172a" stroke-width="1.6" fill="none" stroke-linecap="round" />
            <circle cx="7" cy="14" r="1.2" fill="#fbcfe8" />
            <circle cx="21" cy="14" r="1.2" fill="#fbcfe8" />

          {:else if currentMood === 'music'}
            <path d="M7.5 13.5 L9.5 11 L11.5 13.5" stroke="#0f172a" stroke-width="1.8" fill="none" stroke-linecap="round" />
            <path d="M16.5 13.5 L18.5 11 L20.5 13.5" stroke="#0f172a" stroke-width="1.8" fill="none" stroke-linecap="round" />
            <!-- Neon Headband -->
            <path d="M3.5 8 Q14 0 24.5 8" stroke="#c084fc" stroke-width="2.6" fill="none" stroke-linecap="round" />
            <rect x="2" y="7" width="4.5" height="7.5" rx="2" fill="#06b6d4" stroke="#0891b2" stroke-width="0.8" class="neon-earcup" />
            <rect x="21.5" y="7" width="4.5" height="7.5" rx="2" fill="#06b6d4" stroke="#0891b2" stroke-width="0.8" class="neon-earcup" />
            <circle cx="4.2" cy="10.8" r="1.2" fill="#ffffff" />
            <circle cx="23.8" cy="10.8" r="1.2" fill="#ffffff" />

          {:else if currentMood === 'typing'}
            <ellipse cx="9.5" cy="12" rx="2.4" ry="2.6" fill="#0ea5e9" stroke="#0f172a" stroke-width="0.8" />
            <circle cx="10" cy="11.5" r="1" fill="#ffffff" />
            <ellipse cx="18.5" cy="12" rx="2.4" ry="2.6" fill="#0ea5e9" stroke="#0f172a" stroke-width="0.8" />
            <circle cx="19" cy="11.5" r="1" fill="#ffffff" />

          {:else if currentMood === 'agent'}
            <rect x="6" y="9.5" width="16" height="5.5" rx="1.5" fill="rgba(14, 165, 233, 0.45)" stroke="#0284c7" stroke-width="1" />
            <line x1="7" y1="12" x2="21" y2="12" stroke="#38bdf8" stroke-width="1.4" class="visor-laser" />
            <line x1="14" y1="7" x2="14" y2="3.5" stroke="#3b82f6" stroke-width="1.2" />
            <circle cx="14" cy="3" r="1.4" fill="#38bdf8" class="antenna-beacon" />

          {:else if currentMood === 'petting'}
            <path d="M7 10.5 L9.5 12.5 L7 14.5" stroke="#0f172a" stroke-width="1.8" fill="none" stroke-linecap="round" />
            <path d="M21 10.5 L18.5 12.5 L21 14.5" stroke="#0f172a" stroke-width="1.8" fill="none" stroke-linecap="round" />
            <circle cx="6.5" cy="14" r="2" fill="#fb7185" />
            <circle cx="21.5" cy="14" r="2" fill="#fb7185" />
            <path d="M12 15 Q13 16.5 14 15 Q15 16.5 16 15" stroke="#0f172a" stroke-width="1.2" fill="none" />

          {:else if currentMood === 'alert'}
            <circle cx="9.5" cy="12" r="2.8" fill="#ffffff" stroke="#0f172a" stroke-width="1" />
            <circle cx="9.5" cy="12" r="1" fill="#0f172a" />
            <circle cx="18.5" cy="12" r="2.8" fill="#ffffff" stroke="#0f172a" stroke-width="1" />
            <circle cx="18.5" cy="12" r="1" fill="#0f172a" />
            <ellipse cx="23.5" cy="9.5" rx="1.4" ry="2.2" fill="#38bdf8" class="sweat-drop" />

          {:else}
            <ellipse cx="9.5" cy="12" rx="2.2" ry="2.5" fill="#0284c7" stroke="#0f172a" stroke-width="0.8" />
            <circle cx="10" cy="11.5" r="1" fill="#ffffff" />
            <ellipse cx="18.5" cy="12" rx="2.2" ry="2.5" fill="#0284c7" stroke="#0f172a" stroke-width="0.8" />
            <circle cx="19" cy="11.5" r="1" fill="#ffffff" />
          {/if}
        </g>

        <!-- FOREGROUND PAWS & LAPTOP FOR CAT -->
        {#if currentMood === 'typing'}
          <g class="laptop-group">
            <polygon points="5,21 23,21 25,26 3,26" fill="#334155" stroke="#0f172a" stroke-width="0.9" />
            <rect x="6" y="22" width="16" height="2.5" fill="#38bdf8" class="glowing-keys" />
            <rect x="7" y="14" width="14" height="7.5" rx="1" fill="#022c22" stroke="#10b981" stroke-width="0.9" />
            <text x="8.5" y="19" fill="#22c55e" font-size="4.5" font-family="monospace" font-weight="bold">&gt;_</text>
          </g>
          <ellipse cx="7.5" cy="19.5" rx="2.5" ry="3.5" fill="#f472b6" stroke="#0f172a" stroke-width="0.8" class="paw-left" />
          <ellipse cx="20.5" cy="19.5" rx="2.5" ry="3.5" fill="#f472b6" stroke="#0f172a" stroke-width="0.8" class="paw-right" />
        {:else}
          <g class="resting-paws">
            <ellipse cx="8" cy="23.5" rx="2.6" ry="2" fill="#f8fafc" stroke="#0f172a" stroke-width="0.8" />
            <ellipse cx="20" cy="23.5" rx="2.6" ry="2" fill="#f8fafc" stroke="#0f172a" stroke-width="0.8" />
          </g>
        {/if}
      {/if}
    </svg>
  </button>
{/if}

<style>
  .pixel-mascot-container {
    background: none;
    border: none;
    padding: 0;
    margin: 0;
    outline: none;
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    user-select: none;
    flex-shrink: 0;
    transition: transform 0.18s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  .pixel-mascot-container:hover {
    transform: scale(1.15) translateY(-1px);
  }

  .pixel-mascot-container:active {
    transform: scale(0.92);
  }

  /* Bold, prominent sizes */
  .size-sm {
    width: 32px;
    height: 32px;
  }

  .size-md {
    width: 40px;
    height: 40px;
  }

  .size-lg {
    width: 64px;
    height: 64px;
  }

  .mascot-svg {
    width: 100%;
    height: 100%;
    overflow: visible;
  }

  /* ==================== 1. SLEEPING STATE ==================== */
  .mood-sleeping .cat-head-group,
  .mood-sleeping .duck-head-group,
  .mood-sleeping .bot-head-group,
  .mood-sleeping .fox-head-group {
    animation: mascotBreathe 3.2s ease-in-out infinite;
    transform-origin: 14px 17px;
  }

  @keyframes mascotBreathe {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(1.2px); }
  }

  .water-ripple {
    animation: rippleExpand 2.8s infinite ease-out;
    transform-origin: 14px 25px;
  }

  @keyframes rippleExpand {
    0% { transform: scale(0.7); opacity: 0.8; }
    100% { transform: scale(1.4); opacity: 0; }
  }

  .pixel-particle {
    position: absolute;
    pointer-events: none;
    line-height: 1;
    font-weight: 800;
  }

  .pixel-z {
    color: #38bdf8;
    font-family: monospace;
    text-shadow: 0 0 6px rgba(56, 189, 248, 0.8);
    opacity: 0;
  }

  .z-1 {
    font-size: 10px;
    top: -2px;
    right: 2px;
    animation: floatZ 3s infinite ease-out;
  }

  .z-2 {
    font-size: 13px;
    top: -8px;
    right: -3px;
    animation: floatZ 3s infinite ease-out 1s;
  }

  .z-3 {
    font-size: 9px;
    top: -14px;
    right: -7px;
    animation: floatZ 3s infinite ease-out 2s;
  }

  @keyframes floatZ {
    0% { transform: translate(0, 4px) scale(0.6); opacity: 0; }
    25% { opacity: 1; }
    75% { opacity: 0.7; }
    100% { transform: translate(8px, -16px) scale(1.3); opacity: 0; }
  }

  /* ==================== 2. MUSIC STATE (Grooving) ==================== */
  .mood-music .cat-head-group,
  .mood-music .duck-head-group,
  .mood-music .bot-head-group,
  .mood-music .fox-head-group {
    animation: headBobGroove 0.48s ease-in-out infinite alternate;
    transform-origin: 14px 18px;
  }

  .mood-music .cat-tail-group,
  .mood-music .duck-tail,
  .mood-music .fox-tail {
    animation: tailWagMusic 0.48s ease-in-out infinite alternate;
    transform-origin: 21px 22px;
  }

  .duck-dancing-wing {
    animation: duckWingDance 0.48s ease-in-out infinite alternate;
    transform-origin: 10px 18px;
  }

  @keyframes headBobGroove {
    0% { transform: translateY(0) rotate(0deg); }
    100% { transform: translateY(2.2px) rotate(-3deg); }
  }

  @keyframes tailWagMusic {
    0% { transform: rotate(-8deg); }
    100% { transform: rotate(18deg); }
  }

  @keyframes duckWingDance {
    0% { transform: translateY(0) scaleY(1); }
    100% { transform: translateY(-3px) scaleY(1.15) rotate(-10deg); }
  }

  /* Robot Live CRT Equalizer Animation */
  .bot-eq-1 {
    animation: botEqFlicker 0.42s ease-in-out infinite alternate;
    transform-origin: bottom;
  }

  .bot-eq-2 {
    animation: botEqFlicker 0.28s ease-in-out infinite alternate 0.1s;
    transform-origin: bottom;
  }

  .bot-eq-3 {
    animation: botEqFlicker 0.35s ease-in-out infinite alternate 0.05s;
    transform-origin: bottom;
  }

  @keyframes botEqFlicker {
    0% { transform: scaleY(0.4); }
    100% { transform: scaleY(1.2); }
  }

  .spirit-flame {
    animation: flamePulse 1.2s ease-in-out infinite alternate;
  }

  @keyframes flamePulse {
    0% { transform: scale(0.8) translateY(0); filter: drop-shadow(0 0 2px #38bdf8); }
    100% { transform: scale(1.3) translateY(-2px); filter: drop-shadow(0 0 6px #06b6d4); }
  }

  .neon-earcup {
    filter: drop-shadow(0 0 4px #06b6d4);
  }

  .music-note {
    font-size: 14px;
    color: #c084fc;
    text-shadow: 0 0 8px #c084fc;
  }

  .note-1 {
    top: -8px;
    left: -4px;
    animation: floatNote 1.4s infinite ease-out;
  }

  .note-2 {
    top: -12px;
    right: -4px;
    animation: floatNote 1.4s infinite ease-out 0.7s;
  }

  @keyframes floatNote {
    0% { transform: translateY(4px) scale(0.7) rotate(-10deg); opacity: 0; }
    30% { opacity: 1; }
    80% { opacity: 0.8; }
    100% { transform: translateY(-14px) scale(1.2) rotate(15deg); opacity: 0; }
  }

  /* ==================== 3. TYPING STATE (Bongo Mascot) ==================== */
  .mood-typing .paw-left {
    animation: bongoSlapLeft 0.13s ease-in-out infinite alternate;
    transform-origin: 7.5px 22px;
  }

  .mood-typing .paw-right {
    animation: bongoSlapRight 0.13s ease-in-out infinite alternate;
    transform-origin: 20.5px 22px;
  }

  @keyframes bongoSlapLeft {
    0% { transform: translateY(0); }
    100% { transform: translateY(-3.5px); }
  }

  @keyframes bongoSlapRight {
    0% { transform: translateY(-3.5px); }
    100% { transform: translateY(0); }
  }

  .glowing-keys {
    animation: keysFlicker 0.25s infinite alternate;
  }

  @keyframes keysFlicker {
    0% { fill: #38bdf8; filter: drop-shadow(0 0 3px #38bdf8); }
    100% { fill: #f59e0b; filter: drop-shadow(0 0 4px #f59e0b); }
  }

  .spark {
    font-size: 13px;
    color: #fbbf24;
    text-shadow: 0 0 6px #fbbf24;
  }

  .spark-1 {
    top: 4px;
    left: -5px;
    animation: sparkPop 0.8s infinite ease-out;
  }

  .spark-2 {
    top: 2px;
    right: -4px;
    animation: sparkPop 0.8s infinite ease-out 0.4s;
  }

  @keyframes sparkPop {
    0% { transform: scale(0.4) rotate(0deg); opacity: 0; }
    40% { opacity: 1; transform: scale(1.2) rotate(20deg); }
    100% { transform: scale(0.6) translateY(-8px) rotate(40deg); opacity: 0; }
  }

  /* ==================== 4. AI AGENT STATE (Cyber Visor) ==================== */
  .visor-laser {
    animation: laserScan 1.1s ease-in-out infinite alternate;
  }

  @keyframes laserScan {
    0% { transform: translateY(-1.8px); stroke: #38bdf8; }
    100% { transform: translateY(1.8px); stroke: #a855f7; }
  }

  .antenna-beacon {
    animation: beaconBlink 0.7s infinite alternate;
  }

  @keyframes beaconBlink {
    0% { fill: #38bdf8; filter: drop-shadow(0 0 2px #38bdf8); }
    100% { fill: #f43f5e; filter: drop-shadow(0 0 5px #f43f5e); }
  }

  .agent-bubble {
    top: -12px;
    font-size: 14px;
    animation: thoughtFloat 1.8s infinite ease-in-out alternate;
  }

  @keyframes thoughtFloat {
    0% { transform: translateY(0) scale(0.9); }
    100% { transform: translateY(-4px) scale(1.15); }
  }

  /* ==================== 5. PETTING STATE ==================== */
  .mood-petting {
    animation: happyPetHop 0.35s ease-in-out 3 alternate;
  }

  @keyframes happyPetHop {
    0% { transform: translateY(0) scale(1); }
    100% { transform: translateY(-5px) scale(1.12); }
  }

  .pixel-heart {
    top: -6px;
    left: 50%;
    color: #f43f5e;
    font-size: 15px;
    text-shadow: 0 0 8px rgba(244, 63, 94, 0.8);
    animation: floatHeart 1.1s forwards ease-out;
  }

  @keyframes floatHeart {
    0% { transform: translate(var(--burst-x, 0px), 0) scale(0.5); opacity: 1; }
    100% { transform: translate(calc(var(--burst-x, 0px) * 2), -24px) scale(1.5); opacity: 0; }
  }

  /* ==================== 6. ALERT STATE ==================== */
  .sweat-drop {
    animation: dropFall 1.2s infinite ease-in;
  }

  @keyframes dropFall {
    0% { transform: translateY(0); opacity: 0; }
    30% { opacity: 1; }
    100% { transform: translateY(8px); opacity: 0; }
  }
</style>
