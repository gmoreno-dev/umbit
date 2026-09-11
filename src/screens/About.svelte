<script>
  // sobre (:sobre): versão, quem está logado, atalhos, dedicatória e assinatura
  import { app, goBackFromAbout } from '../lib/state.svelte.js';
  import { setScreenKeys, isBack } from '../lib/keyboard.js';
  import Cover from '../components/Cover.svelte';
  import Signature from '../components/Signature.svelte';
  import Hints from '../components/Hints.svelte';

  // tabela curta: o resto está nas dicas de cada tela
  const KEYS = [
    ['1 2 3 4', 'telas'],
    ['/', 'buscar'],
    [':', 'comando'],
    ['espaço', 'tocar / pausar'],
    ['n p', 'pular'],
    ['← →', '−5 s / +5 s'],
    ['- = t', 'volume · tema'],
  ];

  $effect(() =>
    setScreenKeys((e) => {
      if (isBack(e)) {
        goBackFromAbout();
        return true;
      }
      return false;
    }),
  );
</script>

<div class="about">
  <div class="body scroll">
    <div class="top">
      <Cover url={null} size={120} frame ink={app.ink} paper={app.paper} />
    </div>
    <div class="v pix">umbit v{app.config.version}</div>
    <div class="desc">cliente leve para spotify premium · não oficial · feito com librespot</div>
    <div class="user">logado como {app.session.username || app.session.display_name || '—'}</div>

    <table class="keys">
      <tbody>
        {#each KEYS as [k, d] (k)}
          <tr><td class="k">{k}</td><td>{d}</td></tr>
        {/each}
      </tbody>
    </table>

    <div class="dedic pix">feito para ana lívia</div>
    <div class="sig"><Signature width={120} /></div>
  </div>
  <Hints items={['esc voltar']} />
</div>

<style>
  .about {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .body {
    margin-top: 22px;
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }
  .top {
    margin-top: 4px;
    margin-bottom: 26px;
    flex-shrink: 0;
  }
  .v {
    font-weight: 600;
    font-size: 22px;
    line-height: 1;
  }
  .desc {
    margin-top: 10px;
    font-size: 12px;
    opacity: 0.7;
    max-width: 300px;
    line-height: 1.5;
  }
  .user {
    margin-top: 6px;
    font-size: 12px;
    opacity: 0.7;
  }
  .keys {
    margin-top: 18px;
    border-collapse: collapse;
    font-size: 12px;
    text-align: left;
    flex-shrink: 0;
  }
  .keys td {
    padding: 2px 10px 2px 0;
    line-height: 1.4;
    vertical-align: top;
  }
  .keys .k {
    font-weight: 700;
    white-space: nowrap;
  }
  .dedic {
    margin-top: 22px;
    font-size: 14px;
    letter-spacing: 0.04em;
  }
  .sig {
    margin-top: 6px;
    margin-bottom: 12px;
    color: var(--ink);
    flex-shrink: 0;
  }
</style>
