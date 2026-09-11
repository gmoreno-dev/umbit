<script>
  // um bilhete de papel: borda de tinta, sombra dura, levemente torto, escrito à máquina.
  let { lines = [], header = null, tracks = null, fim = false, small = false, active = false, tilt = true, speed = 38, onclick = null } = $props();

  let shown = $state(0);
  const total = $derived(lines.reduce((a, l) => a + l.length + 1, 0));
  const done = $derived(shown >= total);

  // digita letra por letra; recomeça quando as linhas mudam
  $effect(() => {
    void lines;
    shown = 0;
    const id = setInterval(() => {
      shown += 1;
      if (shown >= total) clearInterval(id);
    }, speed);
    return () => clearInterval(id);
  });

  const visible = $derived.by(() => {
    let left = shown;
    return lines.map((l) => {
      const s = l.slice(0, Math.max(0, Math.min(l.length, left)));
      left -= l.length + 1;
      return s;
    });
  });
  // em qual linha está o cursor
  const cursorAt = $derived.by(() => {
    if (done) return -1;
    let left = shown;
    for (let i = 0; i < lines.length; i++) {
      if (left <= lines[i].length) return i;
      left -= lines[i].length + 1;
    }
    return -1;
  });
</script>

<div class="bilhete" class:small class:active class:tilt class:clickable={!!onclick} role={onclick ? 'button' : undefined} tabindex="-1" onclick={onclick}>
  {#if header}<div class="para pix">{header}</div>{/if}
  {#each visible as v, i (i)}
    <div class="linha">{v}{#if cursorAt === i}<span class="caret"></span>{/if}</div>
  {/each}
  {#if tracks && done}
    <div class="faixas">
      {#each tracks as t (t.n)}
        <div class="faixa"><span class="n">{t.n}</span><span class="nome">{t.nome}</span><span class="nota">· {t.nota}</span></div>
      {/each}
    </div>
  {/if}
  {#if fim && done}
    <div class="coracao">
      <svg width="22" height="20" viewBox="0 0 11 10" fill="currentColor" shape-rendering="crispEdges"><path d="M2 1h2v1h1v1h1V2h1V1h2v1h1v3H9v1H8v1H7v1H6v1H5V8H4V7H3V6H2V5H1V2h1z" /></svg>
    </div>
  {/if}
</div>

<style>
  .bilhete {
    background: var(--paper);
    color: var(--ink);
    border: 2px solid var(--ink);
    box-shadow: 6px 6px 0 var(--ink);
    padding: 14px 16px 16px 16px;
    font-family: 'Space Mono', 'Courier New', monospace;
    font-size: 13px;
    line-height: 1.6;
    max-width: 100%;
    box-sizing: border-box;
  }
  .tilt {
    transform: rotate(-1.5deg);
  }
  .small {
    font-size: 11px;
    line-height: 1.5;
    padding: 8px 10px 9px 10px;
    box-shadow: 4px 4px 0 var(--ink);
  }
  .active {
    outline: 2px solid var(--ink);
    outline-offset: 3px;
  }
  .clickable {
    cursor: pointer;
  }
  .para {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    opacity: 0.6;
    margin-bottom: 8px;
  }
  .linha {
    min-height: 1.6em;
    white-space: pre-wrap;
  }
  .caret {
    display: inline-block;
    width: 8px;
    height: 0.9em;
    background: var(--ink);
    vertical-align: -0.1em;
    margin-left: 1px;
    animation: blink 1s steps(1) infinite;
  }
  @keyframes blink {
    50% {
      opacity: 0;
    }
  }
  .faixas {
    margin-top: 10px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .faixa {
    display: block;
    text-indent: -2.6em;
    padding-left: 2.6em;
  }
  .faixa .n {
    display: inline-block;
    width: 2.6em;
    text-indent: 0;
  }
  .faixa .nome {
    margin-right: 0.4em;
  }
  .n {
    opacity: 0.5;
  }
  .nota {
    opacity: 0.7;
  }
  .coracao {
    margin-top: 12px;
    display: flex;
    justify-content: flex-end;
  }
</style>
