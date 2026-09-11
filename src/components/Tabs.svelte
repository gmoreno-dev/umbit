<script>
  // abas do topo: tocando / biblioteca / busca / fila, e o indicador "connect"
  import { app, setScreen } from '../lib/state.svelte.js';

  const TABS = [
    ['playing', 'tocando'],
    ['library', 'biblioteca'],
    ['search', 'busca'],
    ['queue', 'fila'],
  ];
</script>

<div class="tabs pix">
  <div class="left">
    {#each TABS as [id, label] (id)}
      <button type="button" tabindex="-1" class:active={app.screen === id} onclick={() => setScreen(id)}>{label}</button>
    {/each}
  </div>
  <span class="connect" class:active={app.now.active} title={app.now.active ? 'este é o dispositivo ativo' : 'outro dispositivo está ativo'}>connect</span>
</div>

<style>
  .tabs {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    font-size: 14px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    flex-shrink: 0;
  }
  .left {
    display: flex;
    gap: 16px;
  }
  button {
    opacity: 0.45;
    border-bottom: 2px solid transparent;
    padding-bottom: 2px;
    line-height: 1;
  }
  button.active {
    opacity: 1;
    border-bottom-color: var(--ink);
  }
  .connect {
    font-size: 12px;
    opacity: 0.6;
  }
  .connect.active {
    opacity: 1;
  }
  /* janelas estreitas (mínimo 320): aperta as abas para o "connect" caber */
  @media (max-width: 359px) {
    .tabs {
      font-size: 13px;
    }
    .left {
      gap: 10px;
    }
  }
</style>
