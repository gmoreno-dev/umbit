<script>
  // linha de comando do rodapé (":sobre", ":tema papel", ":sair", ":topo")
  import { app, runCommand, closeCommand } from '../lib/state.svelte.js';

  let { bottom = false } = $props();
  let input = $state(null);

  $effect(() => {
    if (input) input.focus();
  });

  function onkeydown(e) {
    if (e.key === 'Enter') {
      e.preventDefault();
      runCommand(app.cmd);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      closeCommand();
    }
  }
</script>

<div class="cmd" class:bottom>
  <span class="colon">:</span>
  <input bind:this={input} bind:value={app.cmd} {onkeydown} onblur={closeCommand} spellcheck="false" autocomplete="off" autocapitalize="off" aria-label="comando" />
</div>

<style>
  .cmd {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    margin-top: 10px;
    height: 16px;
    flex-shrink: 0;
  }
  .cmd.bottom {
    margin-top: auto;
  }
  .colon {
    opacity: 0.6;
  }
  input {
    flex: 1;
    min-width: 0;
    font-size: 12px;
    caret-color: var(--ink);
  }
</style>
