<script>
  // assinatura "gm" em pixel art: traço de 2 px, monocromática, cara de manuscrita
  let { width = 120 } = $props();
</script>

<svg class="sig" {width} height={width * 0.4} viewBox="0 0 120 48" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="square" stroke-linejoin="miter" shape-rendering="crispEdges" aria-label="assinatura gm">
  <!-- g: bojo aberto, haste e descendente que enrola -->
  <path d="M34 14 C 32 9, 18 9, 17 18 C 16 27, 30 28, 34 20" />
  <path d="M34 11 L34 30 C 34 40, 26 44, 19 39" />
  <!-- m: três hastes e um floreio final -->
  <path d="M48 30 L48 14 C 50 10, 57 10, 58 16 L58 30" />
  <path d="M58 16 C 60 10, 68 10, 69 16 L69 30 C 69 34, 74 34, 78 29" />
  <!-- risco de baixo -->
  <path d="M12 41 C 40 36, 70 45, 108 38" />
  <path d="M104 36 L108 38 L105 41" />
</svg>

<style>
  .sig {
    display: block;
  }
</style>
