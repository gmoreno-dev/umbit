// estado global (runes) e ações de alto nível: init, telas, tema, toast, comandos ":"

import * as api from './backend.js';
import { THEMES, THEME_ORDER, EGG_THEMES, applyThemeColors } from './themes.js';
import { errorText } from './format.js';

export const SCREENS = ['playing', 'library', 'search', 'queue'];
const LIVIA_KEY = 'umbit.livia';

function emptyList() {
  return { items: [], total: 0, loaded: false, loading: false };
}

export const app = $state({
  ready: false,
  screen: 'login', // login | playing | library | search | queue | about
  prevScreen: 'playing',

  session: { logged_in: false, connecting: false, username: '', display_name: '', easter_egg: false, birthday: false, error: null },
  now: { track: null, playing: false, loading: false, position_ms: 0, at_ms: 0, volume: 50, shuffle: false, repeat: false, active: false, egg: false },
  queue: { context_name: '', context_uri: null, current: null, current_index: 0, upcoming: [], total: 0, note: null },
  config: { theme: 'papel', device_name: 'umbit', client_id: '', egg_theme_ink: '#5a1e3a', egg_theme_paper: '#f7e6ee', version: '0.1.0' },

  theme: 'papel',
  ink: THEMES.papel[0],
  paper: THEMES.papel[1],
  liviaUnlocked: false,

  toast: null, // { text, id }
  cmd: null, // string enquanto a linha de comando está aberta

  // estado das telas (sobrevive à troca de aba)
  library: {
    section: 'playlists', // playlists | liked | albums
    cursor: 0,
    open: null, // { kind: 'playlist'|'album', uri, name, items, total, loading }
    openFromCursor: 0, // cursor de onde a lista foi aberta, para voltar
    playlists: emptyList(),
    liked: emptyList(),
    albums: emptyList(),
    likedUri: null,
    eggTracks: null,
  },
  search: {
    query: '',
    results: null,
    loading: false,
    cursor: -1, // -1 = foco no campo
    open: null, // álbum aberto: { uri, name, items, loading }
    openFromCursor: 0,
    wantFocus: false, // "/" pede o foco no campo
  },
  queueCursor: 0,
});

// memória não reativa (de propósito): coisas que não devem disparar efeitos
export const memo = { revealedUri: null };

// ---------- toast ----------

let toastTimer = null;
export function toast(text) {
  clearTimeout(toastTimer);
  app.toast = { text: String(text), id: Date.now() };
  toastTimer = setTimeout(() => (app.toast = null), 3000);
}

// chama um comando e mostra o erro num toast; devolve undefined em caso de erro
export async function act(name, args) {
  try {
    return await api.call(name, args);
  } catch (err) {
    toast(errorText(err));
    return undefined;
  }
}

// ---------- telas ----------

export function setScreen(id) {
  if (!app.session.logged_in) return;
  if (id === app.screen) return;
  if (id === 'about') app.prevScreen = SCREENS.includes(app.screen) ? app.screen : 'playing';
  app.screen = id;
  app.cmd = null;
}

export function cycleScreen(dir = 1) {
  const i = SCREENS.indexOf(app.screen);
  const base = i < 0 ? 0 : i;
  setScreen(SCREENS[(base + dir + SCREENS.length) % SCREENS.length]);
}

export function goBackFromAbout() {
  setScreen(app.prevScreen || 'playing');
}

// ---------- temas ----------

function liviaAllowed() {
  return app.liviaUnlocked && app.session.easter_egg;
}

function themePair(name) {
  if (THEMES[name]) return THEMES[name];
  if (name === 'livia' && liviaAllowed()) return [app.config.egg_theme_ink, app.config.egg_theme_paper];
  return null;
}

export function availableThemes() {
  return liviaAllowed() ? [...THEME_ORDER, 'livia'] : THEME_ORDER;
}

// par de cores a usar agora: o do easter egg enquanto uma das duas faixas toca, senão o do tema
function eggPair() {
  if (!app.session.easter_egg || !app.now.egg || !app.now.track) return null;
  return EGG_THEMES[app.now.track.extra] || null;
}

function paint() {
  const pair = eggPair() || themePair(app.theme) || THEMES.papel;
  app.ink = pair[0];
  app.paper = pair[1];
  applyThemeColors(pair[0], pair[1]);
}

// chamado a cada mudança de reprodução: liga ou desliga o tema automático do easter egg
export function syncEggTheme() {
  paint();
}

export function applyTheme(name, persist = true) {
  const pair = themePair(name);
  if (!pair) return false;
  app.theme = name;
  paint();
  if (persist) api.call('set_theme', { theme: name }).catch(() => {});
  return true;
}

export function cycleTheme() {
  const list = availableThemes();
  const i = list.indexOf(app.theme);
  applyTheme(list[(i + 1) % list.length]);
}

export function unlockLivia() {
  if (!app.session.easter_egg) return;
  try {
    localStorage.setItem(LIVIA_KEY, '1');
  } catch {
    /* sem armazenamento */
  }
  app.liviaUnlocked = true;
  applyTheme('livia');
  toast('tema desbloqueado ♥');
}

// ---------- sessão ----------

export async function login() {
  if (app.session.connecting) return;
  app.session.connecting = true;
  app.session.error = null;
  try {
    // o client id digitado na tela de login vai para a configuração antes de entrar
    await api.call('set_client_id', { clientId: app.config.client_id || '' });
    await api.call('login');
    // o núcleo normalmente emite "session"; confirma de qualquer jeito
    const s = await api.call('get_session');
    if (s) applySession(s);
  } catch (err) {
    app.session.connecting = false;
    app.session.error = errorText(err);
  }
}

export async function logout() {
  await act('logout');
}

function resetData() {
  app.library.open = null;
  app.library.cursor = 0;
  app.library.playlists = emptyList();
  app.library.liked = emptyList();
  app.library.albums = emptyList();
  app.library.eggTracks = null;
  app.search.results = null;
  app.search.open = null;
  app.search.cursor = -1;
  app.queueCursor = 0;
  memo.revealedUri = null;
}

function applySession(s) {
  const wasLogged = app.session.logged_in;
  app.session = s;
  if (!s.logged_in) {
    if (app.screen !== 'login') {
      app.screen = 'login';
      app.cmd = null;
    }
    if (wasLogged) resetData();
  } else if (app.screen === 'login') {
    app.screen = 'playing';
  }
  // o tema "livia" só vale com o easter egg ligado
  if (app.theme === 'livia' && !liviaAllowed()) applyTheme('papel', false);
  else syncEggTheme();
}

// ---------- linha de comando ----------

export function openCommand() {
  app.cmd = '';
}
export function closeCommand() {
  app.cmd = null;
}

export async function runCommand(line) {
  const [cmd, ...rest] = String(line || '')
    .trim()
    .replace(/^:/, '')
    .split(/\s+/);
  const arg = rest.join(' ');
  closeCommand();
  switch (cmd) {
    case '':
      return;
    case 'sobre':
      setScreen('about');
      return;
    case 'tema':
      if (!arg) {
        toast(`tema: ${app.theme}`);
        return;
      }
      if (!applyTheme(arg)) toast('tema desconhecido');
      return;
    case 'sair':
      await logout();
      return;
    case 'topo':
      try {
        const on = await api.toggleAlwaysOnTop();
        toast(on ? 'sempre no topo' : 'topo desligado');
      } catch (err) {
        toast(errorText(err));
      }
      return;
    default:
      toast(`comando desconhecido: ${cmd}`);
  }
}

// ---------- inicialização ----------

export async function init() {
  try {
    app.liviaUnlocked = localStorage.getItem(LIVIA_KEY) === '1';
  } catch {
    app.liviaUnlocked = false;
  }

  const safe = (name, args, fallback) => api.call(name, args).catch(() => fallback);
  const [config, session, now, queue] = await Promise.all([
    safe('get_config', undefined, null),
    safe('get_session', undefined, null),
    safe('get_now_playing', undefined, null),
    safe('get_queue', undefined, null),
  ]);

  if (config) app.config = config;
  if (now) app.now = now;
  if (queue) app.queue = queue;
  if (session) applySession(session);

  // tema salvo; se for "livia" gravado no núcleo, conta como desbloqueado
  if (app.config.theme === 'livia') app.liviaUnlocked = true;
  if (!applyTheme(app.config.theme || 'papel', false)) applyTheme('papel', false);

  await Promise.all([
    api.on('session', applySession),
    api.on('now_playing', (p) => {
      app.now = p;
      syncEggTheme();
    }),
    api.on('queue', (p) => {
      app.queue = p;
    }),
    api.on('error', (p) => toast(p && p.message ? p.message : 'erro no núcleo')),
  ]);

  app.ready = true;
}
