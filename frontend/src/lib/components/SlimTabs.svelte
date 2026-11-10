<script lang="ts">
  // Keyboard-first layout's tab line (see App.svelte's keyboardUi): one
  // thin row of numbered tabs, matching Ctrl+Shift+1…9. Only rendered with
  // two or more tabs open - renaming and the rest are hotkeys and palette
  // actions there, but a tab can be closed with its × (or a middle click)
  // and right-clicked for the same menu as TabStrip's. Its empty stretch
  // past the tabs drags the window, standing in for a title bar.
  import { createEventDispatcher } from "svelte";
  import type { Protocol, SessionState } from "../bridge";
  import type { ContextMenuItem } from "./ContextMenu.svelte";

  export let tabs: Array<{ id: string; title: string; state: SessionState; protocol?: Protocol; folderSlug?: string | null }> = [];
  export let activeId: string | null = null;

  const dispatch = createEventDispatcher<{
    select: { id: string };
    close: { id: string };
    saveAs: { id: string };
    reconnect: { id: string };
    // Rendered by App.svelte, like TabStrip's.
    openContextMenu: { x: number; y: number; items: ContextMenuItem[] };
  }>();

  function openTabMenu(event: MouseEvent, tab: { id: string; state: SessionState }) {
    dispatch("openContextMenu", {
      x: event.clientX,
      y: event.clientY,
      items: [
        // Same rule as TabStrip: only once the session has dropped.
        ...(tab.state === "disconnected" ? [{ label: "Reconnect", action: () => dispatch("reconnect", { id: tab.id }) }] : []),
        { label: "Save as session…", action: () => dispatch("saveAs", { id: tab.id }) },
        { label: "", separator: true },
        { label: "Close", action: () => dispatch("close", { id: tab.id }) },
      ],
    });
  }
</script>

<!-- Right-clicking the empty stretch does nothing (not the webview's own
     menu); a tab's own menu stops its event before it gets here. -->
<!-- svelte-ignore a11y_interactive_supports_focus -->
<div class="slim-tabs" role="tablist" data-tauri-drag-region on:contextmenu|preventDefault>
  {#each tabs as tab, i (tab.id)}
    <button
      class="slim-tab"
      class:active={tab.id === activeId}
      role="tab"
      aria-selected={tab.id === activeId}
      tabindex="-1"
      on:mousedown|preventDefault={(e) => e.button === 0 && dispatch("select", { id: tab.id })}
      on:auxclick|preventDefault={(e) => e.button === 1 && dispatch("close", { id: tab.id })}
      on:contextmenu|preventDefault|stopPropagation={(e) => openTabMenu(e, tab)}
    >
      <span class="index">{i + 1}</span>
      <span class="status-dot" data-state={tab.state} data-protocol={tab.protocol}></span>
      <span class="title">{tab.title}</span>
      {#if tab.folderSlug}<span class="slug">[{tab.folderSlug}]</span>{/if}
      <!-- A span, not a button: it sits inside the tab's button. -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span
        class="close"
        title="Close tab"
        aria-hidden="true"
        on:mousedown|preventDefault|stopPropagation={(e) => e.button === 0 && dispatch("close", { id: tab.id })}>×</span
      >
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
  /* Only on the active or hovered tab, so the line stays quiet. */
  .close {
    visibility: hidden;
    margin-right: -0.3rem;
    padding: 0 2px;
    border-radius: var(--radius-sm);
    line-height: 1;
    color: var(--fg-tertiary);
  }
  .slim-tab.active .close,
  .slim-tab:hover .close {
    visibility: visible;
  }
  .close:hover {
    color: var(--fg-primary);
    background: var(--surface-4);
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
