<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { FolderAction, Group, SaveActionInput } from "../bridge";
  import Dialog from "./Dialog.svelte";

  // Creates or edits a folder action - a status-line button in the
  // keyboard-first layout that types its command into the focused pane
  // (see FolderAction in bridge.ts).
  export let groups: Group[] = [];
  export let editAction: FolderAction | null = null;
  /** Pre-ticked folders for a new action - the active tab's own folder,
   * when it has one. */
  export let initialGroupIds: string[] = [];

  const dispatch = createEventDispatcher<{
    save: SaveActionInput;
    delete: FolderAction;
    cancel: void;
  }>();

  let name = editAction?.name ?? "";
  let command = editAction?.command ?? "";
  let selected = new Set(editAction?.groupIds ?? initialGroupIds);

  function folderPath(id: string): string {
    const names: string[] = [];
    let current: string | null = id;
    while (current && names.length < 32) {
      const group = groups.find((g) => g.id === current);
      if (!group) break;
      names.unshift(group.name);
      current = group.parentId ?? null;
    }
    return names.join("/");
  }

  $: folders = groups.map((g) => ({ id: g.id, path: folderPath(g.id) })).sort((a, b) => a.path.localeCompare(b.path));
  $: canSave = name.trim().length > 0 && command.trim().length > 0 && selected.size > 0;

  function toggle(id: string) {
    if (selected.has(id)) selected.delete(id);
    else selected.add(id);
    selected = selected;
  }

  function submit() {
    if (!canSave) return;
    dispatch("save", { id: editAction?.id ?? null, name: name.trim(), command: command.trim(), groupIds: [...selected] });
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && (event.target as HTMLElement | null)?.tagName === "INPUT") {
      event.preventDefault();
      submit();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<Dialog label={editAction ? "Edit action" : "New action"} width="380px" on:cancel={() => dispatch("cancel")}>
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

  <div class="field">
    <span>Show in folders</span>
    {#if folders.length}
      <div class="folders">
        {#each folders as folder (folder.id)}
          <label class="folder">
            <input type="checkbox" checked={selected.has(folder.id)} on:change={() => toggle(folder.id)} />
            <span>{folder.path}</span>
          </label>
        {/each}
      </div>
      <span class="hint">Also shows for sessions in their subfolders.</span>
    {:else}
      <span class="hint">No folders yet - create one first ("New folder…" in the palette).</span>
    {/if}
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

  input[type="text"] {
    background: var(--surface-1);
    border: none;
    border-radius: var(--radius-sm);
    padding: 0.4rem 0.5rem;
    color: var(--fg-primary);
    font-size: 0.8rem;
  }
  input[type="text"]:focus-visible {
    box-shadow: 0 0 0 2px var(--accent);
  }
  .mono {
    font-family: var(--font-mono);
  }

  .folders {
    display: flex;
    flex-direction: column;
    max-height: 180px;
    overflow-y: auto;
    background: var(--surface-1);
    border-radius: var(--radius-sm);
    padding: 0.3rem 0;
  }
  .folder {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.2rem 0.5rem;
    color: var(--fg-primary);
    font-size: 0.78rem;
    cursor: pointer;
  }
  .folder:hover {
    background: var(--surface-3);
  }
  .folder input {
    accent-color: var(--accent);
    margin: 0;
  }
  .hint {
    color: var(--fg-tertiary);
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
