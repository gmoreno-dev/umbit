<script>
  // minibarra de reprodução (biblioteca, busca, fila): clicar leva para "tocando"
  import { app, setScreen } from '../lib/state.svelte.js';
  import { mmss, currentPosition, trackLine } from '../lib/format.js';

  const SEGS = Array.from({ length: 24 }, (_, i) => i);
  let pos = $state(0);

  function tick() {
    pos = currentPosition(app.now);
  }
  // 250 ms só enquanto a minibarra está montada
  $effect(() => {
    const id = setInterval(tick, 250);
    return () => clearInterval(id);
  });
  // e imediatamente quando o núcleo manda um estado novo
  $effect(() => {
    tick();
  });

  const dur = $derived(app.now.track ? app.now.track.duration_ms : 0);
  const filled = $derived(dur ? Math.round(Math.min(1, pos / dur) * 24) : 0);
</script>

<button type="button" tabindex="-1" class="mini" onclick={() => setScreen('playing')} aria-label="ir para tocando">
  <div class="line">
    <svg width="12" height="12" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
      {#if app.now.playing}
        <path d="M5 3h4v14H5zM11 3h4v14h-4z" />
      {:else}
        <path d="M4 3l13 7-13 7z" />
      {/if}
    </svg>
    <span class="t">{trackLine(app.now.track) || 'nada tocando'}</span>
    <span class="time">{app.now.loading ? '…' : mmss(pos)}</span>
  </div>
  <div class="segs">
    {#each SEGS as i (i)}
      <div class="seg" class:off={i >= filled}></div>
    {/each}
  </div>
</button>

<style>
  .mini {
    margin-top: auto;
    border-top: 2px solid var(--ink);
    padding-top: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
    flex-shrink: 0;
  }
  .line {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 12px;
  }
  .t {
    flex-grow: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }
  .time {
    opacity: 0.6;
    flex-shrink: 0;
  }
  .segs {
    display: flex;
    gap: 3px;
    height: 4px;
  }
  .seg {
    flex: 1;
    background: var(--ink);
  }
  .seg.off {
    opacity: 0.25;
  }
</style>
