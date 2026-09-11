// teclado global. cada tela registra o próprio tratador (tem prioridade);
// o que ele não consumir cai nos atalhos globais.

import { app, setScreen, cycleScreen, act, cycleTheme, unlockLivia, openCommand, goBackFromAbout } from './state.svelte.js';

let screenHandler = null;

// registra o tratador da tela ativa; devolve a função que o remove
export function setScreenKeys(fn) {
  screenHandler = fn;
  return () => {
    if (screenHandler === fn) screenHandler = null;
  };
}

// últimas 8 teclas digitadas fora de inputs, para a sequência "analivia"
let seq = '';
function track(key) {
  if (key.length !== 1) return;
  seq = (seq + key.toLowerCase()).slice(-8);
  if (seq === 'analivia' && !app.liviaUnlocked) unlockLivia();
}

function inInput(target) {
  if (!target) return false;
  const tag = target.tagName;
  return tag === 'INPUT' || tag === 'TEXTAREA' || target.isContentEditable === true;
}

export function onKey(e) {
  if (inInput(e.target)) return; // inputs cuidam das próprias teclas
  if (e.metaKey || e.ctrlKey || e.altKey) return;
  if (!app.ready) return;

  track(e.key);

  // login só reage ao enter
  if (!app.session.logged_in) {
    if (e.key === 'Enter' && screenHandler && screenHandler(e)) e.preventDefault();
    return;
  }

  if (screenHandler && screenHandler(e)) {
    e.preventDefault();
    return;
  }

  switch (e.key) {
    case '1':
      setScreen('playing');
      break;
    case '2':
      setScreen('library');
      break;
    case '3':
      setScreen('search');
      break;
    case '4':
      setScreen('queue');
      break;
    case 'Tab':
      cycleScreen(e.shiftKey ? -1 : 1);
      break;
    case '/':
      setScreen('search');
      app.search.wantFocus = true;
      break;
    case ':':
      openCommand();
      break;
    case ' ':
      act('play_pause');
      break;
    case 'n':
      act('next_track');
      break;
    case 'p':
      act('prev_track');
      break;
    case 'ArrowLeft':
      act('seek_relative', { deltaMs: e.shiftKey ? -30000 : -5000 });
      break;
    case 'ArrowRight':
      act('seek_relative', { deltaMs: e.shiftKey ? 30000 : 5000 });
      break;
    case '-':
    case '_':
      act('set_volume', { volume: Math.max(0, app.now.volume - 5) });
      break;
    case '=':
    case '+':
      act('set_volume', { volume: Math.min(100, app.now.volume + 5) });
      break;
    case 't':
      cycleTheme();
      break;
    case 'Escape':
      if (app.screen === 'about') goBackFromAbout();
      else return;
      break;
    default:
      return; // não é nosso: deixa passar
  }
  e.preventDefault();
}

// movimentação padrão de cursor em listas: devolve o novo índice ou null se a tecla não é de cursor
export function moveCursor(e, cursor, length) {
  if (length <= 0) return null;
  switch (e.key) {
    case 'j':
    case 'ArrowDown':
      return Math.min(length - 1, cursor + 1);
    case 'k':
    case 'ArrowUp':
      return Math.max(0, cursor - 1);
    case 'g':
      return 0;
    case 'G':
      return length - 1;
    default:
      return null;
  }
}

export const isBack = (e) => e.key === 'b' || e.key === 'Escape' || e.key === 'Backspace';
