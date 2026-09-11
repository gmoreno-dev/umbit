// mock do núcleo para rodar no navegador (npm run dev sem tauri).
// imita todos os comandos e eventos do contrato, com dados falsos.
//
// chaves de localStorage úteis para testar:
//   umbit.mock.birthday = '1'   -> modo aniversário
//   umbit.mock.egg      = '0'   -> desliga os easter eggs
//   umbit.mock.loginfail= '1'   -> o login falha com erro
//   umbit.mock.logged   = '1'   -> começa logado (gravado pelo próprio login)

import { normalize } from './format.js';

const clone = (x) => JSON.parse(JSON.stringify(x));
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const ls = {
  get: (k) => {
    try {
      return localStorage.getItem(k);
    } catch {
      return null;
    }
  },
  set: (k, v) => {
    try {
      if (v == null) localStorage.removeItem(k);
      else localStorage.setItem(k, v);
    } catch {
      /* sem armazenamento */
    }
  },
};

// ---------- dados ----------

const T = (i, name, artist, album, m, s) => ({
  uri: `spotify:track:mock${String(i).padStart(2, '0')}`,
  name,
  artists: Array.isArray(artist) ? artist : [artist],
  album,
  duration_ms: (m * 60 + s) * 1000,
  cover: null,
});

const TRACKS = [
  T(1, 'Cantaloupe Island', 'Herbie Hancock', 'Empyrean Isles', 5, 32),
  T(2, 'Watermelon Man', 'Herbie Hancock', 'Head Hunters', 6, 29),
  T(3, 'Chameleon', 'Herbie Hancock', 'Head Hunters', 15, 41),
  T(4, 'The Egg', 'Herbie Hancock', 'Empyrean Isles', 14, 0),
  T(5, 'Maiden Voyage', 'Herbie Hancock', 'Maiden Voyage', 7, 57),
  T(6, 'Blue in Green', 'Miles Davis', 'Kind of Blue', 5, 37),
  T(7, 'So What', 'Miles Davis', 'Kind of Blue', 9, 22),
  T(8, 'Naima', 'John Coltrane', 'Giant Steps', 4, 21),
  T(9, 'Giant Steps', 'John Coltrane', 'Giant Steps', 4, 43),
  T(10, 'Peace Piece', 'Bill Evans', 'Everybody Digs Bill Evans', 6, 44),
  T(11, 'Waltz for Debby', 'Bill Evans', 'Waltz for Debby', 6, 54),
  T(12, 'Take Five', 'Dave Brubeck', 'Time Out', 5, 24),
  T(13, 'Águas de Março', ['Elis Regina', 'Tom Jobim'], 'Elis & Tom', 3, 32),
  T(14, 'Travessia', 'Milton Nascimento', 'Milton Nascimento', 3, 39),
  T(15, 'Clube da Esquina Nº 2', ['Milton Nascimento', 'Lô Borges'], 'Clube da Esquina', 3, 41),
  T(16, 'Ponta de Areia', 'Milton Nascimento', 'Minas', 4, 6),
  T(17, 'Construção', 'Chico Buarque', 'Construção', 6, 24),
  T(18, 'Panis et Circenses', 'Os Mutantes', 'Os Mutantes', 3, 39),
  T(19, 'Baby', ['Os Mutantes', 'Gal Costa'], 'Os Mutantes', 3, 26),
  T(20, 'Avarandado', 'Caetano Veloso', 'Caetano Veloso', 2, 54),
  T(21, 'An Ending (Ascent)', 'Brian Eno', 'Apollo', 4, 22),
  T(22, 'Rhubarb', 'Aphex Twin', 'Selected Ambient Works Volume II', 7, 43),
  T(23, 'Xtal', 'Aphex Twin', 'Selected Ambient Works 85-92', 4, 51),
  T(24, 'Music for Airports 1/1', 'Brian Eno', 'Ambient 1: Music for Airports', 17, 21),
  T(25, 'Teardrop', 'Massive Attack', 'Mezzanine', 5, 30),
  T(26, 'Roads', 'Portishead', 'Dummy', 5, 2),
  T(27, 'Nude', 'Radiohead', 'In Rainbows', 4, 15),
  T(28, 'Pyramid Song', 'Radiohead', 'Amnesiac', 4, 49),
  T(29, 'Holocene', 'Bon Iver', 'Bon Iver, Bon Iver', 5, 36),
  T(30, 'Re: Stacks', 'Bon Iver', 'For Emma, Forever Ago', 6, 41),
];

const ALBUM_YEARS = {
  'Empyrean Isles': '1964',
  'Head Hunters': '1973',
  'Maiden Voyage': '1965',
  'Kind of Blue': '1959',
  'Giant Steps': '1960',
  'Everybody Digs Bill Evans': '1959',
  'Waltz for Debby': '1962',
  'Time Out': '1959',
  'Elis & Tom': '1974',
  'Milton Nascimento': '1969',
  'Clube da Esquina': '1972',
  Minas: '1975',
  Construção: '1971',
  'Os Mutantes': '1968',
  'Caetano Veloso': '1968',
  Apollo: '1983',
  'Selected Ambient Works Volume II': '1994',
  'Selected Ambient Works 85-92': '1992',
  'Ambient 1: Music for Airports': '1978',
  Mezzanine: '1998',
  Dummy: '1994',
  'In Rainbows': '2007',
  Amnesiac: '2001',
  'Bon Iver, Bon Iver': '2011',
  'For Emma, Forever Ago': '2007',
};

// álbuns únicos derivados das faixas
const ALBUMS = [];
for (const t of TRACKS) {
  let a = ALBUMS.find((x) => x.name === t.album);
  if (!a) {
    a = {
      uri: `spotify:album:mock${ALBUMS.length + 1}`,
      name: t.album,
      artists: [t.artists[0]],
      year: ALBUM_YEARS[t.album] || null,
      track_count: 0,
      cover: null,
    };
    ALBUMS.push(a);
  }
  a.track_count++;
}

const ARTISTS = [];
for (const t of TRACKS) {
  for (const n of t.artists) {
    if (!ARTISTS.find((a) => a.name === n)) ARTISTS.push({ uri: `spotify:artist:mock${ARTISTS.length + 1}`, name: n, cover: null });
  }
}

const PLAYLISTS = [
  ['Jazz de madrugada', 142],
  ['Foco profundo', 86],
  ['Trabalho sem letra', 310],
  ['Descobertas da semana', 30],
  ['Brasil 70', 58],
  ['Eletrônica lenta', 97],
  ['Chuva e café', 45],
  ['Domingo de manhã', 23],
].map(([name, n], i) => ({ uri: `spotify:playlist:mock${i + 1}`, name, owner: 'moreno', track_count: n, cover: null }));

const LIKED_URI = 'spotify:user:moreno:collection';
const LIKED_TOTAL = 120;
const SAVED_ALBUMS = ALBUMS.slice(0, 9);

// faixa i de uma lista grande, derivada das 30 base (uri única por posição)
function derived(base, i, tag) {
  const t = clone(base);
  if (i >= TRACKS.length) {
    t.uri = `${base.uri}:${tag}:${i}`;
    t.name = `${base.name} (take ${Math.floor(i / TRACKS.length) + 1})`;
  }
  return t;
}
function playlistTrack(pi, i) {
  return derived(TRACKS[(i * 7 + pi * 3) % TRACKS.length], i, 'p' + pi);
}
function likedTrack(i) {
  return derived(TRACKS[(i * 11 + 5) % TRACKS.length], i, 'liked');
}
function albumTracks(album) {
  return TRACKS.filter((t) => t.album === album.name).map(clone);
}

// ---------- easter egg ----------

const MATILDA = {
  uri: 'spotify:track:egg:matilda',
  name: 'Matilda',
  artists: ['Harry Styles'],
  album: "Harry's House",
  duration_ms: 245000,
  cover: 'mock://matilda',
};
const OURS = {
  uri: 'spotify:track:egg:ours',
  name: 'Sign of the Times',
  artists: ['Harry Styles'],
  album: 'Harry Styles',
  duration_ms: 340000,
  cover: 'mock://ours',
};

const eggOn = () => ls.get('umbit.mock.egg') !== '0';

// ---------- estado ----------

const listeners = new Map();
function emit(name, payload) {
  const set = listeners.get(name);
  if (!set) return;
  for (const cb of set) {
    try {
      cb(clone(payload));
    } catch (e) {
      console.error('mock listener', e);
    }
  }
}

const session = {
  logged_in: ls.get('umbit.mock.logged') === '1',
  connecting: false,
  username: 'moreno',
  display_name: 'Moreno',
  easter_egg: eggOn(),
  birthday: ls.get('umbit.mock.birthday') === '1',
  error: null,
};

const now = {
  track: null,
  playing: false,
  loading: false,
  position_ms: 0,
  at_ms: Date.now(),
  volume: 60,
  shuffle: false,
  repeat: false,
  active: true,
  egg: false,
};

const queue = { context_name: '', context_uri: null, current: null, current_index: 0, upcoming: [], total: 0, note: null };

let list = []; // faixas do contexto atual
let index = 0;
let endTimer = null;
let loadTimer = null;

function currentPos() {
  if (!now.track) return 0;
  let p = now.position_ms;
  if (now.playing) p += Date.now() - now.at_ms;
  return Math.min(p, now.track.duration_ms);
}

function syncQueue() {
  queue.current = list[index] ? clone(list[index]) : null;
  queue.current_index = index;
  queue.upcoming = list.slice(index + 1).map(clone);
  queue.total = list.length;
  emit('queue', queue);
}

function scheduleEnd() {
  clearTimeout(endTimer);
  if (!now.track || !now.playing) return;
  const rest = now.track.duration_ms - currentPos();
  endTimer = setTimeout(() => {
    if (index + 1 < list.length) startTrack(index + 1);
    else {
      now.playing = false;
      now.position_ms = now.track.duration_ms;
      now.at_ms = Date.now();
      emit('now_playing', now);
    }
  }, Math.max(50, rest));
}

function startTrack(i) {
  clearTimeout(endTimer);
  clearTimeout(loadTimer);
  index = i;
  const t = list[i];
  now.track = clone(t);
  now.loading = true;
  now.playing = false;
  now.position_ms = 0;
  now.at_ms = Date.now();
  now.egg = t.extra === 'egg-ours' || t.extra === 'egg-hers';
  emit('now_playing', now);
  syncQueue();
  // "carregando" por um instante, como o núcleo real
  loadTimer = setTimeout(() => {
    now.loading = false;
    now.playing = true;
    now.position_ms = 0;
    now.at_ms = Date.now();
    emit('now_playing', now);
    scheduleEnd();
  }, 400);
}

function requireLogin() {
  if (!session.logged_in) throw 'faça login primeiro';
}

async function tracksForContext(uri) {
  if (uri === LIKED_URI) return Array.from({ length: LIKED_TOTAL }, (_, i) => likedTrack(i));
  const pi = PLAYLISTS.findIndex((p) => p.uri === uri);
  if (pi >= 0) return Array.from({ length: PLAYLISTS[pi].track_count }, (_, i) => playlistTrack(pi, i));
  const al = ALBUMS.find((a) => a.uri === uri);
  if (al) return albumTracks(al);
  throw 'contexto não encontrado';
}

// capa sintética (png) para as urls mock://, desenhada num canvas
async function syntheticCover(url) {
  const seed = [...url].reduce((h, c) => (h * 31 + c.charCodeAt(0)) >>> 0, 7);
  const rnd = () => {
    let x = (seed * 9301 + 49297) % 233280;
    return () => (x = (x * 9301 + 49297) % 233280) / 233280;
  };
  const r = rnd();
  const size = 256;
  const c = document.createElement('canvas');
  c.width = c.height = size;
  const ctx = c.getContext('2d');
  const g = ctx.createLinearGradient(0, 0, size, size);
  g.addColorStop(0, `hsl(${r() * 360}, 40%, ${20 + r() * 20}%)`);
  g.addColorStop(1, `hsl(${r() * 360}, 40%, ${60 + r() * 30}%)`);
  ctx.fillStyle = g;
  ctx.fillRect(0, 0, size, size);
  for (let i = 0; i < 6; i++) {
    ctx.beginPath();
    ctx.arc(r() * size, r() * size, 20 + r() * 70, 0, Math.PI * 2);
    ctx.fillStyle = `hsla(${r() * 360}, 60%, ${r() * 100}%, 0.8)`;
    ctx.fill();
  }
  const blob = await new Promise((res) => c.toBlob(res, 'image/png'));
  return blob.arrayBuffer();
}

// ---------- comandos ----------

const commands = {
  async get_session() {
    return session;
  },

  async login() {
    if (session.logged_in) return;
    session.connecting = true;
    session.error = null;
    emit('session', session);
    await sleep(1000);
    if (ls.get('umbit.mock.loginfail') === '1') {
      session.connecting = false;
      session.error = 'o spotify não autorizou. tenta de novo?';
      emit('session', session);
      throw session.error;
    }
    session.connecting = false;
    session.logged_in = true;
    ls.set('umbit.mock.logged', '1');
    emit('session', session);
  },

  async logout() {
    clearTimeout(endTimer);
    clearTimeout(loadTimer);
    session.logged_in = false;
    session.connecting = false;
    ls.set('umbit.mock.logged', null);
    now.track = null;
    now.playing = false;
    now.egg = false;
    list = [];
    index = 0;
    emit('session', session);
    emit('now_playing', now);
    syncQueue();
  },

  async get_config() {
    return {
      theme: ls.get('umbit.mock.theme') || 'papel',
      device_name: 'umbit (mock)',
      client_id: ls.get('umbit.mock.client_id') || '',
      egg_theme_ink: '#5a1e3a',
      egg_theme_paper: '#f7e6ee',
      version: '0.1.0',
    };
  },

  async check_update({ force } = {}) {
    // no navegador não há o que atualizar; `umbit.mock.update='1'` simula uma versão nova (manual)
    if (ls.get('umbit.mock.update') === '1') return { version: '9.9.9', notes: null, auto: false };
    return null;
  },
  async install_update() {
    throw 'nesta instalação a atualização é manual';
  },
  async set_client_id({ clientId }) {
    const id = String(clientId || '').trim();
    if (id && id.length !== 32) throw 'um client id tem 32 caracteres';
    ls.set('umbit.mock.client_id', id || null);
  },
  async open_dashboard() {
    console.log('[mock] abriria https://developer.spotify.com/dashboard');
  },
  async set_theme({ theme }) {
    ls.set('umbit.mock.theme', theme);
  },

  async get_now_playing() {
    return now;
  },

  async get_queue() {
    return queue;
  },

  async get_playlists({ offset = 0 } = {}) {
    requireLogin();
    await sleep(120);
    return { items: PLAYLISTS.slice(offset, offset + 50), total: PLAYLISTS.length, offset };
  },

  async get_playlist_tracks({ uri, offset = 0 }) {
    requireLogin();
    await sleep(150);
    const pi = PLAYLISTS.findIndex((p) => p.uri === uri);
    if (pi < 0) throw 'playlist não encontrada';
    const total = PLAYLISTS[pi].track_count;
    const end = Math.min(total, offset + 100);
    const items = [];
    for (let i = offset; i < end; i++) items.push(playlistTrack(pi, i));
    return { items, total, offset };
  },

  async get_liked({ offset = 0 } = {}) {
    requireLogin();
    await sleep(150);
    const end = Math.min(LIKED_TOTAL, offset + 50);
    const items = [];
    for (let i = offset; i < end; i++) items.push(likedTrack(i));
    return { items, total: LIKED_TOTAL, offset };
  },

  async get_liked_uri() {
    return LIKED_URI;
  },

  async get_saved_albums({ offset = 0 } = {}) {
    requireLogin();
    await sleep(120);
    return { items: SAVED_ALBUMS.slice(offset, offset + 50).map(clone), total: SAVED_ALBUMS.length, offset };
  },

  async get_album_tracks({ uri }) {
    requireLogin();
    await sleep(120);
    const al = ALBUMS.find((a) => a.uri === uri);
    if (!al) throw 'álbum não encontrado';
    const items = albumTracks(al);
    return { items, total: items.length, offset: 0 };
  },

  async search({ query }) {
    requireLogin();
    await sleep(200);
    const q = String(query || '').trim();
    const n = normalize(q);
    const res = { query: q, tracks: [], albums: [], artists: [] };
    if (!n) return res;
    const m = q.match(/^artist:"?(.+?)"?$/i);
    if (m) {
      const an = normalize(m[1]);
      res.tracks = TRACKS.filter((t) => t.artists.some((a) => normalize(a) === an)).map(clone);
      res.albums = ALBUMS.filter((a) => a.artists.some((x) => normalize(x) === an)).map(clone);
      res.artists = ARTISTS.filter((a) => normalize(a.name) === an).map(clone);
      return res;
    }
    const hit = (s) => normalize(s).includes(n);
    res.tracks = TRACKS.filter((t) => hit(t.name) || t.artists.some(hit) || hit(t.album))
      .slice(0, 12)
      .map(clone);
    res.albums = ALBUMS.filter((a) => hit(a.name) || a.artists.some(hit))
      .slice(0, 6)
      .map(clone);
    res.artists = ARTISTS.filter((a) => hit(a.name))
      .slice(0, 4)
      .map(clone);
    // busca "ana livia": a matilda entra no topo, com a provocação no artista
    if (n === 'analivia' && eggOn()) res.tracks.unshift({ ...clone(MATILDA), extra: 'egg' });
    return res;
  },

  async play_context({ uri, index: idx, trackUri, name, tracks }) {
    requireLogin();
    list = tracks && tracks.length ? tracks.map(clone) : await tracksForContext(uri);
    let i = 0;
    if (trackUri) i = list.findIndex((t) => t.uri === trackUri);
    if (i < 0) i = typeof idx === 'number' ? idx : 0;
    if (i < 0 || i >= list.length) throw 'faixa não encontrada';
    queue.context_uri = uri || null;
    queue.context_name = name || '';
    queue.note = null;
    startTrack(i);
  },

  async play_tracks({ tracks, name }) {
    requireLogin();
    if (!tracks || !tracks.length) throw 'nada para tocar';
    list = tracks.map(clone);
    queue.context_uri = null;
    queue.context_name = name || '';
    queue.note = null;
    startTrack(0);
  },

  async play_egg() {
    requireLogin();
    if (!eggOn()) throw 'nada aqui';
    list = [{ ...clone(MATILDA), extra: 'egg-hers' }, { ...clone(OURS), extra: 'egg-ours' }];
    queue.context_uri = null;
    queue.context_name = 'pra ana lívia';
    queue.note = 'primeiro a sua, depois a nossa';
    startTrack(0);
  },

  async get_egg_tracks() {
    if (!eggOn()) return null;
    return [{ ...clone(MATILDA), extra: 'egg' }, { ...clone(OURS), extra: 'egg-ours' }];
  },

  async play_pause() {
    requireLogin();
    if (!now.track) throw 'nada tocando';
    if (now.loading) return;
    const p = currentPos();
    now.position_ms = p;
    now.at_ms = Date.now();
    now.playing = !now.playing;
    emit('now_playing', now);
    scheduleEnd();
  },

  async next_track() {
    requireLogin();
    if (index + 1 >= list.length) throw 'fim da fila';
    startTrack(index + 1);
  },

  async prev_track() {
    requireLogin();
    if (!now.track) throw 'nada tocando';
    // como no spotify: volta ao início se já passou de 3 s
    if (currentPos() > 3000 || index === 0) {
      now.position_ms = 0;
      now.at_ms = Date.now();
      emit('now_playing', now);
      scheduleEnd();
      return;
    }
    startTrack(index - 1);
  },

  async seek({ positionMs }) {
    requireLogin();
    if (!now.track) throw 'nada tocando';
    now.position_ms = Math.max(0, Math.min(now.track.duration_ms, positionMs | 0));
    now.at_ms = Date.now();
    emit('now_playing', now);
    scheduleEnd();
  },

  async seek_relative({ deltaMs }) {
    return commands.seek({ positionMs: currentPos() + (deltaMs | 0) });
  },

  async set_volume({ volume }) {
    now.volume = Math.max(0, Math.min(100, volume | 0));
    emit('now_playing', now);
  },

  async get_cover({ url }) {
    if (!url) throw 'sem capa';
    if (url.startsWith('mock://')) return syntheticCover(url);
    throw 'capa indisponível no mock';
  },
};

export const mock = {
  async invoke(name, args = {}) {
    const fn = commands[name];
    if (!fn) throw `comando desconhecido: ${name}`;
    const r = await fn(args);
    return r && typeof r === 'object' && !(r instanceof ArrayBuffer) ? clone(r) : r;
  },
  listen(name, cb) {
    if (!listeners.has(name)) listeners.set(name, new Set());
    listeners.get(name).add(cb);
    return () => listeners.get(name)?.delete(cb);
  },
};
