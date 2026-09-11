// ponte com o núcleo. no tauri usa invoke/listen reais; no navegador cai no mock.

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { mock } from './mock.js';

export const isTauri = typeof window !== 'undefined' && !!window.__TAURI_INTERNALS__;

// invoke(nome, args) -> promessa; rejeita com string de erro em português
export const call = isTauri ? (name, args) => invoke(name, args) : (name, args) => mock.invoke(name, args);

// on(evento, cb) -> promessa de função que cancela a assinatura
export const on = isTauri
  ? (name, cb) => listen(name, (e) => cb(e.payload))
  : (name, cb) => Promise.resolve(mock.listen(name, cb));

// bytes da capa (ArrayBuffer)
export const getCover = (url) => call('get_cover', { url });

// "sempre no topo" (só faz sentido no tauri; no mock é ignorado)
let onTop = false;
export async function toggleAlwaysOnTop() {
  onTop = !onTop;
  if (!isTauri) return onTop;
  const { getCurrentWindow } = await import('@tauri-apps/api/window');
  await getCurrentWindow().setAlwaysOnTop(onTop);
  return onTop;
}
