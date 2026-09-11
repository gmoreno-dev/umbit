// utilidades de formatação e tempo

// milissegundos -> "m:ss"
export function mmss(ms) {
  const s = Math.max(0, Math.floor((ms || 0) / 1000));
  const m = Math.floor(s / 60);
  const r = s % 60;
  return `${m}:${r < 10 ? '0' : ''}${r}`;
}

// posição interpolada: position_ms é a posição no instante at_ms
export function currentPosition(now) {
  if (!now || !now.track) return 0;
  let p = now.position_ms || 0;
  if (now.playing && now.at_ms) p += Date.now() - now.at_ms;
  const d = now.track.duration_ms || 0;
  if (d) p = Math.min(p, d);
  return Math.max(0, p);
}

// "artista 1, artista 2"
export function artistsOf(track) {
  if (!track || !track.artists || !track.artists.length) return '';
  return track.artists.join(', ');
}

// "título · artista"
export function trackLine(track) {
  if (!track) return '';
  const a = artistsOf(track);
  return a ? `${track.name} · ${a}` : track.name;
}

// contagem com separador de milhar em pt-br (1.204)
export function count(n) {
  return (n || 0).toLocaleString('pt-BR');
}

// normaliza texto: minúsculo, sem acento, sem espaço
export function normalize(text) {
  return String(text || '')
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/\s+/g, '')
    .toLowerCase();
}

// texto de erro vindo do núcleo (string) ou de uma exceção
export function errorText(err) {
  if (typeof err === 'string') return err;
  if (err && typeof err.message === 'string') return err.message;
  return 'algo deu errado';
}
