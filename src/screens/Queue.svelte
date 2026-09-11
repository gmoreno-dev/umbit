<script>
  // fila: faixa atual invertida com ícone, depois as próximas numeradas
  import { app, act } from '../lib/state.svelte.js';
  import { setScreenKeys, moveCursor } from '../lib/keyboard.js';
  import { trackLine, mmss } from '../lib/format.js';
  import Row from '../components/Row.svelte';
  import Hints from '../components/Hints.svelte';

  const q = $derived(app.queue);
  const n = $derived((q.current ? 1 : 0) + q.upcoming.length);
  const minutes = $derived(Math.round(((q.current ? q.current.duration_ms : 0) + q.upcoming.reduce((s, t) => s + (t.duration_ms || 0), 0)) / 60000));
  let listEl = $state(null);

  const pad = (i) => String(i).padStart(2, '0');

  function playFrom(i) {
    app.queueCursor = i;
    const t = q.upcoming[i];
    if (!t) return;
    if (q.context_uri) {
      act('play_context', { uri: q.context_uri, index: q.current_index + 1 + i, trackUri: t.uri, name: q.context_name });
    } else {
      act('play_tracks', { tracks: q.upcoming.slice(i), name: q.context_name });
    }
  }

  $effect(() =>
    setScreenKeys((e) => {
      const moved = moveCursor(e, app.queueCursor, q.upcoming.length);
      if (moved !== null) {
        app.queueCursor = moved;
        return true;
      }
      if (e.key === 'Enter') {
        playFrom(app.queueCursor);
        return true;
      }
      return false;
    }),
  );

  // cursor dentro dos limites e visível
  $effect(() => {
    const c = app.queueCursor;
    const len = q.upcoming.length;
    if (c >= len && len) app.queueCursor = len - 1;
    if (!listEl) return;
    const el = listEl.children[(q.current ? 1 : 0) + c];
    if (el) el.scrollIntoView({ block: 'nearest' });
  });
</script>

<div class="queue">
  <div class="head">
    <span class="ctx">de: {q.context_name || '—'}</span>
    <span>{n} {n === 1 ? 'faixa' : 'faixas'} · {minutes} min</span>
  </div>

  <div class="list scroll" bind:this={listEl}>
    {#if q.current}
      <Row playing active text={trackLine(q.current)} meta={mmss(q.current.duration_ms)} heart={q.current.extra === 'egg-next'} onclick={() => act('play_pause')} />
    {/if}
    {#each q.upcoming as t, i (t.uri + ':' + i)}
      <Row num={pad(i + 1)} text={trackLine(t)} meta={mmss(t.duration_ms)} heart={t.extra === 'egg-next'} active={app.queueCursor === i} onclick={() => playFrom(i)} />
    {/each}
    {#if !n}
      <div class="empty">fila vazia</div>
    {/if}
  </div>

  {#if q.note}
    <div class="note">{q.note}</div>
  {/if}

  <Hints items={['j k mover', 'enter tocar daqui']} />
</div>

<style>
  .queue {
    flex: 0 1 auto;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .head {
    margin-top: 22px;
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 12px;
    font-size: 12px;
    opacity: 0.6;
    flex-shrink: 0;
    white-space: nowrap;
  }
  .ctx {
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }
  .list {
    margin-top: 10px;
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
  .note {
    margin-top: 8px;
    font-size: 11px;
    opacity: 0.7;
    flex-shrink: 0;
  }
</style>
