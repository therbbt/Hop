<script lang="ts" context="module">
  /** One row of the palette. `run` either finishes (returns nothing - the
   * palette closes) or returns the next step, for actions that need more
   * from the user: pick a session, then a folder, or type a new name. */
  export interface PaletteItem {
    id: string;
    label: string;
    /** Muted text after the label: protocol, folder, "open tab". */
    detail?: string;
    /** Right-aligned, e.g. the action's shortcut. */
    hint?: string;
    /** Left out of the list until something is typed - keeps the root
     * palette's opening list to tabs and sessions. */
    searchOnly?: boolean;
    run: () => PaletteStep | void;
  }

  export type PaletteStep =
    | { kind: "list"; placeholder: string; items: PaletteItem[] }
    | { kind: "input"; placeholder: string; initial?: string; onSubmit: (value: string) => void };
</script>

<script lang="ts">
  import { createEventDispatcher, onMount, tick } from "svelte";
  import { fuzzyScore } from "../fuzzy";

  export let step: PaletteStep;

  const dispatch = createEventDispatcher<{ close: void }>();
  const MAX_RESULTS = 50;

  // Earlier steps, so Escape goes back one step before closing.
  let history: PaletteStep[] = [];
  let query = step.kind === "input" ? (step.initial ?? "") : "";
  let selected = 0;
  let inputEl: HTMLInputElement;
  let listEl: HTMLUListElement | undefined;

  $: results = step.kind === "list" ? filter(step.items, query) : [];
  $: if (selected >= results.length) selected = Math.max(0, results.length - 1);

  function filter(items: PaletteItem[], q: string): PaletteItem[] {
    if (!q.trim()) return items.filter((item) => !item.searchOnly).slice(0, MAX_RESULTS);
    return items
      .map((item) => ({ item, score: fuzzyScore(q, item.detail ? `${item.label} ${item.detail}` : item.label) }))
      .filter((r): r is { item: PaletteItem; score: number } => r.score !== null)
      .sort((a, b) => b.score - a.score)
      .slice(0, MAX_RESULTS)
      .map((r) => r.item);
  }

  async function show(next: PaletteStep) {
    history = [...history, step];
    step = next;
    query = next.kind === "input" ? (next.initial ?? "") : "";
    selected = 0;
    await tick();
    inputEl?.select();
  }

  function back() {
    const prev = history[history.length - 1];
    if (!prev) {
      dispatch("close");
      return;
    }
    history = history.slice(0, -1);
    step = prev;
    query = "";
    selected = 0;
  }

  function runItem(item: PaletteItem | undefined) {
    if (!item) return;
    const next = item.run();
    if (next) void show(next);
    else dispatch("close");
  }

  function submit() {
    if (step.kind === "input") {
      step.onSubmit(query);
      dispatch("close");
    } else {
      runItem(results[selected]);
    }
  }

  async function move(delta: number) {
    if (!results.length) return;
    selected = (selected + delta + results.length) % results.length;
    await tick();
    listEl?.children[selected]?.scrollIntoView({ block: "nearest" });
  }

  function handleKeydown(event: KeyboardEvent) {
    const ctrl = event.ctrlKey && !event.shiftKey && !event.altKey;
    if (event.key === "Escape") {
      event.preventDefault();
      back();
    } else if (event.key === "Enter") {
      event.preventDefault();
      submit();
    } else if (event.key === "ArrowDown" || (ctrl && (event.key === "n" || event.key === "j"))) {
      event.preventDefault();
      void move(1);
    } else if (event.key === "ArrowUp" || (ctrl && (event.key === "p" || event.key === "k"))) {
      event.preventDefault();
      void move(-1);
    }
  }

  onMount(() => {
    inputEl.focus();
    inputEl.select();
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" on:mousedown|self={() => dispatch("close")}>
  <div class="palette" role="dialog" aria-modal="true" aria-label="Command palette">
    <input
      bind:this={inputEl}
      bind:value={query}
      on:keydown={handleKeydown}
      on:input={() => (selected = 0)}
      placeholder={step.placeholder}
      spellcheck="false"
      autocomplete="off"
    />
    {#if step.kind === "list"}
      {#if results.length}
        <ul bind:this={listEl} role="listbox">
          {#each results as item, i (item.id)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <li
              role="option"
              aria-selected={i === selected}
              class:selected={i === selected}
              on:mousemove={() => (selected = i)}
              on:click={() => runItem(item)}
            >
              <span class="label">{item.label}</span>
              {#if item.detail}<span class="detail">{item.detail}</span>{/if}
              {#if item.hint}<span class="hint">{item.hint}</span>{/if}
            </li>
          {/each}
        </ul>
      {:else}
        <p class="empty">No matches</p>
      {/if}
    {:else}
      <p class="empty">Enter to confirm · Esc to go back</p>
    {/if}
  </div>
</div>

<style>
  /* Styled after Omarchy's own launcher (its shell's Menu.qml): square
     corners like Hyprland's tiles, a 2px foreground border, a half-dimmed
     backdrop, monospace text, roomy rows, and the selected row marked by a
     faint wash plus accent-colored text rather than a solid block. */
  .overlay {
    position: fixed;
    inset: var(--window-shadow-margin);
    z-index: 1000;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 14vh;
    background: color-mix(in srgb, var(--surface-0) 55%, transparent);
  }
  .palette {
    width: min(520px, calc(100vw - 32px));
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3);
    background: var(--surface-0);
    border: 2px solid var(--fg-secondary);
    font-family: var(--font-mono);
  }
  input {
    border: none;
    outline: none;
    background: transparent;
    color: var(--fg-primary);
    font-family: inherit;
    font-size: 15px;
    padding: var(--space-1) var(--space-2);
  }
  /* The palette is the only thing taking keys while open, so the app's
     global focus ring (tokens.css) would only add a box inside the box. */
  input:focus-visible {
    outline: none;
    box-shadow: none;
  }
  input::placeholder {
    color: var(--fg-tertiary);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: min(380px, 60vh);
    overflow-y: auto;
    scrollbar-width: none;
  }
  ul::-webkit-scrollbar {
    width: 0;
  }
  li {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    height: 34px;
    padding: 0 var(--space-2);
    font-size: 13px;
    color: var(--fg-secondary);
    cursor: pointer;
  }
  li.selected {
    background: color-mix(in srgb, var(--fg-primary) 8%, transparent);
    color: var(--accent);
  }
  .label {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .detail {
    min-width: 0;
    color: var(--fg-tertiary);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .hint {
    margin-left: auto;
    flex-shrink: 0;
    color: var(--fg-tertiary);
    font-size: 11px;
  }
  .empty {
    margin: 0;
    padding: var(--space-1) var(--space-2);
    font-size: 12px;
    color: var(--fg-tertiary);
  }
</style>
