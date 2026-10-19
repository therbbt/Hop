<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import RingMark from "./RingMark.svelte";

  import { shortcutKeys, type ShortcutId } from "../shortcuts";

  /** Keyboard-first layout: hotkeys to start from instead of buttons. */
  export let keyboard = false;

  const dispatch = createEventDispatcher<{ newShell: void; newSsh: void; newSerial: void; newRdp: void }>();
  const KEYBOARD_HINTS: Array<{ id: ShortcutId; label: string }> = [
    { id: "palette", label: "Sessions and commands" },
    { id: "newShell", label: "New local shell" },
    { id: "newSession", label: "New session" },
    { id: "shortcuts", label: "All shortcuts" },
  ];
</script>

<div class="empty-main">
  <RingMark size={64} />
  <p class="title">No sessions open</p>
  {#if keyboard}
    <dl class="hints">
      {#each KEYBOARD_HINTS as hint (hint.id)}
        <dt>{shortcutKeys(hint.id)}</dt>
        <dd>{hint.label}</dd>
      {/each}
    </dl>
  {:else}
    <div class="actions">
      <button class="cta" on:click={() => dispatch("newSsh")}>Connect over SSH</button>
      <button class="cta secondary" on:click={() => dispatch("newRdp")}>Connect over RDP</button>
      <button class="cta secondary" on:click={() => dispatch("newSerial")}>Connect over serial</button>
      <button class="cta secondary" on:click={() => dispatch("newShell")}>Open a local shell</button>
    </div>
  {/if}
</div>

<style>
  .empty-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    background: var(--surface-0);
  }
  .title {
    margin: 0;
    color: var(--fg-secondary);
    font-size: 13px;
  }
  .hints {
    display: grid;
    grid-template-columns: auto auto;
    gap: var(--space-1) var(--space-4);
    margin: 0;
    font-size: 12px;
  }
  .hints dt {
    font-family: var(--font-mono);
    color: var(--fg-primary);
    text-align: right;
  }
  .hints dd {
    margin: 0;
    color: var(--fg-tertiary);
  }
  .actions {
    display: flex;
    gap: var(--space-2);
  }
  .cta {
    padding: var(--space-2) var(--space-4);
    background: var(--accent);
    color: var(--accent-fg);
    border: none;
    border-radius: var(--radius-md);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }
  .cta:hover {
    filter: brightness(1.08);
  }
  .cta.secondary {
    background: var(--surface-2);
    color: var(--fg-primary);
    font-weight: 500;
  }
  .cta.secondary:hover {
    background: var(--surface-3);
    filter: none;
  }
</style>
