<script>
  // dicas de teclado no rodapé (11 px). com a linha de comando aberta, mostra-a no lugar.
  import { app } from '../lib/state.svelte.js';
  import CommandLine from './CommandLine.svelte';

  let { items = [], bottom = false } = $props();
</script>

{#if app.cmd !== null}
  <CommandLine {bottom} />
{:else}
  <div class="hints" class:bottom>
    {#each items as h, i (i)}
      <span>{h}</span>
    {/each}
  </div>
{/if}

<style>
  .hints {
    font-size: 11px;
    opacity: 0.5;
    display: flex;
    gap: 14px;
    margin-top: 10px;
    flex-shrink: 0;
    white-space: nowrap;
    overflow: hidden;
  }
  .hints.bottom {
    margin-top: auto;
  }
</style>
