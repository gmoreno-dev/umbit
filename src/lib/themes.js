// temas: um par tinta/papel por nome. o tema "livia" vem do núcleo (get_config).

export const THEMES = {
  papel: ['#141210', '#f2efe6'],
  ambar: ['#ffb000', '#0b0a08'],
  fosforo: ['#5cf08a', '#06100a'],
  gameboy: ['#0f380f', '#9bbc0f'],
};

export const THEME_ORDER = ['papel', 'ambar', 'fosforo', 'gameboy'];

// temas automáticos do easter egg, só enquanto a faixa toca (nunca persistem):
// a dela = roxo escuro; a nossa = roxo sobre cinza.
export const EGG_THEMES = {
  'egg-next': ['#e9dcff', '#1c0f2e'],
  'egg-ours': ['#4a2a7a', '#cfc9d6'],
};

// "#rrggbb" -> [r, g, b]
export function hexToRgb(hex) {
  const h = String(hex || '').replace('#', '');
  const v = h.length === 3 ? h.split('').map((c) => c + c).join('') : h.padEnd(6, '0');
  const n = parseInt(v.slice(0, 6), 16);
  if (Number.isNaN(n)) return [0, 0, 0];
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

// luminância relativa aproximada (0..1) para decidir o color-scheme
function luma(hex) {
  const [r, g, b] = hexToRgb(hex);
  return (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255;
}

// escreve as variáveis no :root e ajusta o color-scheme conforme o papel
export function applyThemeColors(ink, paper) {
  const root = document.documentElement;
  root.style.setProperty('--ink', ink);
  root.style.setProperty('--paper', paper);
  root.style.colorScheme = luma(paper) > 0.5 ? 'light' : 'dark';
}
