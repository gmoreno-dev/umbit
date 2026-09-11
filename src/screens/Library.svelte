<script>
  // biblioteca: playlists / curtidas / álbuns, com abertura de faixas na mesma tela
  import { untrack } from 'svelte';
  import { app, act } from '../lib/state.svelte.js';
  import { setScreenKeys, moveCursor, isBack } from '../lib/keyboard.js';
  import { trackLine, mmss, count } from '../lib/format.js';
  import Row from '../components/Row.svelte';
  import Hints from '../components/Hints.svelte';

  const SECTIONS = [
    ['playlists', 'playlists'],
    ['liked', 'curtidas'],
    ['albums', 'álbuns'],
  ];
  const CMD = { playlists: 'get_playlists', liked: 'get_liked', albums: 'get_saved_albums' };

  const lib = $derived(app.library);
  let listEl = $state(null);

  const bday = $derived(app.session.easter_egg && app.session.birthday);
  const eggRow = $derived(bday && lib.section === 'playlists' && !lib.open && lib.eggTracks ? lib.eggTracks[0] : null);

  // linhas visíveis
  const rows = $derived.by(() => {
    if (lib.open) return lib.open.items;
    const items = lib[lib.section].items;
    return eggRow ? [eggRow, ...items] : items;
  });
  const loading = $derived(lib.open ? lib.open.loading : lib[lib.section].loading);

  // ---- carregamento ----
  async function ensureLoaded(section) {
    const l = app.library[section];
    if (l.loaded || l.loading) return;
    l.loading = true;
    const page = await act(CMD[section], { offset: 0 });
    l.loading = false;
    if (page) {
      l.items = page.items;
      l.total = page.total;
      l.loaded = true;
    }
  }
  async function loadMore(section) {
    const l = app.library[section];
    if (!l.loaded || l.loading || l.items.length >= l.total) return;
    l.loading = true;
    const page = await act(CMD[section], { offset: l.items.length });
    l.loading = false;
    if (page) l.items.push(...page.items);
  }
  async function loadMoreOpen() {
    const o = app.library.open;
    if (!o || o.kind !== 'playlist' || o.loading || o.items.length >= o.total) return;
    o.loading = true;
    const page = await act('get_playlist_tracks', { uri: o.uri, offset: o.items.length });
    if (app.library.open !== o && app.library.open?.uri !== o.uri) return;
    o.loading = false;
    if (page) o.items.push(...page.items);
  }

  $effect(() => {
    const s = lib.section;
    untrack(() => ensureLoaded(s));
  });
  $effect(() => {
    if (bday && !lib.eggTracks) act('get_egg_tracks').then((t) => t && (app.library.eggTracks = t));
  });

  // cursor sempre visível + paginação a 10 do fim
  $effect(() => {
    const c = lib.cursor;
    const n = rows.length;
    if (listEl) {
      const el = listEl.children[c];
      if (el) el.scrollIntoView({ block: 'nearest' });
    }
    untrack(() => {
      if (n && c >= n - 10) {
        if (app.library.open) loadMoreOpen();
        else loadMore(app.library.section);
      }
    });
  });

  // ---- ações ----
  async function openList(kind, uri, name) {
    const l = app.library;
    l.openFromCursor = l.cursor;
    l.open = { kind, uri, name, items: [], total: 0, loading: true };
    l.cursor = 0;
    const page = await act(kind === 'playlist' ? 'get_playlist_tracks' : 'get_album_tracks', kind === 'playlist' ? { uri, offset: 0 } : { uri });
    if (!l.open || l.open.uri !== uri) return;
    l.open.loading = false;
    if (page) {
      l.open.items = page.items;
      l.open.total = page.total;
    } else {
      l.open = null;
    }
  }
  function back() {
    const l = app.library;
    if (!l.open) return false;
    l.open = null;
    l.cursor = l.openFromCursor || 0;
    return true;
  }
  async function playAt(i) {
    const l = app.library;
    if (l.open) {
      const t = l.open.items[i];
      if (!t) return;
      act('play_context', { uri: l.open.uri, trackUri: t.uri, index: i, name: l.open.name, tracks: l.open.items });
      return;
    }
    if (l.section === 'liked') {
      const t = l.liked.items[i];
      if (!t) return;
      if (!l.likedUri) l.likedUri = (await act('get_liked_uri')) || null;
      act('play_context', { uri: l.likedUri, trackUri: t.uri, index: i, name: 'curtidas', tracks: l.liked.items });
    }
  }
  function activate(i) {
    const l = app.library;
    l.cursor = i;
    if (l.open) return playAt(i);
    if (l.section === 'playlists') {
      if (eggRow && i === 0) return act('play_egg');
      const p = l.playlists.items[eggRow ? i - 1 : i];
      if (p) openList('playlist', p.uri, p.name);
    } else if (l.section === 'liked') {
      playAt(i);
    } else {
      const a = l.albums.items[i];
      if (a) openList('album', a.uri, a.name);
    }
  }
  function setSection(dir) {
    const l = app.library;
    const i = SECTIONS.findIndex(([id]) => id === l.section);
    l.section = SECTIONS[(i + dir + SECTIONS.length) % SECTIONS.length][0];
    l.open = null;
    l.cursor = 0;
  }

  $effect(() =>
    setScreenKeys((e) => {
      const l = app.library;
      const moved = moveCursor(e, l.cursor, rows.length);
      if (moved !== null) {
        l.cursor = moved;
        return true;
      }
      if (e.key === 'Enter') {
        activate(l.cursor);
        return true;
      }
      if (e.key === 'Tab' || e.key === 'l') {
        setSection(e.shiftKey ? -1 : 1);
        return true;
      }
      if (e.key === 'h') {
        setSection(-1);
        return true;
      }
      if (isBack(e)) return back();
      return false;
    }),
  );

  const hints = $derived(lib.open ? ['j k mover', 'enter tocar', 'b voltar'] : lib.section === 'liked' ? ['j k mover', 'enter tocar', 'tab seção'] : ['j k mover', 'enter abrir', 'tab seção']);
</script>

<div class="library">
  {#if lib.open}
    <div class="filters pix">
      <button type="button" tabindex="-1" class="filter on" onclick={back}>← {lib.open.name}</button>
    </div>
  {:else}
    <div class="filters pix">
      {#each SECTIONS as [id, label] (id)}
        <button
          type="button"
          tabindex="-1"
          class="filter"
          class:on={lib.section === id}
          onclick={() => {
            app.library.section = id;
            app.library.open = null;
            app.library.cursor = 0;
          }}>{label}</button
        >
      {/each}
    </div>
  {/if}

  <div class="list scroll" bind:this={listEl}>
    {#if lib.open}
      {#each rows as t, i (t.uri + ':' + i)}
        <Row text={trackLine(t)} meta={mmss(t.duration_ms)} active={lib.cursor === i} onclick={() => activate(i)} />
      {/each}
    {:else if lib.section === 'playlists'}
      {#each rows as p, i (p.uri + ':' + i)}
        {#if eggRow && i === 0}
          <Row text={trackLine(p)} heart active={lib.cursor === i} onclick={() => activate(i)} />
        {:else}
          <Row text={p.name} meta={count(p.track_count)} active={lib.cursor === i} onclick={() => activate(i)} />
        {/if}
      {/each}
    {:else if lib.section === 'liked'}
      {#each rows as t, i (t.uri + ':' + i)}
        <Row text={trackLine(t)} meta={mmss(t.duration_ms)} active={lib.cursor === i} onclick={() => activate(i)} />
      {/each}
    {:else}
      {#each rows as a, i (a.uri)}
        <Row text={a.name} meta={a.year || ''} thumb={a.cover} active={lib.cursor === i} onclick={() => activate(i)} />
      {/each}
    {/if}
    {#if loading && !rows.length}
      <div class="empty">carregando…</div>
    {:else if !rows.length}
      <div class="empty">nada aqui</div>
    {/if}
  </div>

  <Hints items={hints} />
</div>

<style>
  .library {
    flex: 0 1 auto;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .filters {
    margin-top: 22px;
    display: flex;
    gap: 8px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    flex-shrink: 0;
    overflow: hidden;
  }
  .filter {
    border: 2px solid transparent;
    padding: 4px 8px;
    opacity: 0.5;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.2;
  }
  .filter.on {
    border-color: var(--ink);
    opacity: 1;
  }
  .list {
    margin-top: 14px;
    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--ink);
    flex: 0 1 auto;
    min-height: 44px;
  }
  .empty {
    height: 44px;
    display: flex;
    align-items: center;
    padding: 0 10px;
    font-size: 12px;
    opacity: 0.6;
  }
</style>
