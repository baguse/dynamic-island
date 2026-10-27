<script lang="ts">
  import { onMount } from 'svelte';
  import { islandState } from '../state/island.svelte';

  interface Props {
    isPlaying?: boolean;
    style?: 'bars' | 'wave' | 'dots';
    size?: 'sm' | 'md';
    showGlow?: boolean;
  }

  let props: Props = $props();

  const isPlaying = $derived(props.isPlaying !== undefined ? props.isPlaying : islandState.isPlaying);
  const style = $derived(props.style ?? islandState.settings.equalizer_style ?? 'bars');
  const size = $derived(props.size ?? 'sm');
  const showGlow = $derived(props.showGlow ?? true);

  let canvasEl = $state<HTMLCanvasElement | null>(null);
  let animId: number | null = null;
  let t = 0;

  // Smoothing states for bars, wave, and dots
  const barHeights = [4, 4, 4, 4, 4];
  let waveAmp = 0;
  const dotScales = [1, 1, 1, 1];

  function cycleStyle(e: MouseEvent) {
    e.stopPropagation();
    const next = style === 'bars' ? 'wave' : style === 'wave' ? 'dots' : 'bars';
    islandState.updateSettings({ equalizer_style: next });
  }

  function render() {
    if (!canvasEl) return;
    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;

    const w = canvasEl.width;
    const h = canvasEl.height;

    ctx.clearRect(0, 0, w, h);

    if (style === 'wave') {
      renderWave(ctx, w, h);
    } else if (style === 'dots') {
      renderDots(ctx, w, h);
    } else {
      renderBars(ctx, w, h);
    }

    t += 0.07;

    // Check if we need to keep rendering
    // If playing, keep looping at 60fps. If paused, loop until settled at rest.
    const isSettled =
      !isPlaying &&
      barHeights.every((bh) => Math.abs(bh - 3) < 0.2) &&
      waveAmp < 0.1 &&
      dotScales.every((ds) => Math.abs(ds - 0.7) < 0.05);

    if (isPlaying || !isSettled) {
      animId = requestAnimationFrame(render);
    } else {
      animId = null;
    }
  }

  // ---------------------------------------------------------------------------
  // Style 1: Neon Spectrum Equalizer Bars (Real Audio Reactive)
  // ---------------------------------------------------------------------------
  function renderBars(ctx: CanvasRenderingContext2D, w: number, h: number) {
    const numBars = 5;
    const barW = size === 'md' ? 4 : 3;
    const gap = size === 'md' ? 3 : 2;
    const totalW = numBars * barW + (numBars - 1) * gap;
    const startX = Math.round((w - totalW) / 2);
    const maxH = h - 2;
    const restH = size === 'md' ? 4 : 3;

    const spectrum = islandState.audioSpectrum;
    const realBands = spectrum.bands;
    const hasRealAudio = realBands.some((b) => b > 0.02);

    // If real PipeWire/PulseAudio spectrum is active, use genuine live frequency audio!
    const targets = isPlaying
      ? [
          restH + (maxH - restH) * (hasRealAudio ? Math.max(realBands[0], 0.15 * Math.abs(Math.sin(t * 1.9))) : (0.2 + 0.75 * Math.abs(Math.sin(t * 1.9)))),
          restH + (maxH - restH) * (hasRealAudio ? Math.max(realBands[1], 0.15 * Math.abs(Math.sin(t * 2.8 + 0.8))) : (0.15 + 0.85 * Math.abs(Math.sin(t * 2.8 + 0.8)))),
          restH + (maxH - restH) * (hasRealAudio ? Math.max(realBands[2], 0.15 * Math.abs(Math.sin(t * 1.5 + 1.7))) : (0.3 + 0.7 * Math.abs(Math.sin(t * 1.5 + 1.7)))),
          restH + (maxH - restH) * (hasRealAudio ? Math.max(realBands[3], 0.15 * Math.abs(Math.sin(t * 3.4 + 2.3))) : (0.15 + 0.85 * Math.abs(Math.sin(t * 3.4 + 2.3)))),
          restH + (maxH - restH) * (hasRealAudio ? Math.max(realBands[4], 0.15 * Math.abs(Math.sin(t * 4.1 + 3.1))) : (0.2 + 0.65 * Math.abs(Math.sin(t * 4.1 + 3.1)))),
        ]
      : [restH, restH, restH, restH, restH];

    for (let i = 0; i < numBars; i++) {
      const target = targets[i];
      if (hasRealAudio) {
        if (target > barHeights[i]) {
          // Snappy attack: jump immediately to catch the beat/kick!
          barHeights[i] += (target - barHeights[i]) * 0.75;
        } else {
          // Smooth decay: natural physical release
          barHeights[i] += (target - barHeights[i]) * 0.22;
        }
      } else {
        barHeights[i] += (target - barHeights[i]) * 0.18;
      }
      const bh = Math.max(restH, Math.min(maxH, barHeights[i]));
      const x = startX + i * (barW + gap);
      const y = h - bh;

      // Gradient
      const grad = ctx.createLinearGradient(0, y, 0, h);
      grad.addColorStop(0, '#38bdf8');
      grad.addColorStop(0.5, '#818cf8');
      grad.addColorStop(1, '#c084fc');

      ctx.save();
      if (showGlow && isPlaying) {
        ctx.shadowColor = 'rgba(56, 189, 248, 0.45)';
        ctx.shadowBlur = 4;
      }
      ctx.globalAlpha = isPlaying ? 0.95 : 0.4;
      ctx.fillStyle = grad;

      ctx.beginPath();
      ctx.roundRect(x, y, barW, bh, barW / 2);
      ctx.fill();
      ctx.restore();
    }
  }

  // ---------------------------------------------------------------------------
  // Style 2: Fluid Sine Waveform (Apple Style Real Audio Reactive)
  // ---------------------------------------------------------------------------
  function renderWave(ctx: CanvasRenderingContext2D, w: number, h: number) {
    const midY = h / 2;
    const spectrum = islandState.audioSpectrum;
    const realEnergy = spectrum.energy;
    const bassEnergy = (spectrum.bands[0] + spectrum.bands[1]) * 0.5;
    const audioMod = Math.max(realEnergy, bassEnergy);
    const hasRealAudio = audioMod > 0.02;

    const baseAmp = size === 'md' ? 8.5 : 6.0;
    const targetAmp = isPlaying
      ? baseAmp * (hasRealAudio ? 0.35 + 0.9 * audioMod : 1.0)
      : 0;
    if (targetAmp > waveAmp) {
      waveAmp += (targetAmp - waveAmp) * (hasRealAudio ? 0.65 : 0.22);
    } else {
      waveAmp += (targetAmp - waveAmp) * (hasRealAudio ? 0.20 : 0.12);
    }

    // Cyan / Blue Wave (Background)
    ctx.save();
    if (showGlow && isPlaying) {
      ctx.shadowColor = 'rgba(56, 189, 248, 0.5)';
      ctx.shadowBlur = 5;
    }
    ctx.beginPath();
    ctx.lineWidth = size === 'md' ? 2.2 : 1.8;
    ctx.strokeStyle = '#38bdf8';
    ctx.globalAlpha = isPlaying ? 0.8 : 0.35;
    for (let x = 0; x <= w; x += 2) {
      const normX = x / w;
      const env = Math.sin(normX * Math.PI);
      const y = midY + Math.sin(normX * 8.5 + t * 1.4) * waveAmp * env;
      if (x === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.stroke();
    ctx.restore();

    // Magenta / Purple Wave (Foreground Harmonic)
    ctx.save();
    if (showGlow && isPlaying) {
      ctx.shadowColor = 'rgba(192, 132, 252, 0.5)';
      ctx.shadowBlur = 5;
    }
    ctx.beginPath();
    ctx.lineWidth = size === 'md' ? 2.2 : 1.8;
    ctx.strokeStyle = '#c084fc';
    ctx.globalAlpha = isPlaying ? 0.9 : 0.35;
    for (let x = 0; x <= w; x += 2) {
      const normX = x / w;
      const env = Math.sin(normX * Math.PI);
      const y = midY + Math.sin(normX * 10.5 - t * 1.8) * (waveAmp * 0.85) * env;
      if (x === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.stroke();
    ctx.restore();
  }

  // ---------------------------------------------------------------------------
  // Style 3: Minimalist Pulsing Dots (Real Audio Reactive)
  // ---------------------------------------------------------------------------
  function renderDots(ctx: CanvasRenderingContext2D, w: number, h: number) {
    const numDots = 4;
    const baseR = size === 'md' ? 2.2 : 1.8;
    const gap = size === 'md' ? 6 : 4.5;
    const totalW = (numDots - 1) * gap;
    const startX = (w - totalW) / 2;
    const midY = h / 2;

    const spectrum = islandState.audioSpectrum;
    const realBands = spectrum.bands;
    const hasRealAudio = realBands.some((b) => b > 0.02);

    for (let i = 0; i < numDots; i++) {
      const realLevel = realBands[i] ?? 0;
      const proceduralScale = 0.6 + 0.65 * (0.5 + 0.5 * Math.sin(t * 2.2 + i * 0.7));
      const targetScale = isPlaying
        ? (hasRealAudio ? Math.max(0.45 + realLevel * 0.85, proceduralScale * 0.4) : proceduralScale)
        : 0.65;
      if (targetScale > dotScales[i]) {
        dotScales[i] += (targetScale - dotScales[i]) * (hasRealAudio ? 0.65 : 0.25);
      } else {
        dotScales[i] += (targetScale - dotScales[i]) * (hasRealAudio ? 0.20 : 0.15);
      }
      const r = baseR * dotScales[i];
      const x = startX + i * gap;

      ctx.save();
      if (showGlow && isPlaying) {
        ctx.shadowColor = 'rgba(56, 189, 248, 0.6)';
        ctx.shadowBlur = 5;
      }
      ctx.globalAlpha = isPlaying ? 0.4 + 0.6 * (dotScales[i] / 1.25) : 0.35;
      ctx.fillStyle = '#38bdf8';
      ctx.beginPath();
      ctx.arc(x, midY, r, 0, Math.PI * 2);
      ctx.fill();
      ctx.restore();
    }
  }

  // Trigger animation loop whenever state changes
  $effect(() => {
    // Read reactive dependencies
    const _p = isPlaying;
    const _s = style;
    if (canvasEl && animId === null) {
      animId = requestAnimationFrame(render);
    }
  });

  onMount(() => {
    animId = requestAnimationFrame(render);
    return () => {
      if (animId !== null) {
        cancelAnimationFrame(animId);
        animId = null;
      }
    };
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="equalizer-container size-{size} style-{style}"
  class:is-playing={isPlaying}
  class:is-paused={!isPlaying}
  onclick={cycleStyle}
  title="Equalizer: {style} (click to cycle style)"
  aria-label="Audio Equalizer {style}"
>
  <canvas
    bind:this={canvasEl}
    class="equalizer-canvas"
    width={size === 'md' ? (style === 'wave' ? 80 : 56) : (style === 'wave' ? 64 : 44)}
    height={size === 'md' ? 40 : 28}
    style="width: {size === 'md' ? (style === 'wave' ? 40 : 28) : (style === 'wave' ? 32 : 22)}px; height: {size === 'md' ? 20 : 14}px;"
  ></canvas>
</div>

<style>
  .equalizer-container {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    position: relative;
    cursor: pointer;
    user-select: none;
    flex-shrink: 0;
    transition: transform 0.15s ease, opacity 0.2s ease;
  }

  .equalizer-container:hover {
    transform: scale(1.12);
  }

  .equalizer-container:active {
    transform: scale(0.92);
  }

  /* Sizes */
  .size-sm {
    height: 14px;
    min-width: 22px;
  }

  .size-md {
    height: 20px;
    min-width: 28px;
  }

  .equalizer-canvas {
    display: block;
    pointer-events: none;
  }
</style>
