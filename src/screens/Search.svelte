<script>
  // busca: campo com cursor de bloco desenhado, três grupos de resultados, álbum abre na própria tela
  import { untrack } from 'svelte';
  import { app, act } from '../lib/state.svelte.js';
  import { setScreenKeys, moveCursor, isBack } from '../lib/keyboard.js';
  import { trackLine, mmss, normalize } from '../lib/format.js';
  import Row from '../components/Row.svelte';
  import Heart from '../components/Heart.svelte';
  import Hints from '../components/Hints.svelte';
  import Bilhete from '../components/Bilhete.svelte';
  import { BILHETE } from '../lib/bilhete.js';

  const st = $derived(app.search);
  let input = $state(null);
  let measure = $state(null);
  let wrap = $state(null);
  let listEl = $state(null);
  let focused = $state(false);
  let caretLeft = $state(0);
  let timer = null;

  const eggOn = $derived(app.session.easter_egg);
  const heartCursor = $derived(eggOn && normalize(st.query) === 'analivia');
  // a busca por ela vira um bilhete (a primeira linha dos resultados é a matilda marcada)
  const bilhete = $derived(!st.open && st.results && st.results.tracks[0] && st.results.tracks[0].extra === 'egg');
  const abertura = $derived(app.session.birthday ? BILHETE.aberturaAniversario : BILHETE.abertura);

  // grupos com índice global de linha (o cursor percorre todas)
  const groups = $derived.by(() => {
    if (st.open) return [{ id: 'atracks', label: null, rows: st.open.items.map((item, k) => ({ idx: k, kind: 'atrack', item })) }];
    const r = st.results;
    if (!r) return [];
    let idx = 0;
    const g = [];
    const push = (id, label, kind, items) => {
      if (!items.length) return;
      g.push({ id, label, rows: items.map((item) => ({ idx: idx++, kind, item })) });
    };
    push('tracks', 'faixas', 'track', r.tracks);
    push('albums', 'álbuns', 'album', r.albums);
    push('artists', 'artistas', 'artist', r.artists);
    return g;
  });
  const rows = $derived(groups.flatMap((g) => g.rows));

  // ---- busca com debounce de 350 ms ----
  async function runSearch(q) {
    const query = q.trim();
    if (!query) {
      app.search.results = null;
      app.search.loading = false;
      return;
    }
    app.search.loading = true;
    const r = await act('search', { query });
    if (app.search.query.trim() !== query) return; // resposta velha
    app.search.loading = false;
    if (r) {
      app.search.results = r;
      app.search.cursor = focused ? -1 : 0;
    }
  }
  function onInput() {
    clearTimeout(timer);
    app.search.open = null;
    app.search.cursor = -1;
    timer = setTimeout(() => runSearch(app.search.query), 350);
  }
  function clear() {
    clearTimeout(timer);
    app.search.query = '';
    app.search.results = null;
    app.search.open = null;
    app.search.cursor = -1;
    app.search.loading = false;
  }

  // ---- ações ----
  async function openAlbum(a) {
    app.search.openFromCursor = app.search.cursor;
    app.search.open = { uri: a.uri, name: a.name, items: [], loading: true };
    app.search.cursor = 0;
    const page = await act('get_album_tracks', { uri: a.uri });
    if (!app.search.open || app.search.open.uri !== a.uri) return;
    if (page) {
      app.search.open.items = page.items;
      app.search.open.loading = false;
    } else app.search.open = null;
  }
  function back() {
    if (!app.search.open) return false;
    app.search.open = null;
    app.search.cursor = app.search.openFromCursor || 0;
    return true;
  }
  function activate(i) {
    const row = rows[i];
    if (!row) return;
    app.search.cursor = i;
    const { kind, item } = row;
    if (kind === 'track') {
      if (item.extra === 'egg') act('play_egg');
      else act('play_tracks', { tracks: [item], name: 'busca' });
    } else if (kind === 'album') {
      openAlbum(item);
    } else if (kind === 'artist') {
      clearTimeout(timer);
      app.search.query = `artist:"${item.name}"`;
      app.search.cursor = -1;
      runSearch(app.search.query);
    } else if (kind === 'atrack') {
      const o = app.search.open;
      act('play_context', { uri: o.uri, trackUri: item.uri, index: i, name: o.name, tracks: o.items });
    }
  }
  function focusInput() {
    app.search.cursor = -1;
    if (input) input.focus();
  }

  // teclas dentro do campo: enter, esc, ↑/↓ (j/k são texto aqui)
  function onInputKey(e) {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (rows.length) activate(st.cursor >= 0 ? st.cursor : 0);
      else {
        clearTimeout(timer);
        runSearch(app.search.query);
      }
    } else if (e.key === 'Escape') {
      e.preventDefault();
      if (st.query) clear();
      input.blur();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (rows.length) {
        input.blur();
        app.search.cursor = Math.min(rows.length - 1, Math.max(0, st.cursor + 1));
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
    }
  }

  // teclas com o foco fora do campo
  $effect(() =>
    setScreenKeys((e) => {
      const c = st.cursor;
      if ((e.key === 'k' || e.key === 'ArrowUp') && c <= 0) {
        focusInput();
        return true;
      }
      const moved = moveCursor(e, Math.max(0, c), rows.length);
      if (moved !== null) {
        app.search.cursor = moved;
        return true;
      }
      if (e.key === 'Enter') {
        if (c >= 0) activate(c);
        else focusInput();
        return true;
      }
      if (e.key === 'Escape') {
        if (st.open) return back();
        clear();
        focusInput();
        return true;
      }
      if (isBack(e)) return back();
      return false;
    }),
  );

  // foco pedido pelo "/" global; ao entrar na tela sem resultados, também foca
  $effect(() => {
    const want = st.wantFocus;
    if (!input) return;
    untrack(() => {
      if (want || !st.results) {
        app.search.wantFocus = false;
        focusInput();
      }
    });
  });

  // cursor de bloco: logo depois do texto (medido com a mesma fonte)
  $effect(() => {
    void st.query;
    if (!measure || !wrap) return;
    caretLeft = Math.min(measure.offsetWidth, Math.max(0, wrap.clientWidth - 9));
  });

  // cursor da lista sempre visível
  $effect(() => {
    const c = st.cursor;
    if (c < 0 || !listEl) return;
    const el = listEl.querySelector('.active');
    if (el) el.scrollIntoView({ block: 'nearest' });
  });

  const hints = $derived(st.open ? ['j k mover', 'enter tocar', 'b voltar'] : ['j k mover', 'enter tocar', 'esc limpar']);
</script>

<div class="search">
  <div class="field" class:focused>
    <span class="slash">/</span>
    <span class="wrap" bind:this={wrap}>
      <input
        bind:this={input}
        bind:value={app.search.query}
        oninput={onInput}
        onkeydown={onInputKey}
        onfocus={() => (focused = true)}
        onblur={() => (focused = false)}
        spellcheck="false"
        autocomplete="off"
        autocapitalize="off"
        aria-label="buscar"
      />
      <span class="measure" bind:this={measure} aria-hidden="true">{st.query}</span>
      <span class="caret" class:blink={focused} class:heart={heartCursor} style:left="{caretLeft}px">
        {#if heartCursor}<Heart />{/if}
      </span>
    </span>
  </div>

  {#if st.open}
    <div class="head pix">
      <button type="button" tabindex="-1" class="backbtn" onclick={back}>← {st.open.name}</button>
    </div>
  {/if}

  <div class="results scroll" bind:this={listEl}>
    {#if bilhete}
      <div class="bilhete-wrap" class:active={st.cursor === 0}>
        <Bilhete header={BILHETE.para} lines={abertura} tracks={BILHETE.faixas} active={st.cursor === 0} onclick={() => activate(0)} />
      </div>
    {/if}
    {#each groups as g (g.id)}
      <div class="group">
        {#if g.label && !(bilhete && g.id === 'tracks' && g.rows.length === 1)}<div class="label pix">{g.label}</div>{/if}
        {#each g.rows as r (r.kind + r.item.uri + r.idx)}
          {#if r.kind === 'track' && r.item.extra === 'egg'}
            <!-- a matilda é o bilhete acima -->
          {:else if r.kind === 'track'}
            <Row text={trackLine(r.item)} meta={mmss(r.item.duration_ms)} active={st.cursor === r.idx} onclick={() => activate(r.idx)} />
          {:else if r.kind === 'album'}
            <Row text={r.item.name} meta={r.item.year || ''} thumb={r.item.cover} active={st.cursor === r.idx} onclick={() => activate(r.idx)} />
          {:else if r.kind === 'artist'}
            <Row text={r.item.name} meta="artista" active={st.cursor === r.idx} onclick={() => activate(r.idx)} />
          {:else}
            <Row text={trackLine(r.item)} meta={mmss(r.item.duration_ms)} active={st.cursor === r.idx} onclick={() => activate(r.idx)} />
          {/if}
        {/each}
      </div>
    {/each}
    {#if st.loading || (st.open && st.open.loading)}
      <div class="empty">buscando…</div>
    {:else if st.results && !rows.length && !st.open}
      <div class="empty">nada encontrado</div>
    {/if}
  </div>

  <Hints items={hints} />
</div>

<style>
  .bilhete-wrap {
    padding: 10px 8px 18px 4px;
  }
  .bilhete-wrap.active {
    scroll-margin: 8px;
  }
  .search {
    flex: 0 1 auto;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .field {
    margin-top: 22px;
    height: 44px;
    border: 2px solid var(--ink);
    display: flex;
    align-items: center;
    padding: 0 12px;
    font-size: 15px;
    flex-shrink: 0;
  }
  .slash {
    opacity: 0.5;
    margin-right: 8px;
    flex-shrink: 0;
  }
  .wrap {
    position: relative;
    flex: 1;
    min-width: 0;
    height: 100%;
    display: flex;
    align-items: center;
  }
  input {
    width: 100%;
    font-size: 15px;
    caret-color: transparent;
    line-height: 1;
  }
  .measure {
    position: absolute;
    left: 0;
    top: 0;
    visibility: hidden;
    white-space: pre;
    font-size: 15px;
    pointer-events: none;
  }
  .caret {
    position: absolute;
    top: 50%;
    margin-top: -9px;
    width: 9px;
    height: 18px;
    background: var(--ink);
    opacity: 0.35;
    pointer-events: none;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .caret.blink {
    opacity: 1;
  }
  .caret.heart {
    background: transparent;
    color: var(--ink);
    width: 12px;
  }
  .head {
    margin-top: 16px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    flex-shrink: 0;
    overflow: hidden;
  }
  .backbtn {
    border: 2px solid var(--ink);
    padding: 4px 8px;
    max-width: 100%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.2;
  }
  .results {
    margin-top: 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    flex: 0 1 auto;
    min-height: 0;
  }
  .group {
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }
  .label {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    opacity: 0.6;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--ink);
  }
  .empty {
    font-size: 12px;
    opacity: 0.6;
    padding: 0 10px;
    height: 44px;
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }
</style>
