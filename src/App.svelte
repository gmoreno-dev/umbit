<script>
  // raiz: escolhe a tela, abas no topo, minibarra nas telas de lista, toast por cima
  import { onMount } from 'svelte';
  import { app, init } from './lib/state.svelte.js';
  import { onKey } from './lib/keyboard.js';
  import Tabs from './components/Tabs.svelte';
  import MiniBar from './components/MiniBar.svelte';
  import Toast from './components/Toast.svelte';
  import Login from './screens/Login.svelte';
  import Playing from './screens/Playing.svelte';
  import Library from './screens/Library.svelte';
  import Search from './screens/Search.svelte';
  import Queue from './screens/Queue.svelte';
  import About from './screens/About.svelte';

  onMount(() => {
    init();
  });

  const withMini = $derived(app.screen === 'library' || app.screen === 'search' || app.screen === 'queue');

  // clicar num botão não deve roubar o foco (o teclado é global)
  function noFocus(e) {
    const t = e.target;
    if (t && t.closest && t.closest('button')) e.preventDefault();
  }
</script>

<svelte:window onkeydown={onKey} onmousedown={noFocus} />

<div class="app">
  {#if !app.ready}
    <div class="boot pix">umbit</div>
  {:else if app.screen === 'login'}
    <Login />
  {:else}
    <Tabs />
    {#if app.screen === 'playing'}
      <Playing />
    {:else if app.screen === 'library'}
      <Library />
    {:else if app.screen === 'search'}
      <Search />
    {:else if app.screen === 'queue'}
      <Queue />
    {:else if app.screen === 'about'}
      <About />
    {/if}
    {#if withMini}
      <MiniBar />
    {/if}
  {/if}
  <Toast />
</div>

<style>
  .app {
    height: 100%;
    padding: 20px 24px 18px 24px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--paper);
    color: var(--ink);
  }
  .boot {
    margin: auto;
    font-size: 30px;
    font-weight: 600;
    opacity: 0.3;
  }
</style>
