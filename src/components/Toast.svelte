<script>
  // aviso discreto de 3 s no rodapé, invertido tinta/papel
  import { app } from '../lib/state.svelte.js';
</script>

{#if app.toast}
  {#key app.toast.id}
    <div class="toast">{app.toast.text}</div>
  {/key}
{/if}

<style>
  .toast {
    position: fixed;
    left: 24px;
    bottom: 18px;
    max-width: calc(100% - 48px);
    background: var(--ink);
    color: var(--paper);
    font-size: 11px;
    line-height: 1;
    padding: 5px 8px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    z-index: 5;
    pointer-events: none;
  }
</style>
