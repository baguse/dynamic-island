<script lang="ts">
  import { onMount } from 'svelte';
  import { islandState } from '../state/island.svelte';
  import type { SkyTheme, SkyIntensity } from '../types';

  interface Props {
    isExpanded?: boolean;
  }

  let { isExpanded = false }: Props = $props();

  let canvasEl = $state<HTMLCanvasElement | null>(null);
  let animId: number | null = null;

  // Active theme calculation
  const enabled = $derived(islandState.settings.ambient_sky_enabled ?? true);
  const userTheme = $derived<SkyTheme>(islandState.settings.ambient_sky_theme ?? 'auto');
  const intensity = $derived<SkyIntensity>(islandState.settings.ambient_sky_intensity ?? 'subtle');

  const activeTheme = $derived.by<SkyTheme>(() => {
    if (userTheme !== 'auto') return userTheme;
    const w = islandState.weather;
    if (!w) {
      // Fallback based on client hour
      const hr = new Date().getHours();
      if (hr >= 5 && hr <= 6) return 'clouds';
      if (hr >= 17 && hr <= 18) return 'clouds';
      if (hr >= 19 || hr < 5) return 'stars';
      return 'sakura';
    }
    if (w.condition === 'rain' || w.condition === 'thunderstorm') return 'rain';
    if (w.condition === 'snow') return 'snow';
    if (w.time_period === 'sunset' || w.time_period === 'dawn') return 'clouds';
    if (w.time_period === 'night' || !w.is_day) return 'stars';
    return 'sakura';
  });

  const baseOpacity = $derived.by<number>(() => {
    switch (intensity) {
      case 'vivid':
        return 0.75;
      case 'medium':
        return 0.45;
      case 'subtle':
      default:
        return 0.22;
    }
  });

  // Particle models
  interface Star {
    x: number;
    y: number;
    size: number;
    baseAlpha: number;
    twinkleSpeed: number;
    phase: number;
  }

  interface ShootingStar {
    x: number;
    y: number;
    vx: number;
    vy: number;
    length: number;
    life: number;
    maxLife: number;
  }

  interface RainDrop {
    x: number;
    y: number;
    speed: number;
    len: number;
    alpha: number;
  }

  interface Splash {
    x: number;
    y: number;
    r: number;
    maxR: number;
    alpha: number;
  }

  interface SakuraPetal {
    x: number;
    y: number;
    size: number;
    vy: number;
    vx: number;
    swaySpeed: number;
    phase: number;
    rotation: number;
    rotSpeed: number;
    color: string;
  }

  interface CloudPuff {
    x: number;
    y: number;
    w: number;
    h: number;
    speed: number;
    alpha: number;
    color: string;
  }

  interface Snowflake {
    x: number;
    y: number;
    r: number;
    vy: number;
    vx: number;
    phase: number;
    alpha: number;
  }

  // Particle systems
  let stars: Star[] = [];
  let shootingStars: ShootingStar[] = [];
  let rainDrops: RainDrop[] = [];
  let splashes: Splash[] = [];
  let sakuraPetals: SakuraPetal[] = [];
  let clouds: CloudPuff[] = [];
  let snowflakes: Snowflake[] = [];

  let lastTime = 0;
  let nextShootingStar = 3000;
  let thunderTimer = 0;
  let thunderFlash = 0;

  function initParticles(w: number, h: number) {
    // Stars
    stars = Array.from({ length: 48 }, () => ({
      x: Math.random() * w,
      y: Math.random() * h,
      size: Math.random() < 0.25 ? 2.0 : 1.2,
      baseAlpha: 0.3 + Math.random() * 0.7,
      twinkleSpeed: 0.002 + Math.random() * 0.004,
      phase: Math.random() * Math.PI * 2,
    }));

    // Rain
    rainDrops = Array.from({ length: 42 }, () => ({
      x: Math.random() * (w + 100) - 50,
      y: Math.random() * h,
      speed: 7 + Math.random() * 8,
      len: 12 + Math.random() * 16,
      alpha: 0.35 + Math.random() * 0.55,
    }));

    // Sakura
    const petalColors = ['#f472b6', '#fb7185', '#fbcfe8', '#fda4af'];
    sakuraPetals = Array.from({ length: 26 }, () => ({
      x: Math.random() * w,
      y: Math.random() * h,
      size: 5 + Math.random() * 4,
      vy: 0.7 + Math.random() * 0.9,
      vx: 0.2 + Math.random() * 0.5,
      swaySpeed: 0.0015 + Math.random() * 0.002,
      phase: Math.random() * Math.PI * 2,
      rotation: Math.random() * Math.PI * 2,
      rotSpeed: (Math.random() - 0.5) * 0.02,
      color: petalColors[Math.floor(Math.random() * petalColors.length)],
    }));

    // Clouds
    const cloudColors = ['#f97316', '#fb923c', '#c084fc', '#818cf8'];
    clouds = Array.from({ length: 6 }, (_, i) => ({
      x: (w / 5) * i + Math.random() * 50,
      y: 15 + Math.random() * (h * 0.6),
      w: 80 + Math.random() * 90,
      h: 24 + Math.random() * 28,
      speed: 0.15 + Math.random() * 0.2,
      alpha: 0.12 + Math.random() * 0.16,
      color: cloudColors[i % cloudColors.length],
    }));

    // Snow
    snowflakes = Array.from({ length: 36 }, () => ({
      x: Math.random() * w,
      y: Math.random() * h,
      r: 1.2 + Math.random() * 2.2,
      vy: 0.6 + Math.random() * 0.8,
      vx: (Math.random() - 0.5) * 0.4,
      phase: Math.random() * Math.PI * 2,
      alpha: 0.35 + Math.random() * 0.55,
    }));
  }

  function render(time: number) {
    if (!canvasEl || !enabled) return;
    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;

    const dt = lastTime ? Math.min(time - lastTime, 64) : 16;
    lastTime = time;

    const w = canvasEl.width;
    const h = canvasEl.height;

    ctx.clearRect(0, 0, w, h);
    ctx.save();
    ctx.globalAlpha = baseOpacity;

    const theme = activeTheme;

    if (theme === 'stars') {
      renderStars(ctx, w, h, time, dt);
    } else if (theme === 'rain') {
      renderRain(ctx, w, h, dt);
    } else if (theme === 'sakura') {
      renderSakura(ctx, w, h, dt);
    } else if (theme === 'clouds') {
      renderClouds(ctx, w, h, dt);
    } else if (theme === 'snow') {
      renderSnow(ctx, w, h, dt);
    }

    ctx.restore();
    animId = requestAnimationFrame(render);
  }

  // 1. Cosmic Stars & Shooting Stars
  function renderStars(ctx: CanvasRenderingContext2D, w: number, h: number, time: number, dt: number) {
    // Subtle cosmic nebula glow
    const grad = ctx.createRadialGradient(w * 0.8, 10, 10, w * 0.8, 10, w * 0.6);
    grad.addColorStop(0, 'rgba(99, 102, 241, 0.18)');
    grad.addColorStop(1, 'rgba(15, 23, 42, 0)');
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, w, h);

    // Stars
    for (const star of stars) {
      star.phase += star.twinkleSpeed * dt;
      const alpha = star.baseAlpha * (0.5 + 0.5 * Math.sin(star.phase));
      ctx.fillStyle = `rgba(248, 250, 252, ${alpha})`;
      ctx.fillRect(star.x, star.y, star.size, star.size);
    }

    // Shooting stars
    nextShootingStar -= dt;
    if (nextShootingStar <= 0) {
      nextShootingStar = 5000 + Math.random() * 8000;
      shootingStars.push({
        x: Math.random() * (w * 0.7),
        y: Math.random() * (h * 0.3),
        vx: 8 + Math.random() * 6,
        vy: 3 + Math.random() * 3,
        length: 40 + Math.random() * 35,
        life: 0,
        maxLife: 500,
      });
    }

    for (let i = shootingStars.length - 1; i >= 0; i--) {
      const s = shootingStars[i];
      s.life += dt;
      s.x += s.vx;
      s.y += s.vy;
      const progress = s.life / s.maxLife;
      if (progress >= 1 || s.x > w || s.y > h) {
        shootingStars.splice(i, 1);
        continue;
      }
      const alpha = Math.sin(progress * Math.PI);
      const tailX = s.x - (s.vx / (s.vx + s.vy)) * s.length;
      const tailY = s.y - (s.vy / (s.vx + s.vy)) * s.length;

      const starGrad = ctx.createLinearGradient(tailX, tailY, s.x, s.y);
      starGrad.addColorStop(0, 'rgba(56, 189, 248, 0)');
      starGrad.addColorStop(1, `rgba(255, 255, 255, ${alpha * 0.9})`);

      ctx.beginPath();
      ctx.strokeStyle = starGrad;
      ctx.lineWidth = 1.8;
      ctx.moveTo(tailX, tailY);
      ctx.lineTo(s.x, s.y);
      ctx.stroke();
    }
  }

  // 2. Cyberpunk Neon Rain
  function renderRain(ctx: CanvasRenderingContext2D, w: number, h: number, dt: number) {
    // Lightning flash
    thunderTimer += dt;
    if (thunderTimer > 8000) {
      thunderTimer = 0;
      thunderFlash = 1.0;
    }
    if (thunderFlash > 0) {
      ctx.fillStyle = `rgba(129, 140, 248, ${thunderFlash * 0.25})`;
      ctx.fillRect(0, 0, w, h);
      thunderFlash -= 0.05;
    }

    const wind = -1.5;
    ctx.lineWidth = 1.2;

    for (const drop of rainDrops) {
      drop.x += wind * (dt / 16);
      drop.y += drop.speed * (dt / 16);

      if (drop.y > h) {
        // Splash ripple
        if (splashes.length < 15 && Math.random() < 0.4) {
          splashes.push({
            x: drop.x,
            y: h - 2,
            r: 1,
            maxR: 4 + Math.random() * 4,
            alpha: 0.6,
          });
        }
        drop.y = -drop.len;
        drop.x = Math.random() * (w + 100) - 50;
      }

      ctx.strokeStyle = `rgba(56, 189, 248, ${drop.alpha})`;
      ctx.beginPath();
      ctx.moveTo(drop.x, drop.y);
      ctx.lineTo(drop.x + wind * 2.5, drop.y + drop.len);
      ctx.stroke();
    }

    // Splashes
    for (let i = splashes.length - 1; i >= 0; i--) {
      const sp = splashes[i];
      sp.r += 0.35;
      sp.alpha -= 0.04;
      if (sp.alpha <= 0) {
        splashes.splice(i, 1);
        continue;
      }
      ctx.strokeStyle = `rgba(129, 140, 248, ${sp.alpha})`;
      ctx.beginPath();
      ctx.ellipse(sp.x, sp.y, sp.r * 1.8, sp.r * 0.6, 0, 0, Math.PI * 2);
      ctx.stroke();
    }
  }

  // 3. Sakura Cherry Blossom Petals
  function renderSakura(ctx: CanvasRenderingContext2D, w: number, h: number, dt: number) {
    for (const petal of sakuraPetals) {
      petal.phase += petal.swaySpeed * dt;
      petal.rotation += petal.rotSpeed;
      petal.x += (petal.vx + Math.sin(petal.phase) * 0.8) * (dt / 16);
      petal.y += petal.vy * (dt / 16);

      if (petal.y > h + 10) {
        petal.y = -10;
        petal.x = Math.random() * w;
      }
      if (petal.x > w + 10) petal.x = -10;
      if (petal.x < -10) petal.x = w + 10;

      ctx.save();
      ctx.translate(petal.x, petal.y);
      ctx.rotate(petal.rotation);
      ctx.fillStyle = petal.color;
      ctx.beginPath();
      // Draw organic petal teardrop shape
      ctx.ellipse(0, 0, petal.size, petal.size * 0.55, 0, 0, Math.PI * 2);
      ctx.fill();
      ctx.restore();
    }
  }

  // 4. Sunset Clouds & Twilight Glow
  function renderClouds(ctx: CanvasRenderingContext2D, w: number, h: number, dt: number) {
    // Twilight horizon gradient
    const sunsetGrad = ctx.createLinearGradient(0, 0, 0, h);
    sunsetGrad.addColorStop(0, 'rgba(67, 24, 110, 0.25)');
    sunsetGrad.addColorStop(0.5, 'rgba(180, 83, 9, 0.22)');
    sunsetGrad.addColorStop(1, 'rgba(234, 88, 12, 0.15)');
    ctx.fillStyle = sunsetGrad;
    ctx.fillRect(0, 0, w, h);

    // Drifting cloud puffs
    for (const cloud of clouds) {
      cloud.x += cloud.speed * (dt / 16);
      if (cloud.x - cloud.w > w) {
        cloud.x = -cloud.w;
        cloud.y = 15 + Math.random() * (h * 0.6);
      }

      ctx.fillStyle = cloud.color;
      ctx.globalAlpha = baseOpacity * cloud.alpha;
      ctx.beginPath();
      ctx.roundRect(cloud.x, cloud.y, cloud.w, cloud.h, cloud.h / 2);
      ctx.fill();
    }
  }

  // 5. Winter Snowfall
  function renderSnow(ctx: CanvasRenderingContext2D, w: number, h: number, dt: number) {
    for (const flake of snowflakes) {
      flake.phase += 0.002 * dt;
      flake.x += (flake.vx + Math.sin(flake.phase) * 0.5) * (dt / 16);
      flake.y += flake.vy * (dt / 16);

      if (flake.y > h + 5) {
        flake.y = -5;
        flake.x = Math.random() * w;
      }
      if (flake.x > w + 5) flake.x = -5;
      if (flake.x < -5) flake.x = w + 5;

      ctx.fillStyle = `rgba(224, 242, 254, ${flake.alpha})`;
      ctx.beginPath();
      ctx.arc(flake.x, flake.y, flake.r, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  function resizeCanvas() {
    if (!canvasEl) return;
    const rect = canvasEl.getBoundingClientRect();
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    canvasEl.width = Math.round(rect.width * dpr);
    canvasEl.height = Math.round(rect.height * dpr);
    initParticles(canvasEl.width, canvasEl.height);
  }

  $effect(() => {
    // React to expanded state changes
    const _exp = isExpanded;
    const _t = activeTheme;
    if (canvasEl) {
      resizeCanvas();
    }
  });

  onMount(() => {
    resizeCanvas();
    animId = requestAnimationFrame(render);

    const ro = new ResizeObserver(() => {
      resizeCanvas();
    });
    if (canvasEl) ro.observe(canvasEl);

    return () => {
      ro.disconnect();
      if (animId !== null) {
        cancelAnimationFrame(animId);
        animId = null;
      }
    };
  });
</script>

{#if enabled}
  <canvas
    bind:this={canvasEl}
    class="ambient-sky-canvas"
    class:is-expanded={isExpanded}
    aria-hidden="true"
  ></canvas>
{/if}

<style>
  .ambient-sky-canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    border-radius: inherit;
    z-index: 0;
    transition: opacity 0.3s ease;
  }
</style>
