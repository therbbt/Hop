<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { FolderAction, SaveActionInput } from "../bridge";
  import Dialog from "./Dialog.svelte";

  // Creates or edits a folder action - a status-line button in the
  // keyboard-first layout that types its command into the focused pane
  // while it's in one of the action's directories (see FolderAction in
  // bridge.ts).
  export let editAction: FolderAction | null = null;
  /** Pre-filled directory for a new action - the focused pane's own. */
  export let initialPath = "";

  const dispatch = createEventDispatcher<{
    save: SaveActionInput;
    delete: FolderAction;
    cancel: void;
  }>();

  let name = editAction?.name ?? "";
  let command = editAction?.command ?? "";
  let pathsText = (editAction?.paths ?? (initialPath ? [initialPath] : [])).join("\n");
  let statusCommand = editAction?.statusCommand ?? "";
  let stopName = editAction?.stopName ?? "";
  let stopCommand = editAction?.stopCommand ?? "";

  $: paths = pathsText
    .split("\n")
    .map((p) => p.trim())
    .filter(Boolean);
  $: canSave = name.trim().length > 0 && command.trim().length > 0 && paths.length > 0;

  function submit() {
    if (!canSave) return;
    dispatch("save", {
      id: editAction?.id ?? null,
      name: name.trim(),
      command: command.trim(),
      paths,
      statusCommand: statusCommand.trim() || null,
      stopName: stopName.trim() || null,
      stopCommand: stopCommand.trim() || null,
    });
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && (event.target as HTMLElement | null)?.tagName === "INPUT") {
      event.preventDefault();
      submit();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<Dialog label={editAction ? "Edit action" : "New action"} width="420px" on:cancel={() => dispatch("cancel")}>
  <h2 class="title">{editAction ? "Edit action" : "New action"}</h2>

  <label class="field">
    <span>Name</span>
    <!-- svelte-ignore a11y_autofocus -->
    <input type="text" bind:value={name} placeholder="start" autofocus />
  </label>

  <label class="field">
    <span>Command</span>
    <input type="text" class="mono" bind:value={command} placeholder="docker compose up" spellcheck="false" />
  </label>

  <label class="field">
    <span>Show in directories (one per line)</span>
    <textarea class="mono" rows="3" bind:value={pathsText} placeholder="~/projects/app" spellcheck="false"></textarea>
    <span class="hint">Also shows in their subdirectories. <code>~</code> is the pane's home, local or over SSH.</span>
  </label>

  <label class="field">
    <span>Running check (optional)</span>
    <input
      type="text"
      class="mono"
      bind:value={statusCommand}
      placeholder="docker compose ps --status running -q | grep -q ."
      spellcheck="false"
    />
    <span class="hint">Runs in the background every few seconds, never in the terminal. Exit 0 means running.</span>
  </label>

  <div class="row" class:disabled={!statusCommand.trim()}>
    <label class="field stop-name">
      <span>While running</span>
      <input type="text" bind:value={stopName} placeholder="stop" disabled={!statusCommand.trim()} />
    </label>
    <label class="field grow">
      <span>Stop command</span>
      <input
        type="text"
        class="mono"
        bind:value={stopCommand}
        placeholder="docker compose down"
        spellcheck="false"
        disabled={!statusCommand.trim()}
      />
    </label>
  </div>

  <div class="actions">
    {#if editAction}
      <button class="btn danger" on:click={() => editAction && dispatch("delete", editAction)}>Delete</button>
    {/if}
    <span class="spacer"></span>
    <button class="btn" on:click={() => dispatch("cancel")}>Cancel</button>
    <button class="btn primary" disabled={!canSave} on:click={submit}>Save</button>
  </div>
</Dialog>

<style>
  .title {
    margin: 0;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--fg-primary);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 0.72rem;
    color: var(--fg-secondary);
  }

  input[type="text"],
  textarea {
    background: var(--surface-1);
    border: none;
    border-radius: var(--radius-sm);
    padding: 0.4rem 0.5rem;
    color: var(--fg-primary);
    font-size: 0.8rem;
  }
  textarea {
    resize: vertical;
  }
  input[type="text"]:focus-visible,
  textarea:focus-visible {
    box-shadow: 0 0 0 2px var(--accent);
  }
  .mono {
    font-family: var(--font-mono);
  }

  .hint {
    color: var(--fg-tertiary);
  }

  .row {
    display: flex;
    gap: var(--space-2);
  }
  .row.disabled {
    opacity: 0.5;
  }
  .stop-name {
    width: 6.5rem;
  }
  .stop-name input {
    min-width: 0;
  }
  .grow {
    flex: 1;
    min-width: 0;
  }
  .hint code {
    font-family: var(--font-mono);
  }

  .actions {
    display: flex;
    gap: var(--space-2);
    margin-top: var(--space-1);
  }
  .spacer {
    flex: 1;
  }

  .btn {
    border: none;
    border-radius: var(--radius-md);
    background: var(--surface-3);
    color: var(--fg-primary);
    font-size: 0.78rem;
    padding: 0.4rem 0.9rem;
    cursor: pointer;
  }
  .btn:hover {
    background: var(--surface-4);
  }
  .btn.danger {
    color: var(--status-error);
  }
  .btn.primary {
    background: var(--accent);
    color: var(--accent-fg);
    font-weight: 600;
  }
  .btn.primary:hover {
    filter: brightness(1.08);
  }
  .btn.primary:disabled {
    background: var(--surface-4);
    color: var(--fg-disabled);
    cursor: not-allowed;
  }
</style>
