<script lang="ts">
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher<{ cancel: void }>();

  import { SHORTCUTS as APP_SHORTCUTS } from "../shortcuts";

  /** Keyboard-first layout (see App.svelte): it has no sidebar, so the
   * sidebar toggle isn't listed. */
  export let keyboard = false;

  $: SHORTCUTS = [
    ...APP_SHORTCUTS.filter((s) => !(keyboard && s.id === "toggleSidebar")).map(({ label, keys }) => ({ label, keys })),
    // A true OS-level global hotkey (works even when Portus isn't
    // focused) - registered in Rust (see tray.rs), not App.svelte's key
    // handler like the rest. On Hyprland it's bound in the Hyprland
    // config instead (see the README).
    { label: "Show/hide Portus", keys: "Alt+T" },
  ];

  let panelEl: HTMLDivElement;

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      dispatch("cancel");
    }
  }

  function handleOutsideClick(event: MouseEvent) {
    if (panelEl && !panelEl.contains(event.target as Node)) {
      dispatch("cancel");
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" on:mousedown={handleOutsideClick}>
  <div class="panel" bind:this={panelEl} role="dialog" aria-modal="true" aria-label="Keyboard shortcuts">
    <header>
      <h2>Keyboard shortcuts</h2>
      <button class="close" aria-label="Close" title="Close" on:click={() => dispatch("cancel")}>
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
          <path d="M2 2l12 12M14 2L2 14" />
        </svg>
      </button>
    </header>

    <div class="content">
      <ul class="shortcut-list">
        {#each SHORTCUTS as shortcut (shortcut.label)}
          <li>
            <span class="label">{shortcut.label}</span>
            <span class="keys">{shortcut.keys}</span>
          </li>
        {/each}
      </ul>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: var(--window-shadow-margin);
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .panel {
    width: min(340px, 90vw);
    background: var(--surface-2);
    border-radius: var(--radius-lg);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4) var(--space-4) 0;
    flex-shrink: 0;
  }

  h2 {
    margin: 0;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--fg-primary);
  }

  .close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.5rem;
    height: 1.5rem;
    border: none;
    border-radius: var(--radius-sm);
    background: var(--surface-3);
    color: var(--fg-secondary);
    padding: 0;
    cursor: pointer;
  }
  .close:hover {
    background: var(--surface-4);
    color: var(--fg-primary);
  }

  .content {
    padding: var(--space-4);
    max-height: 70vh;
    overflow-y: auto;
  }

  .shortcut-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }
  .shortcut-list li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }
  .label {
    font-size: 0.78rem;
    color: var(--fg-primary);
  }
  .keys {
    flex-shrink: 0;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    color: var(--fg-secondary);
    background: var(--surface-1);
    padding: 0.15rem 0.45rem;
    border-radius: var(--radius-sm);
  }
</style>
