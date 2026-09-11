<script>
  // tela "tocando": capa, letreiro, barra segmentada, controles, volume
  import { untrack } from 'svelte';
  import { app, act, memo } from '../lib/state.svelte.js';
  import { mmss, currentPosition, artistsOf } from '../lib/format.js';
  import Cover from '../components/Cover.svelte';
  import Marquee from '../components/Marquee.svelte';
  import Heart from '../components/Heart.svelte';
  import Hints from '../components/Hints.svelte';

  let innerWidth = $state(window.innerWidth);
  let innerHeight = $state(window.innerHeight);
  // largura = min(largura da janela − 48, 352), com um teto pela altura para janelas baixas
  // (o resto da tela ocupa ~270 px: abas, título, barra, controles, dicas e margens)
  const coverSize = $derived(Math.max(96, Math.min(innerWidth - 48, 352, innerHeight - 272)));

  // posição interpolada, atualizada a cada 250 ms só enquanto esta tela existe
  let pos = $state(0);
  function tick() {
    pos = currentPosition(app.now);
  }
  $effect(() => {
    const id = setInterval(tick, 250);
    return () => clearInterval(id);
  });
  $effect(() => {
    tick();
  });

  const track = $derived(app.now.track);
  const dur = $derived(track ? track.duration_ms : 0);
  const BLOCKS = Array.from({ length: 20 }, (_, i) => i);
  const VOL = [6, 9, 12, 15, 18, 20];
  const filled = $derived(dur ? Math.round(Math.min(1, pos / dur) * 20) : 0);
  const volFilled = $derived(Math.round((app.now.volume / 100) * 6));

  const eggOn = $derived(app.session.easter_egg);
  // 1:43 -> o ":" vira coração por esse segundo
  const heartColon = $derived(eggOn && !!track && pos >= 103000 && pos < 104000);
  const timeParts = $derived(mmss(pos).split(':'));
  // "a nossa": tocando por causa do egg e a faixa é a sign of the times
  const ours = $derived(eggOn && app.now.egg && !!track && (!track.extra || track.extra === 'egg-ours'));

  // ---- easter egg 2: revelação do título (uma vez por faixa) ----
  const GLYPHS = '▚▞░▒▓#%';
  function scramble(target, p) {
    const n = target.length;
    const settled = Math.floor(p * n);
    let s = '';
    for (let i = 0; i < n; i++) {
      if (i < settled || target[i] === ' ') s += target[i];
      else s += GLYPHS[Math.floor(Math.random() * GLYPHS.length)];
    }
    return s;
  }
  let reveal = $state(null); // { title, artist, cover, from, mix }
  const revealKey = $derived(ours && track ? track.uri : null);
  $effect(() => {
    const uri = revealKey;
    if (!uri || memo.revealedUri === uri) return;
    memo.revealedUri = uri;
    const timers = [];
    untrack(() => {
      const t = app.now.track;
      const matilda = app.queue.upcoming[0];
      const from = matilda && matilda.cover ? matilda.cover : null;
      const finalTitle = `${t.name} · a nossa`;
      const finalArtist = `${artistsOf(t)} · ${t.album}`;
      // 4 s como se fosse a matilda
      reveal = { title: 'Matilda', artist: 'Harry Styles', cover: from, from: null, mix: 1 };
      timers.push(
        setTimeout(() => {
          // 1,2 s de dissolução: título a cada 60 ms, capa a cada 80 ms
          const start = Date.now();
          const step = () => Math.min(1, (Date.now() - start) / 1200);
          reveal = { title: scramble(finalTitle, 0), artist: scramble(finalArtist, 0), cover: t.cover, from, mix: 0 };
          const idT = setInterval(() => {
            const p = step();
            reveal.title = scramble(finalTitle, p);
            reveal.artist = scramble(finalArtist, p);
            if (p >= 1) clearInterval(idT);
          }, 60);
          const idC = setInterval(() => {
            const p = step();
            reveal.mix = p;
            if (p >= 1) {
              clearInterval(idC);
              clearInterval(idT);
              reveal = null;
            }
          }, 80);
          timers.push(idT, idC);
        }, 4000),
      );
    });
    return () => {
      for (const x of timers) {
        clearTimeout(x);
        clearInterval(x);
      }
      reveal = null;
    };
  });

  // ---- aniversário: o letreiro alterna a cada 8 s ----
  let bdayFlip = $state(false);
  $effect(() => {
    if (!(eggOn && app.session.birthday)) {
      bdayFlip = false;
      return;
    }
    const id = setInterval(() => (bdayFlip = !bdayFlip), 8000);
    return () => {
      clearInterval(id);
      bdayFlip = false;
    };
  });

  const title = $derived.by(() => {
    if (reveal) return reveal.title;
    if (bdayFlip) return 'feliz aniversário, ana lívia ♥';
    if (!track) return 'nada tocando';
    return ours ? `${track.name} · a nossa` : track.name;
  });
  const subtitle = $derived.by(() => {
    if (reveal) return reveal.artist;
    if (!track) return '/ para buscar';
    return `${artistsOf(track)} · ${track.album}`;
  });
  const coverUrl = $derived(reveal ? reveal.cover : track ? track.cover : null);

  function seekTo(i) {
    if (!dur) return;
    act('seek', { positionMs: Math.round((i / 20) * dur) });
  }
  function setVol(i) {
    act('set_volume', { volume: Math.round(((i + 1) / 6) * 100) });
  }
</script>

<svelte:window bind:innerWidth bind:innerHeight />

<div class="playing">
  <div class="art">
    <Cover url={coverUrl} from={reveal ? reveal.from : null} mix={reveal ? reveal.mix : 1} size={coverSize} frame ink={app.ink} paper={app.paper} />
  </div>

  <div class="text">
    <div class="title pix"><Marquee text={title} /></div>
    <div class="sub">{subtitle}</div>
  </div>

  <div class="progress">
    <span class="time">
      {#if app.now.loading}
        <span class="blink">carregando</span>
      {:else if heartColon}
        {timeParts[0]}<Heart />{timeParts[1]}
      {:else}
        {mmss(pos)}
      {/if}
    </span>
    <div class="bar">
      {#each BLOCKS as i (i)}
        <button type="button" tabindex="-1" class="block" class:on={i < filled} onclick={() => seekTo(i)} aria-label="ir para {Math.round((i / 20) * 100)}%"></button>
      {/each}
    </div>
    <span class="time total">{mmss(dur)}</span>
  </div>

  <div class="controls">
    <div class="buttons">
      <button type="button" tabindex="-1" class="ctl" onclick={() => act('prev_track')} aria-label="anterior">
        <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"><path d="M4 3v14" /><path d="M16 3L7 10l9 7z" /></svg>
      </button>
      <button type="button" tabindex="-1" class="ctl main" onclick={() => act('play_pause')} aria-label={app.now.playing ? 'pausar' : 'tocar'}>
        {#if app.now.playing}
          <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"><path d="M6 3v14" /><path d="M13 3v14" /></svg>
        {:else}
          <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"><path d="M5 3l11 7-11 7z" /></svg>
        {/if}
      </button>
      <button type="button" tabindex="-1" class="ctl" onclick={() => act('next_track')} aria-label="próxima">
        <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"><path d="M16 3v14" /><path d="M4 3l9 7-9 7z" /></svg>
      </button>
    </div>
    <div class="volume" title="volume {app.now.volume}">
      {#each VOL as h, i (i)}
        <button type="button" tabindex="-1" class="vol" class:on={i < volFilled} style:height="{h}px" onclick={() => setVol(i)} aria-label="volume {Math.round(((i + 1) / 6) * 100)}"></button>
      {/each}
    </div>
  </div>

  <Hints bottom items={['/ buscar', 'j k mover', 'espaço tocar', 'n p pular']} />
</div>

<style>
  .playing {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .art {
    margin-top: 22px;
    flex-shrink: 0;
  }
  .text {
    margin-top: 26px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }
  .title {
    font-weight: 600;
    font-size: 24px;
    line-height: 1.1;
    min-height: 26px;
  }
  .sub {
    font-size: 13px;
    opacity: 0.7;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .progress {
    margin-top: 20px;
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 12px;
  }
  .time {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    min-width: 30px;
  }
  .time.total {
    opacity: 0.6;
    justify-content: flex-end;
  }
  .bar {
    flex-grow: 1;
    display: flex;
    gap: 3px;
    height: 10px;
  }
  .block {
    flex: 1;
    height: 10px;
    border: 1px solid var(--ink);
  }
  .block.on {
    background: var(--ink);
  }
  .controls {
    margin-top: 18px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .buttons {
    display: flex;
    gap: 10px;
    align-items: center;
  }
  .ctl {
    width: 44px;
    height: 44px;
    border: 2px solid var(--ink);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--ink);
  }
  .ctl.main {
    width: 56px;
    border: 0;
    background: var(--ink);
    color: var(--paper);
  }
  .volume {
    display: flex;
    gap: 2px;
    align-items: flex-end;
    height: 20px;
  }
  .vol {
    width: 6px;
    border: 1px solid var(--ink);
  }
  .vol.on {
    background: var(--ink);
  }
</style>
