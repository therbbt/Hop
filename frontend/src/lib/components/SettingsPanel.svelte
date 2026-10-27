<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import type { TerminalColors, Theme } from "../bridge";
  import { THERBBT_COLORS, THERBBT_THEME_ID } from "../therbbtTheme";

  export let terminalFontFamily: string;
  export let terminalFontSize: number;
  export let themes: Theme[];
  export let activeThemeId: string | null;
  /** `null` = automatic (on under Hyprland) - see bridge.ts. */
  export let keyboardUi: boolean | null = null;
  /** App.svelte owns the actual check (and opens the update dialog itself
   * if one's found) — this just triggers it and reports whether anything
   * came back, so this panel can show its own "you're up to date" message. */
  export let onCheckForUpdate: () => Promise<boolean>;

  const dispatch = createEventDispatcher<{
    save: {
      terminalFontFamily: string;
      terminalFontSize: number;
      terminalColors: TerminalColors;
      themes: Theme[];
      activeThemeId: string | null;
      keyboardUi: boolean | null;
    };
    cancel: void;
  }>();

  // Hop's own default ANSI palette, not a generic stock one — must match
  // tokens.css's --ansi-* defaults exactly, for both the dark and light
  // media blocks there (see the comments in tokens.css for why each color
  // is what it is). Used both to pre-fill an unset swatch and, on save, to
  // detect "the user dragged this back to the default" so it's stored as
  // an actual reset (null) rather than an explicit override that just
  // happens to match.
  //
  // Can't just read the live --ansi-* custom properties via
  // getComputedStyle here instead of hardcoding them a second time — once
  // a color IS customized, App.svelte's applyTerminalColorVars sets it as
  // an inline style on <html>, which shadows tokens.css's rule in the
  // computed value. Reading "the default" would then return whatever's
  // currently overridden, not the actual default, defeating the one case
  // (reset) that needs the real default most.
  const DARK_DEFAULT_COLORS: Record<keyof TerminalColors, string> = {
    black: "#3a3a42",
    red: "#a1362b",
    green: "#5b9bd5",
    yellow: "#a17e2b",
    blue: "#5d91d4",
    magenta: "#862ba1",
    cyan: "#2b86a1",
    white: "#97958d",
    brightBlack: "#726f66",
    brightRed: "#d05a4d",
    brightGreen: "#5b9bd5",
    brightYellow: "#d0a94d",
    brightBlue: "#97958d",
    brightMagenta: "#b14dd0",
    brightCyan: "#4db1d0",
    brightWhite: "#ecebe7",
    highlightGreen: "#4dd04d",
    highlightGet: "#2da990",
    highlightUrl: "#d76a98",
    highlightIpv6: "#8c72da",
    sidebarBackground: "#1e1e22",
    folderIcon: "#e8a33d",
    sshIndicator: "#5b9bd5",
    defaultIndicator: "#97958d",
    windowBackground: "#16161a",
    panelBackground: "#28282d",
    hoverBackground: "#303036",
    activeBackground: "#38383f",
    textPrimary: "#ecebe7",
    textSecondary: "#97958d",
    textTertiary: "#726f66",
    textDisabled: "#4d4b45",
    statusConnecting: "#e8a33d",
    statusDisconnected: "#726f66",
    statusError: "#ef4444",
  };
  const LIGHT_DEFAULT_COLORS: Record<keyof TerminalColors, string> = {
    black: "#211d18",
    red: "#c03d30",
    green: "#2563eb",
    yellow: "#856721",
    blue: "#2e6bba",
    magenta: "#a632c8",
    cyan: "#23738b",
    white: "#6b6259",
    brightBlack: "#8f8579",
    brightRed: "#a7281b",
    brightGreen: "#2563eb",
    brightYellow: "#705412",
    brightBlue: "#8f8579",
    brightMagenta: "#8e1daf",
    brightCyan: "#136076",
    brightWhite: "#e5e0d5",
    highlightGreen: "#1e8a1e",
    highlightGet: "#106554",
    highlightUrl: "#a71b55",
    highlightIpv6: "#5f35de",
    sidebarBackground: "#fefdfb",
    folderIcon: "#8a5a12",
    sshIndicator: "#2563eb",
    defaultIndicator: "#6b6259",
    windowBackground: "#faf8f4",
    panelBackground: "#f0ece4",
    hoverBackground: "#e5e0d5",
    activeBackground: "#d9d3c5",
    textPrimary: "#211d18",
    textSecondary: "#6b6259",
    textTertiary: "#8f8579",
    textDisabled: "#b3aa9c",
    statusConnecting: "#8a5a12",
    statusDisconnected: "#8f8579",
    statusError: "#ef4444",
  };
  const DEFAULT_COLORS =
    typeof window !== "undefined" && window.matchMedia("(prefers-color-scheme: light)").matches
      ? LIGHT_DEFAULT_COLORS
      : DARK_DEFAULT_COLORS;

  // Grouped by what each color is actually *for*, not by ANSI slot/weight —
  // "Standard"/"Bright"/"Dedicated" told you where a color lives in the
  // palette, not what changing it would actually affect, so finding "the
  // folder color" meant already knowing it's Bright Blue. `use` is still
  // what's shown first; `secondary` carries the ANSI name for anyone who
  // thinks in those terms (or has an external tool writing to that code
  // directly), or the non-ANSI explanation for the four dedicated colors
  // that aren't a real slot at all (see each one's `--highlight-*` note in
  // tokens.css). See each rule's own color choice in terminalHighlight.ts
  // for where every one of these actually comes from.
  interface Swatch {
    key: keyof TerminalColors;
    use: string;
    secondary: string;
    tooltip: string;
  }
  const COLOR_GROUPS: Array<{ title: string; swatches: Swatch[] }> = [
    {
      title: "Network",
      swatches: [
        { key: "blue", use: "MAC addresses", secondary: "Blue", tooltip: "ANSI: Blue" },
        { key: "cyan", use: "IPv4 addresses", secondary: "Cyan", tooltip: "ANSI: Cyan" },
        {
          key: "highlightIpv6",
          use: "IPv6 addresses",
          secondary: "Split from IPv4 above so each can be set independently",
          tooltip: "Not part of the 16-color ANSI palette",
        },
        { key: "magenta", use: "Ports (host:port, ->port/tcp)", secondary: "Magenta", tooltip: "ANSI: Magenta" },
        { key: "yellow", use: "Subnet masks, IP prefix lengths (e.g. /24)", secondary: "Yellow", tooltip: "ANSI: Yellow" },
        {
          key: "brightMagenta",
          use: "Interface names, file sizes, hashes/container IDs",
          secondary: "Bright Magenta",
          tooltip: "ANSI: Bright Magenta",
        },
        {
          key: "highlightGet",
          use: "HTTP GET requests",
          secondary: "Split from IPv4 above so each can be set independently",
          tooltip: "Not part of the 16-color ANSI palette",
        },
        {
          key: "highlightUrl",
          use: "URLs",
          secondary: "Split from MAC above so each can be set independently",
          tooltip: "Not part of the 16-color ANSI palette",
        },
      ],
    },
    {
      title: "Files & Directories",
      swatches: [
        { key: "white", use: "File paths, regular files in ls -l", secondary: "White", tooltip: "ANSI: White" },
        { key: "brightBlue", use: "Folders & directories", secondary: "Bright Blue", tooltip: "ANSI: Bright Blue" },
        { key: "brightCyan", use: "Symlinks, JSON keys", secondary: "Bright Cyan", tooltip: "ANSI: Bright Cyan" },
      ],
    },
    {
      title: "Logs & Status",
      swatches: [
        { key: "red", use: "ERROR log lines", secondary: "Red", tooltip: "ANSI: Red" },
        {
          key: "brightRed",
          use: "Critical errors, DELETE, 5xx status, danger levels",
          secondary: "Bright Red",
          tooltip: "ANSI: Bright Red",
        },
        { key: "brightYellow", use: "WARN logs, PUT, caution levels", secondary: "Bright Yellow", tooltip: "ANSI: Bright Yellow" },
        {
          key: "brightBlack",
          use: "Timestamps, DEBUG/TRACE logs, disconnected status",
          secondary: "Bright Black",
          tooltip: "ANSI: Bright Black",
        },
        {
          key: "highlightGreen",
          use: "Success (good/positive highlights)",
          secondary: "e.g. HTTP 2xx, an executable file, a service that's up",
          tooltip: "Not part of the 16-color ANSI palette",
        },
      ],
    },
    {
      title: "Shell Prompt",
      swatches: [
        { key: "green", use: "Matches Accent — a shell prompt's hostname", secondary: "Green", tooltip: "ANSI: Green" },
        {
          key: "brightGreen",
          use: "Matches Accent — a bold shell prompt's hostname",
          secondary: "Bright Green",
          tooltip: "ANSI: Bright Green",
        },
      ],
    },
    {
      title: "Other",
      swatches: [
        {
          key: "black",
          use: "World-writable directories in ls --color, plus any shell text using plain black",
          secondary: "Black",
          tooltip: "ANSI: Black",
        },
        { key: "brightWhite", use: "Brightest default text", secondary: "Bright White", tooltip: "ANSI: Bright White" },
      ],
    },
    {
      // The app's own UI chrome, not terminal text — everything above this
      // point changes what shows up *inside* a terminal; everything here
      // changes Hop itself. Kept in the same theme/swatch list anyway
      // (see TerminalColors's own doc comment in bridge.ts) rather than a
      // separate system, so one theme covers both together.
      title: "App Interface",
      swatches: [
        { key: "windowBackground", use: "Window background", secondary: "Behind everything — sidebar, tab strip, every panel", tooltip: "Not part of the 16-color ANSI palette" },
        { key: "sidebarBackground", use: "Sidebar background", secondary: "Also the top action bar — same zone", tooltip: "Not part of the 16-color ANSI palette" },
        { key: "panelBackground", use: "Raised panels", secondary: "The tab strip itself, dropdown menus, overlays", tooltip: "Not part of the 16-color ANSI palette" },
        { key: "hoverBackground", use: "Hover state", secondary: "Hovered rows in the sidebar and tab strip", tooltip: "Not part of the 16-color ANSI palette" },
        { key: "activeBackground", use: "Active / pressed state", secondary: "The active tab, a pressed button", tooltip: "Not part of the 16-color ANSI palette" },
        { key: "textPrimary", use: "Primary text", secondary: "Main text color throughout the app", tooltip: "Not part of the 16-color ANSI palette" },
        { key: "textSecondary", use: "Secondary text", secondary: "Muted text — session names, field labels", tooltip: "Not part of the 16-color ANSI palette" },
        { key: "textTertiary", use: "Tertiary text", secondary: "Dimmest text — hints, section titles", tooltip: "Not part of the 16-color ANSI palette" },
        { key: "textDisabled", use: "Disabled text", secondary: "Text on a disabled control", tooltip: "Not part of the 16-color ANSI palette" },
        { key: "folderIcon", use: "Sidebar folder icon", secondary: "Different from Folders & directories above (that's inside a terminal)", tooltip: "Not part of the 16-color ANSI palette" },
        { key: "sshIndicator", use: "SSH indicator", secondary: "The tab/pane dot and sidebar icon that mark a session as SSH", tooltip: "Not part of the 16-color ANSI palette" },
        {
          key: "defaultIndicator",
          use: "Non-SSH session indicator",
          secondary: "Same dot/icon, for shell, serial, telnet & RDP sessions",
          tooltip: "Not part of the 16-color ANSI palette",
        },
        { key: "statusConnecting", use: "Connecting status", secondary: "Status dot while a session is still connecting", tooltip: "Not part of the 16-color ANSI palette" },
        { key: "statusDisconnected", use: "Disconnected status", secondary: "Status dot for a closed session", tooltip: "Not part of the 16-color ANSI palette" },
        { key: "statusError", use: "Error status", secondary: "Status dot/text for a failed session", tooltip: "Not part of the 16-color ANSI palette" },
      ],
    },
  ];

  // Default is a theme like any other — pinned first, always present,
  // just not deletable — rather than a separate "reset" action living
  // outside the theme picker. This id is a plain sentinel, never a real
  // theme's (those are crypto.randomUUID()), so it can never collide.
  const DEFAULT_THEME_ID = "default";
  // Hop's second built-in theme (THERBBT_THEME_ID, in therbbtTheme.ts) is
  // pinned right after Default - same fixed/not-deletable treatment, just
  // resolving to THERBBT_COLORS instead of DEFAULT_COLORS. A user can still
  // freely create their own themes on top of either one via "New theme…",
  // same as always.

  let fontFamily = terminalFontFamily;
  let layoutMode: boolean | null = keyboardUi;
  const LAYOUT_MODES: Array<{ value: boolean | null; label: string }> = [
    { value: null, label: "Auto" },
    { value: true, label: "On" },
    { value: false, label: "Off" },
  ];
  let fontSize = terminalFontSize;
  // Local working copy — themes/activeThemeId props are this panel's last
  // *saved* state; edits here (create/delete a theme, pick one to load)
  // only reach App.svelte on Save, same as every color swatch already
  // works. There's no "Custom" state at all — colors are always either
  // Default's (fixed) or a real saved theme's, so `selectedThemeId` is
  // always one of those, never an orphaned in-between. An `activeThemeId`
  // prop pointing at a theme that's since been deleted (or a `null` from
  // a config saved before themes existed) both fall back to Default the
  // same way.
  let savedThemes: Theme[] = [...themes];
  let selectedThemeId: string =
    activeThemeId !== null && (activeThemeId === THERBBT_THEME_ID || savedThemes.some((t) => t.id === activeThemeId))
      ? activeThemeId
      : DEFAULT_THEME_ID;
  // Always holds a real hex per swatch (never null) so <input type="color">
  // always has something valid to show. Deliberately *not* seeded from
  // `terminalColors` directly — always derived from whichever theme
  // `selectedThemeId` above resolved to, so a leftover override from
  // before themes existed (real colors, but no theme to attribute them
  // to) collapses into Default's actual fixed colors on open, rather than
  // silently displaying as if Default were selected while showing
  // something else.
  let colors: Record<keyof TerminalColors, string> =
    selectedThemeId === DEFAULT_THEME_ID
      ? { ...DEFAULT_COLORS }
      : selectedThemeId === THERBBT_THEME_ID
        ? { ...THERBBT_COLORS }
        : { ...DEFAULT_COLORS, ...stripNulls(savedThemes.find((t) => t.id === selectedThemeId)?.colors ?? {}) };
  let showNewThemeInput = false;
  let newThemeName = "";
  let panelEl: HTMLDivElement;
  let newThemeInputEl: HTMLInputElement;

  // Custom popup dropdown, not a native <select> — same reasoning as
  // FolderSelect.svelte: a native select's open option list always uses
  // the OS's own accent color for hover/selection, clashing with Hop's
  // own accent no matter what CSS is applied to the closed control.
  let themeMenuOpen = false;
  let themeTriggerEl: HTMLButtonElement;
  let themeMenuEl: HTMLDivElement | undefined;
  $: selectedThemeName =
    selectedThemeId === DEFAULT_THEME_ID
      ? "Default"
      : selectedThemeId === THERBBT_THEME_ID
        ? "therbbt"
        : (savedThemes.find((t) => t.id === selectedThemeId)?.name ?? "Default");

  function pickTheme(id: string) {
    loadTheme(id);
    themeMenuOpen = false;
  }

  type SettingsTab = "terminal" | "colors" | "updates";
  const TABS: Array<{ id: SettingsTab; label: string }> = [
    { id: "terminal", label: "Terminal" },
    { id: "colors", label: "Colors" },
    { id: "updates", label: "Updates" },
  ];
  let activeTab: SettingsTab = "terminal";

  let appVersion = "";
  let checkingForUpdate = false;
  let updateCheckMessage = "";
  let updateCheckError = "";

  onMount(() => {
    void getVersion().then((v) => (appVersion = v));
  });

  async function checkForUpdates() {
    checkingForUpdate = true;
    updateCheckMessage = "";
    updateCheckError = "";
    try {
      const found = await onCheckForUpdate();
      // If one was found, App.svelte already opens the update dialog with
      // the changelog on top of this panel — nothing more to show here.
      if (!found) updateCheckMessage = "You're up to date.";
    } catch (err) {
      updateCheckError = err instanceof Error ? err.message : "Failed to check for updates.";
    } finally {
      checkingForUpdate = false;
    }
  }

  function stripNulls(input: TerminalColors): Partial<Record<keyof TerminalColors, string>> {
    const result: Partial<Record<keyof TerminalColors, string>> = {};
    for (const key of Object.keys(input) as Array<keyof TerminalColors>) {
      const value = input[key];
      if (value) result[key] = value;
    }
    return result;
  }

  // A swatch matching the default is stored as unset (null) rather than an
  // override that happens to equal it — keeps a config someone hasn't
  // touched clean, and makes loading Default behave like the old "reset"
  // button did. Also what a saved theme's own `colors` holds: not all 24
  // swatches verbatim, just the ones that differ from default, exactly
  // like the top-level override set. Loading a theme (loadTheme below)
  // re-expands it back out with `{ ...DEFAULT_COLORS, ...stripNulls(...) }`,
  // same as this panel already does with the selected theme's colors on
  // open (see the `colors` initializer above).
  function diffFromDefaults(source: Record<keyof TerminalColors, string>): TerminalColors {
    const overrides: TerminalColors = {};
    for (const key of Object.keys(source) as Array<keyof TerminalColors>) {
      if (source[key].toLowerCase() !== DEFAULT_COLORS[key].toLowerCase()) {
        overrides[key] = source[key];
      }
    }
    return overrides;
  }

  function loadTheme(id: string): void {
    selectedThemeId = id;
    if (id === DEFAULT_THEME_ID) {
      colors = { ...DEFAULT_COLORS };
      return;
    }
    if (id === THERBBT_THEME_ID) {
      colors = { ...THERBBT_COLORS };
      return;
    }
    const theme = savedThemes.find((t) => t.id === id);
    if (!theme) return;
    colors = { ...DEFAULT_COLORS, ...stripNulls(theme.colors) };
  }

  // Both built-in themes' colors are fixed — every swatch is rendered
  // `disabled` (see the markup below) whenever this is true, so there's
  // nothing to edit rather than something to edit that then bounces back.
  // Customizing anything requires "New theme" first — see confirmNewTheme
  // below.
  $: colorsLocked = selectedThemeId === DEFAULT_THEME_ID || selectedThemeId === THERBBT_THEME_ID;

  // A real saved theme, though, *is* live-edited in place: picking one
  // (or just having created one below) and then tweaking a swatch updates
  // that theme's own stored colors directly, rather than quietly drifting
  // into a detached, unnamed state that would silently lose the change the
  // next time this same theme gets loaded. Nothing here reaches App.svelte
  // until this panel's own Save, so it's exactly as reversible via Cancel
  // as every other edit in this panel.
  $: if (selectedThemeId !== DEFAULT_THEME_ID && selectedThemeId !== THERBBT_THEME_ID) {
    const id = selectedThemeId;
    const nextColors = diffFromDefaults(colors);
    savedThemes = savedThemes.map((t) => (t.id === id ? { ...t, colors: nextColors } : t));
  }

  // Name-first: this creates the theme immediately (as a snapshot of
  // whatever's currently showing — Default's colors, or a duplicate of
  // whichever theme was selected before) and selects it, rather than
  // waiting until you're done picking colors to ask what to call them.
  // Every swatch changed after this point updates the new theme directly,
  // per the reactive block above.
  function startNewTheme() {
    newThemeName = "";
    showNewThemeInput = true;
    // The input isn't in the DOM yet on this same tick (the {#if} below
    // hasn't rendered it) — same await-a-tick-via-autofocus-action pattern
    // FolderNode's rename input uses, via focusAndSelect's use: directive
    // there; a plain autofocus attribute is simpler here since there's no
    // existing value to select.
    queueMicrotask(() => newThemeInputEl?.focus());
  }

  function cancelNewTheme() {
    showNewThemeInput = false;
  }

  function confirmNewTheme() {
    const name = newThemeName.trim();
    if (!name) return;
    const theme: Theme = { id: crypto.randomUUID(), name, colors: diffFromDefaults(colors) };
    savedThemes = [...savedThemes, theme];
    selectedThemeId = theme.id;
    showNewThemeInput = false;
  }

  function deleteSelectedTheme() {
    if (selectedThemeId === DEFAULT_THEME_ID || selectedThemeId === THERBBT_THEME_ID) return;
    savedThemes = savedThemes.filter((t) => t.id !== selectedThemeId);
    loadTheme(DEFAULT_THEME_ID);
  }

  $: canSubmit = fontFamily.trim().length > 0 && fontSize >= 8 && fontSize <= 32;

  function submit() {
    if (!canSubmit) return;
    dispatch("save", {
      terminalFontFamily: fontFamily.trim(),
      terminalFontSize: fontSize,
      terminalColors: diffFromDefaults(colors),
      themes: savedThemes,
      // DEFAULT_THEME_ID is a UI-only sentinel, never a real theme — stored
      // as `null` (its own meaning on this field, see bridge.ts's Theme
      // doc comment), the same as it arrived as a prop. THERBBT_THEME_ID is
      // also a sentinel (THERBBT_COLORS isn't a real Theme either, same as
      // DEFAULT_COLORS isn't), but it stores as the literal "therbbt"
      // string rather than null, since Settings needs to tell "Default"
      // and "therbbt" apart on next open — loadConfig's fallback-to-Default
      // handling for an unrecognized id (a theme deleted elsewhere, or a
      // config from before therbbt existed) still applies to any value
      // that isn't one of these two sentinels or a real saved theme's id.
      activeThemeId: selectedThemeId === DEFAULT_THEME_ID ? null : selectedThemeId,
      keyboardUi: layoutMode,
    });
  }

  // Escape closes just the theme dropdown first, if it's open, same as
  // FolderSelect.svelte's own Escape handling — only a second press (or
  // one with the dropdown already closed) closes the whole panel. A
  // component can only have one <svelte:window> below, so this and the
  // theme menu's own outside-click both have to live in the one pair of
  // handlers rather than each having their own, the way FolderSelect gets
  // to when it's not nested inside another full-screen listener already.
  function handleKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape") return;
    event.preventDefault();
    if (themeMenuOpen) {
      themeMenuOpen = false;
      return;
    }
    dispatch("cancel");
  }

  function handleOutsideClick(event: MouseEvent) {
    const target = event.target as Node;
    if (
      themeMenuOpen &&
      themeTriggerEl &&
      !themeTriggerEl.contains(target) &&
      !(themeMenuEl && themeMenuEl.contains(target))
    ) {
      themeMenuOpen = false;
    }
    if (panelEl && !panelEl.contains(target)) {
      dispatch("cancel");
    }
  }
</script>


<svelte:window on:keydown={handleKeydown} on:mousedown={handleOutsideClick} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay">
  <div class="panel" bind:this={panelEl} role="dialog" aria-modal="true" aria-label="Settings">
    <header>
      <h2>Settings</h2>
      <button class="close" aria-label="Close" title="Close" on:click={() => dispatch("cancel")}>
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
          <path d="M2 2l12 12M14 2L2 14" />
        </svg>
      </button>
    </header>

    <div class="tabbar" role="tablist">
      {#each TABS as tab (tab.id)}
        <button
          class="tabbar-btn"
          class:active={activeTab === tab.id}
          role="tab"
          aria-selected={activeTab === tab.id}
          type="button"
          on:click={() => (activeTab = tab.id)}
        >
          <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round">
            {#if tab.id === "terminal"}
              <rect x="1.5" y="2.5" width="13" height="11" />
              <path d="M4 6l2 2-2 2M8 10.5h4" />
            {:else if tab.id === "colors"}
              <path d="M8 1.8a6.2 6.2 0 1 0 0 12.4c1 0 1.4-.6 1.4-1.3 0-.9-.8-1.2-.8-2 0-.7.6-1.2 1.3-1.2h1.6a2.7 2.7 0 0 0 2.7-2.7C14.2 4.2 11.4 1.8 8 1.8z" />
              <circle cx="4.9" cy="7.2" r=".7" />
              <circle cx="7.2" cy="4.6" r=".7" />
              <circle cx="10.4" cy="5" r=".7" />
            {:else}
              <path d="M8 2v8M4.5 6.8 8 10.3l3.5-3.5M2.5 13.5h11" />
            {/if}
          </svg>
          {tab.label}
        </button>
      {/each}
    </div>

    <!-- FlashPad's settings layout: a box per group with its title inside,
         one row per setting - label on the left, control(s) on the right -
         and one text size throughout; titles and details are only dimmer. -->
    <div class="content">
      {#if activeTab === "terminal"}
        <section class="settings-group">
          <h3 class="settings-group-title">Font</h3>
          <div class="setting">
            <div class="setting-text">
              <span class="setting-label">Font family</span>
              <span class="setting-detail">"monospace" is the system default</span>
            </div>
            <input class="setting-input font-input" type="text" bind:value={fontFamily} placeholder="monospace" aria-label="Font family" />
          </div>
          <div class="setting">
            <div class="setting-text">
              <span class="setting-label">Font size</span>
            </div>
            <div class="stepper">
              <button
                class="step-btn"
                type="button"
                disabled={fontSize <= 8}
                on:click={() => (fontSize = Math.max(8, fontSize - 1))}
                aria-label="Decrease"
              >
                −
              </button>
              <input class="step-input" type="number" min="8" max="32" bind:value={fontSize} aria-label="Font size" />
              <button
                class="step-btn"
                type="button"
                disabled={fontSize >= 32}
                on:click={() => (fontSize = Math.min(32, fontSize + 1))}
                aria-label="Increase"
              >
                +
              </button>
            </div>
          </div>
        </section>

        <section class="settings-group">
          <h3 class="settings-group-title">Layout</h3>
          <div class="setting">
            <div class="setting-text">
              <span class="setting-label">Keyboard-first layout</span>
              <span class="setting-detail">Hotkeys and Ctrl+Shift+P; Auto turns it on under Hyprland</span>
            </div>
            <div class="segmented" role="radiogroup" aria-label="Keyboard-first layout">
              {#each LAYOUT_MODES as mode (mode.label)}
                <button
                  type="button"
                  role="radio"
                  aria-checked={layoutMode === mode.value}
                  class:active={layoutMode === mode.value}
                  on:click={() => (layoutMode = mode.value)}
                >
                  {mode.label}
                </button>
              {/each}
            </div>
          </div>
        </section>
      {:else if activeTab === "colors"}
        <section class="settings-group">
          <h3 class="settings-group-title">Theme</h3>
          <div class="setting">
            <div class="setting-text">
              <span class="setting-label">Theme</span>
            </div>
            <div class="setting-controls">
              <div class="theme-picker">
                <button
                  type="button"
                  class="select"
                  bind:this={themeTriggerEl}
                  on:click={() => (themeMenuOpen = !themeMenuOpen)}
                  aria-haspopup="listbox"
                  aria-expanded={themeMenuOpen}
                >
                  <span class="select-label">{selectedThemeName}</span>
                  <svg class="caret" width="9" height="9" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M2.5 3.5L5 6.5L7.5 3.5" />
                  </svg>
                </button>
                {#if themeMenuOpen}
                  <div class="dropdown-menu" bind:this={themeMenuEl} role="listbox">
                    <button
                      type="button"
                      class="dropdown-item"
                      class:active={selectedThemeId === DEFAULT_THEME_ID}
                      role="option"
                      aria-selected={selectedThemeId === DEFAULT_THEME_ID}
                      on:click={() => pickTheme(DEFAULT_THEME_ID)}
                    >
                      <span>Default</span>
                      <span class="dropdown-detail">built-in</span>
                    </button>
                    <button
                      type="button"
                      class="dropdown-item"
                      class:active={selectedThemeId === THERBBT_THEME_ID}
                      role="option"
                      aria-selected={selectedThemeId === THERBBT_THEME_ID}
                      on:click={() => pickTheme(THERBBT_THEME_ID)}
                    >
                      <span>therbbt</span>
                      <span class="dropdown-detail">built-in</span>
                    </button>
                    {#each savedThemes as theme (theme.id)}
                      <button
                        type="button"
                        class="dropdown-item"
                        class:active={selectedThemeId === theme.id}
                        role="option"
                        aria-selected={selectedThemeId === theme.id}
                        on:click={() => pickTheme(theme.id)}
                      >
                        <span>{theme.name}</span>
                      </button>
                    {/each}
                  </div>
                {/if}
              </div>
              {#if !showNewThemeInput}
                <button class="btn" type="button" on:click={startNewTheme}>New…</button>
                <button class="btn" type="button" disabled={colorsLocked} on:click={deleteSelectedTheme}>Delete</button>
              {/if}
            </div>
          </div>
          {#if showNewThemeInput}
            <div class="setting-block">
              <div class="setting-controls">
                <input
                  class="setting-input theme-name-input"
                  type="text"
                  placeholder="Theme name"
                  bind:value={newThemeName}
                  bind:this={newThemeInputEl}
                  on:keydown|stopPropagation={(e) => {
                    if (e.key === "Enter") {
                      e.preventDefault();
                      confirmNewTheme();
                    } else if (e.key === "Escape") {
                      e.preventDefault();
                      cancelNewTheme();
                    }
                  }}
                />
                <button class="btn" type="button" disabled={!newThemeName.trim()} on:click={confirmNewTheme}>Create</button>
                <button class="btn" type="button" on:click={cancelNewTheme}>Cancel</button>
              </div>
            </div>
          {/if}
          <p class="setting-message">
            {#if colorsLocked}
              {selectedThemeName}'s colors are fixed - "New…" copies them into a theme you can edit.
            {:else}
              Changes go straight into this theme. Only this machine's config, never synced.
            {/if}
          </p>
        </section>

        {#each COLOR_GROUPS as group (group.title)}
          <section class="settings-group">
            <h3 class="settings-group-title">{group.title}</h3>
            {#each group.swatches as swatch (swatch.key)}
              <label class="setting color-row" class:locked={colorsLocked} title={swatch.tooltip}>
                <span class="setting-text">
                  <span class="setting-label">{swatch.use}</span>
                  <span class="setting-detail">{swatch.secondary}</span>
                </span>
                <input type="color" bind:value={colors[swatch.key]} aria-label={swatch.use} disabled={colorsLocked} />
              </label>
            {/each}
          </section>
        {/each}
      {:else}
        <section class="settings-group">
          <h3 class="settings-group-title">Updates</h3>
          <div class="setting">
            <div class="setting-text">
              <span class="setting-label">{appVersion ? `Hop ${appVersion}` : "Hop"}</span>
              {#if updateCheckMessage}
                <span class="setting-detail">{updateCheckMessage}</span>
              {/if}
            </div>
            <button class="btn" disabled={checkingForUpdate} on:click={checkForUpdates}>
              {checkingForUpdate ? "Checking…" : "Check for updates"}
            </button>
          </div>
          {#if updateCheckError}
            <p class="setting-message error">{updateCheckError}</p>
          {/if}
        </section>
      {/if}
    </div>

    {#if activeTab !== "updates"}
      <div class="actions">
        <button class="btn" on:click={() => dispatch("cancel")}>Cancel</button>
        <button class="btn primary" disabled={!canSubmit} on:click={submit}>Save</button>
      </div>
    {/if}
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
    width: min(560px, 92vw);
    max-height: 82vh;
    display: flex;
    flex-direction: column;
    background: var(--surface-1);
    border: 1px solid var(--hairline);
    border-radius: var(--radius-lg);
    overflow: hidden;
    font-size: 0.8rem;
  }
  /* Settings swaps the two backgrounds tokens.css gives every dialog (window
     background behind, sidebar background for the groups), so it matches
     FlashPad's settings. Needs the extra .overlay to outrank that
     [role="dialog"] !important rule. */
  .overlay .panel[role="dialog"] {
    background: var(--surface-1) !important;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 0.75rem 0;
    flex-shrink: 0;
  }

  h2 {
    margin: 0;
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--fg-primary);
  }

  .close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.5rem;
    height: 1.5rem;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    color: var(--fg-secondary);
    padding: 0;
    cursor: pointer;
  }
  .close:hover {
    color: var(--fg-primary);
  }

  .tabbar {
    display: flex;
    gap: 0.25rem;
    padding: 0.75rem 0.75rem 0;
    border-bottom: 1px solid var(--hairline);
    flex-shrink: 0;
  }

  .tabbar-btn {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    border: none;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--fg-secondary);
    font-size: 0.8rem;
    padding: 0.45rem 0.3rem 0.6rem;
    margin-bottom: -1px;
    cursor: pointer;
  }
  .tabbar-btn:hover {
    color: var(--fg-primary);
  }
  .tabbar-btn.active {
    color: var(--fg-primary);
    border-bottom-color: var(--accent);
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .settings-group {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-sm);
    background: var(--surface-0);
    padding: 0.55rem 0.75rem 0.2rem;
  }

  .settings-group-title {
    margin: 0 0 0.15rem;
    font-size: 0.8rem;
    font-weight: 400;
    color: var(--fg-secondary);
  }

  .setting {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    min-height: 2.5rem;
    padding: 0.35rem 0;
  }

  .setting + .setting,
  .setting + .setting-block,
  .setting + .setting-message,
  .setting-block + .setting-message {
    border-top: 1px solid var(--hairline);
  }

  .setting-text {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    min-width: 0;
  }

  .setting-label {
    color: var(--fg-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .setting-detail {
    color: var(--fg-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .setting-controls {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-shrink: 0;
  }

  .setting-message {
    margin: 0;
    padding: 0.4rem 0;
    color: var(--fg-secondary);
  }
  .setting-message.error {
    color: var(--status-error);
  }

  .setting-block {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 0.5rem 0;
  }

  .setting-input {
    height: 26px;
    min-width: 0;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    color: var(--fg-primary);
    font-size: 0.8rem;
    padding: 0 0.5rem;
  }
  .setting-input:focus-visible {
    outline: none;
    border-color: var(--accent);
  }
  .font-input {
    width: 11rem;
  }
  .theme-name-input {
    flex: 1;
  }

  .segmented {
    display: flex;
    flex-shrink: 0;
    border: 1px solid var(--hairline);
  }
  .segmented button {
    height: 24px;
    border: 0;
    background: transparent;
    color: var(--fg-secondary);
    font-size: 0.8rem;
    padding: 0 0.6rem;
    cursor: pointer;
  }
  .segmented button + button {
    border-left: 1px solid var(--hairline);
  }
  .segmented button:hover {
    color: var(--fg-primary);
  }
  .segmented button.active {
    background: var(--surface-2);
    color: var(--accent);
  }

  .stepper {
    display: flex;
    align-items: stretch;
    height: 26px;
  }
  /* Native number-input spin arrows are unstylable OS widgets on WebKitGTK -
     hidden in favor of the explicit step buttons. */
  .step-input {
    appearance: textfield;
    -webkit-appearance: none;
    width: 2.4rem;
    text-align: center;
    border: 1px solid var(--hairline);
    border-left: 0;
    border-right: 0;
    border-radius: 0;
    background: var(--surface-2);
    color: var(--fg-primary);
    font-size: 0.8rem;
    padding: 0 0.1rem;
  }
  .step-input:focus {
    outline: none;
  }
  .step-input::-webkit-outer-spin-button,
  .step-input::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }
  .step-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.6rem;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    color: var(--fg-primary);
    font-size: 0.8rem;
    line-height: 1;
    cursor: pointer;
  }
  .step-btn:hover:not(:disabled) {
    background: var(--surface-3);
  }
  .step-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  /* A trigger button + popup list rather than a native <select>, whose open
     list would use the OS's accent for hover/selection. */
  .theme-picker {
    position: relative;
  }
  .select {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.4rem;
    width: 11rem;
    height: 26px;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    color: var(--fg-primary);
    font-size: 0.8rem;
    padding: 0 0.6rem;
    text-align: left;
    cursor: pointer;
  }
  .select:hover {
    background: var(--surface-3);
  }
  .select:focus-visible {
    outline: none;
    border-color: var(--accent);
  }
  .select-label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .select .caret {
    flex-shrink: 0;
    color: var(--fg-secondary);
  }
  .dropdown-menu {
    position: absolute;
    top: calc(100% + 0.25rem);
    right: 0;
    z-index: 1001;
    min-width: 100%;
    max-height: 200px;
    overflow-y: auto;
    padding: 0.25rem;
    background: var(--surface-2);
    border: 1px solid var(--hairline);
    border-radius: var(--radius-sm);
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .dropdown-item {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    width: 100%;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--fg-primary);
    text-align: left;
    white-space: nowrap;
    padding: 0.35rem 0.6rem;
    font-size: 0.8rem;
    cursor: pointer;
  }
  .dropdown-item:hover {
    background: var(--surface-1);
  }
  .dropdown-item.active {
    color: var(--accent);
    font-weight: 600;
  }
  .dropdown-detail {
    color: var(--fg-secondary);
    font-weight: 400;
  }

  .color-row {
    cursor: pointer;
  }
  /* A built-in theme's colors: shown, not editable - the swatch dims rather
     than disappearing, so it's still obvious what color it is. */
  .color-row.locked {
    cursor: default;
  }
  .color-row.locked .setting-label {
    color: var(--fg-secondary);
  }
  .color-row.locked input[type="color"] {
    opacity: 0.5;
    cursor: default;
  }
  .color-row input[type="color"] {
    flex-shrink: 0;
    width: 26px;
    height: 26px;
    padding: 0;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-sm);
    background: none;
    cursor: pointer;
  }
  /* WebKitGTK gives a color swatch its own inset border/padding - flattened
     to a plain filled square like the other controls. */
  .color-row input[type="color"]::-webkit-color-swatch-wrapper {
    padding: 0;
  }
  .color-row input[type="color"]::-webkit-color-swatch {
    border: none;
    border-radius: var(--radius-sm);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.4rem;
    padding: 0.75rem;
    border-top: 1px solid var(--hairline);
    flex-shrink: 0;
  }

  .btn {
    height: 26px;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    color: var(--fg-primary);
    font-size: 0.8rem;
    padding: 0 0.7rem;
    white-space: nowrap;
    cursor: pointer;
  }
  .btn:hover:not(:disabled) {
    background: var(--surface-3);
  }
  .btn:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .btn.primary {
    color: var(--accent);
    border-color: var(--accent);
  }
</style>
