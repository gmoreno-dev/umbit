<script>
  // linha de lista de 44 px. `active` = cursor (invertida). `heart` troca a meta pelo coração.
  import Heart from './Heart.svelte';
  import Cover from './Cover.svelte';
  import { app } from '../lib/state.svelte.js';

  let { text = '', meta = '', heart = false, num = null, playing = false, thumb = undefined, active = false, onclick = () => {} } = $props();
</script>

<button type="button" tabindex="-1" class="row" class:active class:withthumb={thumb !== undefined} {onclick}>
  {#if playing}
    <svg class="play" width="12" height="12" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true"><path d="M4 3l13 7-13 7z" /></svg>
  {/if}
  {#if num !== null}
    <span class="num">{num}</span>
  {/if}
  {#if thumb !== undefined}
    <span class="thumb"><Cover url={thumb} size={28} thumb ink={app.ink} paper={app.paper} /></span>
  {/if}
  <span class="text">{text}</span>
  <span class="meta">
    {#if heart}
      <Heart />
    {:else}
      {meta}
    {/if}
  </span>
</button>

<style>
  .row {
    height: 44px;
    flex-shrink: 0;
    width: 100%;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 10px;
    font-size: 14px;
    color: var(--ink);
    background: transparent;
  }
  .row.withthumb {
    gap: 10px;
  }
  .row.active {
    background: var(--ink);
    color: var(--paper);
  }
  .play {
    flex-shrink: 0;
  }
  .num {
    font-size: 12px;
    opacity: 0.5;
    width: 16px;
    flex-shrink: 0;
  }
  .thumb {
    display: block;
    flex-shrink: 0;
  }
  .text {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .meta {
    font-size: 12px;
    opacity: 0.6;
    flex-shrink: 0;
    display: flex;
    align-items: center;
  }
  .row.active .meta {
    opacity: 0.7;
  }
</style>
