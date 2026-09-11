<script>
  // capa ditherizada em 1 bit num canvas. sem url (ou carregando) desenha o padrão procedural.
  // - from/mix: transição de dithering entre a capa `from` (mix=0) e `url` (mix=1)
  // - heart: progresso (0..1) da máscara de coração (aniversário); null = sem máscara
  import { getCover } from '../lib/backend.js';
  import { loadLuma, cachedLuma, proceduralLuma, paint } from '../lib/dither.js';
  import { hexToRgb } from '../lib/themes.js';

  let { url = null, size = 100, ink = '#141210', paper = '#f2efe6', frame = false, thumb = false, from = null, mix = 1, heart = null } = $props();

  let canvas = $state(null);
  let luma = $state.raw(null);
  let fromLuma = $state.raw(null);
  let imageData = null;

  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const border = $derived(frame ? 2 : thumb ? 1 : 0);
  const inner = $derived(Math.max(1, size - border * 2));
  const px = $derived(Math.max(1, Math.round(inner * dpr)));

  // carrega a luminância de uma url (cache primeiro); devolve o cancelador
  function load(u, p, set) {
    set(null);
    if (!u) return undefined;
    const hit = cachedLuma(u, p);
    if (hit) {
      set(hit);
      return undefined;
    }
    let alive = true;
    loadLuma(u, p, getCover)
      .then((l) => {
        if (alive) set(l);
      })
      .catch(() => {});
    return () => {
      alive = false;
    };
  }

  $effect(() => load(url, px, (l) => (luma = l)));
  $effect(() => load(from, px, (l) => (fromLuma = l)));

  // repinta sempre que muda a luminância, o tema, o tamanho, a mistura ou a máscara
  $effect(() => {
    if (!canvas) return;
    const w = px;
    const ctx = canvas.getContext('2d');
    if (!imageData || imageData.width !== w) imageData = ctx.createImageData(w, w);
    const target = luma || proceduralLuma(w, w);
    const inkRgb = hexToRgb(ink);
    const paperRgb = hexToRgb(paper);
    if (from) {
      const origin = fromLuma || proceduralLuma(w, w);
      paint(imageData.data, w, w, origin, inkRgb, paperRgb, target, mix, heart);
    } else {
      paint(imageData.data, w, w, target, inkRgb, paperRgb, null, 0, heart);
    }
    ctx.putImageData(imageData, 0, 0);
  });
</script>

<div class="cover" class:frame class:thumb style:width="{size}px" style:height="{size}px">
  <canvas bind:this={canvas} width={px} height={px} style:width="{inner}px" style:height="{inner}px"></canvas>
</div>

<style>
  .cover {
    position: relative;
    overflow: hidden;
    flex-shrink: 0;
    background: var(--paper);
  }
  .frame {
    border: 2px solid var(--ink);
    box-shadow: 6px 6px 0 var(--ink);
  }
  .thumb {
    border: 1px solid var(--ink);
  }
  canvas {
    display: block;
    image-rendering: pixelated;
  }
</style>
