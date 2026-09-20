<script>
  // tela de login: quadrado ditherizado, nome, botão. enter também entra.
  import { app, login, act } from '../lib/state.svelte.js';
  import { setScreenKeys } from '../lib/keyboard.js';
  import Cover from '../components/Cover.svelte';

  // o client id do app do usuário em developer.spotify.com; sem ele a busca e a
  // biblioteca esbarram na cota do id compartilhado (429)
  const idOk = $derived(/^[a-z0-9]{32}$/i.test(app.config.client_id || ''));
  function onIdKey(e) {
    if (e.key === 'Enter') {
      e.preventDefault();
      e.target.blur();
      login();
    } else if (e.key === 'Escape') {
      e.target.blur();
    }
  }
  function openDashboard() {
    act('open_dashboard');
  }

  const bday = $derived(app.session.easter_egg && app.session.birthday);
  let heart = $state(null);

  // aniversário: o coração se revela de fora para dentro em 2 s (25 quadros)
  $effect(() => {
    if (!bday) {
      heart = null;
      return;
    }
    const start = Date.now();
    heart = 0;
    const id = setInterval(() => {
      const p = Math.min(1, (Date.now() - start) / 2000);
      heart = p;
      if (p >= 1) clearInterval(id);
    }, 80);
    return () => clearInterval(id);
  });

  $effect(() =>
    setScreenKeys((e) => {
      if (e.key === 'Enter') {
        login();
        return true;
      }
      return false;
    }),
  );
</script>

<div class="login">
  <div class="center">
    <Cover url={null} size={160} frame ink={app.ink} paper={app.paper} {heart} />
    <div class="name pix">umbit</div>
    <div class="sub">cliente leve para spotify premium</div>
    <label class="field" class:ok={idOk}>
      <span class="lbl pix">client id</span>
      <input
        bind:value={app.config.client_id}
        onkeydown={onIdKey}
        placeholder="32 caracteres do seu app no spotify"
        spellcheck="false"
        autocomplete="off"
        autocapitalize="off"
        maxlength="40"
        aria-label="client id"
      />
    </label>
    <div class="help tight">
      não tem um? <button type="button" tabindex="-1" class="link" onclick={openDashboard}>abrir o painel do spotify</button> e crie um app com o redirect
      <span class="mono">http://127.0.0.1:8898/login</span>. ou use o de quem te convidou.
    </div>
    {#if app.session.connecting}
      <div class="btn ghost pix"><span class="blink-slow">abrindo o navegador…</span></div>
    {:else}
      <button type="button" tabindex="-1" class="btn pix" onclick={login}>entrar com spotify</button>
    {/if}
    {#if app.session.error}
      <div class="err">{app.session.error}</div>
    {/if}
    <div class="help">abre o navegador para autorizar (na primeira vez, duas telas: o seu app e o player). depois disso não pede mais.</div>
  </div>
  <div class="foot">
    <span>não oficial · librespot</span><span>v{app.config.version}</span>
  </div>
</div>

<style>
  .login {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .center {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex-grow: 1;
    text-align: center;
  }
  .name {
    margin-top: 36px;
    font-weight: 600;
    font-size: 30px;
    line-height: 1;
  }
  .sub {
    margin-top: 10px;
    font-size: 13px;
    opacity: 0.7;
  }
  .field {
    margin-top: 30px;
    width: 280px;
    max-width: 100%;
    height: 44px;
    box-sizing: border-box;
    border: 2px solid var(--ink);
    display: flex;
    align-items: center;
    padding: 0 12px;
    gap: 10px;
    text-align: left;
  }
  .field.ok {
    box-shadow: 4px 4px 0 var(--ink);
  }
  .lbl {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    opacity: 0.6;
    flex-shrink: 0;
  }
  .field input {
    flex: 1;
    min-width: 0;
    font-size: 13px;
    background: transparent;
    border: 0;
    color: var(--ink);
    outline: none;
  }
  .field input::placeholder {
    color: var(--ink);
    opacity: 0.35;
  }
  .help.tight {
    margin-top: 8px;
  }
  .link {
    background: transparent;
    border: 0;
    padding: 0;
    color: var(--ink);
    font: inherit;
    text-decoration: underline;
    cursor: pointer;
    opacity: 1;
  }
  .mono {
    white-space: nowrap;
  }
  .btn {
    margin-top: 24px;
    width: 280px;
    max-width: 100%;
    height: 48px;
    background: var(--ink);
    color: var(--paper);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .btn.ghost {
    background: transparent;
    color: var(--ink);
    text-transform: none;
    letter-spacing: 0;
  }
  .err {
    margin-top: 10px;
    font-size: 11px;
    max-width: 280px;
    line-height: 1.5;
  }
  .help {
    margin-top: 14px;
    font-size: 11px;
    opacity: 0.55;
    max-width: 280px;
    line-height: 1.5;
  }
  .foot {
    font-size: 11px;
    opacity: 0.5;
    display: flex;
    justify-content: space-between;
    flex-shrink: 0;
  }
</style>
