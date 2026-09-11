// dithering ordenado (bayer 4×4), luminância, padrão procedural e cache de capas.
// tudo em 1 bit: cada pixel vira tinta ou papel.

export const BAYER = [
  [0, 8, 2, 10],
  [12, 4, 14, 6],
  [3, 11, 1, 9],
  [15, 7, 13, 5],
];

// limiares pré-calculados: (bayer[y%4][x%4] + 0.5) / 16, indexados por (y&3)*4 + (x&3)
const THRESHOLD = new Float32Array(16);
for (let y = 0; y < 4; y++) for (let x = 0; x < 4; x++) THRESHOLD[y * 4 + x] = (BAYER[y][x] + 0.5) / 16;

export function threshold(x, y) {
  return THRESHOLD[((y & 3) << 2) | (x & 3)];
}

// ImageData -> Float32Array de luminância (0..1)
export function lumaFromImageData(img) {
  const d = img.data;
  const n = img.width * img.height;
  const out = new Float32Array(n);
  for (let i = 0, o = 0; i < n; i++, o += 4) {
    out[i] = (0.2126 * d[o] + 0.7152 * d[o + 1] + 0.0722 * d[o + 2]) / 255;
  }
  return out;
}

// padrão procedural: gradiente diagonal (escuro no canto superior esquerdo,
// claro no inferior direito), como o quadrado ditherizado dos mockups
const proceduralCache = new Map();
export function proceduralLuma(w, h) {
  const key = w + 'x' + h;
  let l = proceduralCache.get(key);
  if (l) return l;
  l = new Float32Array(w * h);
  const span = Math.max(1, w + h - 2);
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      l[y * w + x] = 0.06 + 0.9 * ((x + y) / span);
    }
  }
  if (proceduralCache.size > 8) proceduralCache.delete(proceduralCache.keys().next().value);
  proceduralCache.set(key, l);
  return l;
}

// máscara de coração (1 = dentro), revelada de fora para dentro conforme `progress` (0..1).
// usa a curva implícita (x²+y²−1)³ − x²y³ < 0; a "profundidade" cresce da borda ao centro.
const heartCache = new Map();
export function heartDepth(w, h) {
  const key = w + 'x' + h;
  let d = heartCache.get(key);
  if (d) return d;
  d = new Float32Array(w * h);
  const sx = w * 0.36;
  const sy = h * 0.36;
  const cx = w / 2;
  const cy = h / 2 + h * 0.05;
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const nx = (x - cx) / sx;
      const ny = -(y - cy) / sy;
      const q = nx * nx + ny * ny - 1;
      const v = q * q * q - nx * nx * ny * ny * ny;
      d[y * w + x] = v < 0 ? Math.min(1, Math.cbrt(-v)) : -1; // -1 = fora
    }
  }
  if (heartCache.size > 4) heartCache.delete(heartCache.keys().next().value);
  heartCache.set(key, d);
  return d;
}

// pinta `out` (Uint8ClampedArray w*h*4) a partir da luminância.
// - luma2/mix: mistura entre duas imagens (transição de dithering)
// - heart: progresso (0..1) da máscara de coração; dentro dela a luminância é invertida
export function paint(out, w, h, luma, ink, paper, luma2 = null, mix = 0, heart = null) {
  const depth = heart != null ? heartDepth(w, h) : null;
  const m = Math.max(0, Math.min(1, mix));
  let o = 0;
  for (let y = 0; y < h; y++) {
    const row = (y & 3) << 2;
    const base = y * w;
    for (let x = 0; x < w; x++) {
      const i = base + x;
      let l = luma[i];
      if (luma2) l += (luma2[i] - l) * m;
      if (depth) {
        const dd = depth[i];
        if (dd >= 0 && dd <= heart) l = 1 - l;
      }
      const c = l < THRESHOLD[row | (x & 3)] ? ink : paper;
      out[o] = c[0];
      out[o + 1] = c[1];
      out[o + 2] = c[2];
      out[o + 3] = 255;
      o += 4;
    }
  }
}

// cache lru de luminâncias por url+tamanho (até 20 entradas)
const MAX_CACHE = 20;
const cache = new Map();
const pending = new Map();

function cacheGet(key) {
  const v = cache.get(key);
  if (v) {
    cache.delete(key);
    cache.set(key, v);
  }
  return v || null;
}
function cachePut(key, v) {
  cache.set(key, v);
  while (cache.size > MAX_CACHE) cache.delete(cache.keys().next().value);
}

let scratch = null;
function scratchCanvas(px) {
  if (!scratch) scratch = document.createElement('canvas');
  scratch.width = px;
  scratch.height = px;
  return scratch;
}

function toBlob(bytes) {
  if (bytes instanceof Blob) return bytes;
  if (bytes instanceof ArrayBuffer || ArrayBuffer.isView(bytes)) return new Blob([bytes]);
  if (Array.isArray(bytes)) return new Blob([new Uint8Array(bytes)]);
  throw new Error('capa em formato inesperado');
}

// carrega a capa (bytes via fetchBytes), redimensiona para px×px e devolve a luminância.
// chamadas simultâneas para a mesma chave compartilham a promessa.
export function loadLuma(url, px, fetchBytes) {
  const key = url + '@' + px;
  const hit = cacheGet(key);
  if (hit) return Promise.resolve(hit);
  let p = pending.get(key);
  if (p) return p;
  p = (async () => {
    const bytes = await fetchBytes(url);
    const bitmap = await createImageBitmap(toBlob(bytes));
    try {
      const c = scratchCanvas(px);
      const ctx = c.getContext('2d', { willReadFrequently: true });
      ctx.clearRect(0, 0, px, px);
      ctx.drawImage(bitmap, 0, 0, px, px);
      const luma = lumaFromImageData(ctx.getImageData(0, 0, px, px));
      cachePut(key, luma);
      return luma;
    } finally {
      if (bitmap.close) bitmap.close();
    }
  })();
  pending.set(key, p);
  p.finally(() => pending.delete(key)).catch(() => {});
  return p;
}

export function cachedLuma(url, px) {
  return url ? cacheGet(url + '@' + px) : null;
}
