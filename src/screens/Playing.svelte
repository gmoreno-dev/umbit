<script>
  // tela "tocando": capa, letreiro, barra segmentada, controles, volume
  import { untrack } from 'svelte';
  import { app, act, memo, toggleShuffle } from '../lib/state.svelte.js';
  import { mmss, currentPosition, artistsOf } from '../lib/format.js';
  import Cover from '../components/Cover.svelte';
  import Marquee from '../components/Marquee.svelte';
  import Heart from '../components/Heart.svelte';
  import Hints from '../components/Hints.svelte';
  import Bilhete from '../components/Bilhete.svelte';
  import { BILHETE } from '../lib/bilhete.js';

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
  // ---- o bilhete ----
  // fase: 'egg-hers' (a dela), 'egg-ours' (a nossa) ou null
  const eggPhase = $derived(eggOn && app.now.egg && track && BILHETE.durante[track.extra] ? track.extra : null);

  // quando a nossa começa, o título se assenta letra a letra
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
  let scrambled = $state(null);
  const scrambleKey = $derived(eggPhase === 'egg-ours' && track ? track.uri : null);
  $effect(() => {
    const uri = scrambleKey;
    if (!uri || memo.revealedUri === uri) return;
    memo.revealedUri = uri;
    const name = untrack(() => app.now.track.name);
    const start = Date.now();
    scrambled = scramble(name, 0);
    const id = setInterval(() => {
      const p = Math.min(1, (Date.now() - start) / 1200);
      scrambled = scramble(name, p);
      if (p >= 1) {
        clearInterval(id);
        scrambled = null;
      }
    }, 60);
    return () => {
      clearInterval(id);
      scrambled = null;
    };
  });

  // bilhete colado na capa: aparece 2,5 s depois que a faixa começa e fica 16 s (uma vez por faixa)
  let nota = $state(null); // string[] ou null
  const notaKey = $derived(eggPhase && track ? `${eggPhase}:${track.uri}` : null);
  $effect(() => {
    const key = notaKey;
    if (!key || memo.notaShown === key) return;
    memo.notaShown = key;
    const lines = BILHETE.durante[key.split(':')[0]];
    const t1 = setTimeout(() => (nota = lines), 2500);
    const t2 = setTimeout(() => (nota = null), 2500 + 16000);
    return () => {
      clearTimeout(t1);
      clearTimeout(t2);
      nota = null;
    };
  });

  // o bilhete fechado: nos últimos segundos da nossa, fica até qualquer tecla
  let fim = $state(false);
  const fimKey = $derived(eggPhase === 'egg-ours' && dur > 0 && pos >= dur - 1500 ? track.uri : null);
  $effect(() => {
    const k = fimKey;
    if (!k || memo.fimShown === k) return;
    memo.fimShown = k;
    nota = null;
    fim = true;
  });
  function anyKey() {
    if (fim) fim = false;
  }

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
    if (scrambled) return scrambled;
    if (bdayFlip) return 'feliz aniversário, ana lívia ♥';
    if (!track) return 'nada tocando';
    return track.name;
  });
  const subtitle = $derived.by(() => {
    if (!track) return '/ para buscar';
    return `${artistsOf(track)} · ${track.album}`;
  });
  const coverUrl = $derived(track ? track.cover : null);

  function seekTo(i) {
    if (!dur) return;
    act('seek', { positionMs: Math.round((i / 20) * dur) });
  }
  function setVol(i) {
    act('set_volume', { volume: Math.round(((i + 1) / 6) * 100) });
  }
</script>

<svelte:window bind:innerWidth bind:innerHeight onkeydown={anyKey} />

<div class="playing">
  <div class="art">
    <div class="artbox" style:width="{coverSize}px" style:height="{coverSize}px">
      {#if fim}
        <div class="fim">
          <Bilhete header={BILHETE.para} lines={BILHETE.fim} fim tilt={false} />
        </div>
      {:else}
        <Cover url={coverUrl} size={coverSize} frame ink={app.ink} paper={app.paper} />
        {#if nota}
          <div class="nota">
            <Bilhete lines={nota} small />
          </div>
        {/if}
      {/if}
    </div>
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
      <button type="button" tabindex="-1" class="ctl small" class:on={app.now.shuffle} onclick={toggleShuffle} aria-label={app.now.shuffle ? 'aleatório ligado' : 'aleatório desligado'}>
        <svg width="18" height="18" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"><path d="M2 5h4l9 10h3" /><path d="M15 12l3 3-3 3" /><path d="M2 15h4l2.5-2.9" /><path d="M11.5 7.2 15 5h3" /><path d="M15 2l3 3-3 3" /></svg>
      </button>
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

  <Hints bottom items={['/ buscar', 'espaço tocar', 'n p pular', 's aleatório']} />
</div>

<style>
  .artbox {
    position: relative;
    max-width: 100%;
  }
  .nota {
    position: absolute;
    left: 10px;
    right: 16px;
    bottom: 14px;
  }
  .fim {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 6px 8px 6px 2px;
  }
  .fim :global(.bilhete) {
    width: 100%;
  }
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
  .ctl.small {
    width: 40px;
    height: 40px;
  }
  .ctl.small.on {
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
