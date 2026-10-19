<script lang="ts">
  // Keyboard-first layout's tab line (see App.svelte's keyboardUi): one
  // thin row of numbered tabs, matching Ctrl+Shift+1…9. Only rendered with
  // two or more tabs open - renaming, closing and the rest are hotkeys and
  // palette actions there, so this only shows where you are.
  import { createEventDispatcher } from "svelte";
  import type { Protocol, SessionState } from "../bridge";

  export let tabs: Array<{ id: string; title: string; state: SessionState; protocol?: Protocol; folderSlug?: string | null }> = [];
  export let activeId: string | null = null;

  const dispatch = createEventDispatcher<{ select: { id: string } }>();
</script>

<div class="slim-tabs" role="tablist">
  {#each tabs as tab, i (tab.id)}
    <button
      class="slim-tab"
      class:active={tab.id === activeId}
      role="tab"
      aria-selected={tab.id === activeId}
      tabindex="-1"
      on:mousedown|preventDefault={() => dispatch("select", { id: tab.id })}
    >
      <span class="index">{i + 1}</span>
      <span class="status-dot" data-state={tab.state} data-protocol={tab.protocol}></span>
      <span class="title">{tab.title}</span>
      {#if tab.folderSlug}<span class="slug">[{tab.folderSlug}]</span>{/if}
    </button>
  {/each}
</div>

<style>
  .slim-tabs {
    flex-shrink: 0;
    display: flex;
    height: 22px;
    background: var(--surface-1);
    border-bottom: 1px solid var(--hairline);
    overflow: hidden;
  }
  .slim-tab {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    min-width: 0;
    max-width: 220px;
    padding: 0 0.7rem;
    border: none;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--fg-tertiary);
    font-family: var(--font-mono);
    font-size: 0.7rem;
    cursor: pointer;
  }
  .slim-tab.active {
    color: var(--fg-primary);
    border-bottom-color: var(--accent);
  }
  .index {
    color: var(--fg-tertiary);
  }
  .title {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .slug {
    color: var(--fg-tertiary);
  }
  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
    background: var(--status-disconnected);
  }
  .status-dot[data-state="connecting"] {
    background: var(--status-connecting);
  }
  .status-dot[data-state="connected"] {
    background: var(--status-connected-default);
  }
  .status-dot[data-state="connected"][data-protocol="ssh"] {
    background: var(--status-connected);
  }
</style>
