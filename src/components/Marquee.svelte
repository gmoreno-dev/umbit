<script module>
  // folha de estilo compartilhada por todos os letreiros: um @keyframes por instância
  let sheet = null;
  let seq = 0;
  function getSheet() {
    if (!sheet) {
      const el = document.createElement('style');
      document.head.appendChild(el);
      sheet = el.sheet;
    }
    return sheet;
  }
  function removeRule(name) {
    if (!sheet) return;
    const rules = sheet.cssRules;
    for (let i = rules.length - 1; i >= 0; i--) if (rules[i].name === name) sheet.deleteRule(i);
  }
</script>

<script>
  // letreiro de lcd: se o texto cabe, fica parado; se não, rola em `transform`
  // com pausa de 2 s nas pontas. os keyframes são gerados em css para que a
  // pausa e a velocidade (40 px/s) sejam exatas para qualquer distância.
  let { text = '' } = $props();

  let outer = $state(null);
  let inner = $state(null);
  let bump = $state(0); // força nova medição (resize, fontes)
  let anim = $state(null); // { name, dur }

  const SPEED = 40; // px por segundo
  const PAUSE = 2; // segundos em cada ponta
  const name = `umbit-mq-${++seq}`;

  function setRule(dist, p1, p2) {
    const s = getSheet();
    removeRule(name);
    s.insertRule(`@keyframes ${name} { 0%, ${p1}% { transform: translateX(0) } ${p2}%, 100% { transform: translateX(-${dist}px) } }`, s.cssRules.length);
  }

  $effect(() => {
    void text;
    void bump;
    if (!outer || !inner) return;
    const dist = inner.scrollWidth - outer.clientWidth;
    if (dist <= 2) {
      anim = null;
      return;
    }
    const dur = PAUSE * 2 + dist / SPEED;
    const p1 = ((PAUSE / dur) * 100).toFixed(2);
    const p2 = (100 - (PAUSE / dur) * 100).toFixed(2);
    setRule(dist, p1, p2);
    anim = { name, dur };
  });

  // fontes carregam depois do primeiro desenho: mede de novo
  $effect(() => {
    let alive = true;
    if (document.fonts && document.fonts.ready) document.fonts.ready.then(() => alive && bump++);
    return () => {
      alive = false;
      removeRule(name);
    };
  });
</script>

<svelte:window onresize={() => bump++} />

<div class="mq" bind:this={outer}>
  <span class="inner" bind:this={inner} style:animation={anim ? `${anim.name} ${anim.dur}s linear infinite alternate` : 'none'}>{text}</span>
</div>

<style>
  .mq {
    overflow: hidden;
    white-space: nowrap;
    width: 100%;
  }
  .inner {
    display: inline-block;
    white-space: nowrap;
    will-change: transform;
  }
</style>
