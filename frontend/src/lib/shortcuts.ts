// The app's own keyboard shortcuts, in one place: App.svelte's key
// handler matches against this list, Terminal.svelte uses it to keep
// xterm.js from also sending a matched combo to the shell, and the
// shortcuts sheet and command palette display it.
//
// Ctrl+Shift+<letter> for almost everything - xterm.js's default keymap
// doesn't turn Ctrl+Shift combos into shell control sequences, so they're
// safe to reserve even while a terminal has focus (the same convention
// Windows Terminal and GNOME Terminal use). Letters match on `event.key`,
// so they follow the active keyboard layout; Ctrl+Shift+C/V are left alone
// for copy/paste. Digits match on `event.code`, since Shift turns them
// into a layout-specific symbol in `event.key`.

export type ShortcutId =
  | "palette"
  | "newShell"
  | "newSession"
  | "sessions"
  | "closePane"
  | "splitRow"
  | "splitColumn"
  | "nextTab"
  | "prevTab"
  | "goToTab"
  | "nextPane"
  | "prevPane"
  | "renameTab"
  | "reconnect"
  | "saveAsSession"
  | "files"
  | "toggleSidebar"
  | "settings"
  | "shortcuts";

interface Combo {
  ctrl: boolean;
  shift: boolean;
  /** Matched case-insensitively against `event.key`. */
  key?: string;
  /** Matched against `event.code`. */
  code?: string;
}

export interface Shortcut {
  id: ShortcutId;
  label: string;
  /** How the shortcut is shown to people. */
  keys: string;
  combos: Combo[];
}

const ctrlShift = (key: string): Combo => ({ ctrl: true, shift: true, key });

export const SHORTCUTS: Shortcut[] = [
  { id: "palette", label: "Command palette", keys: "Ctrl+Shift+P", combos: [ctrlShift("p")] },
  { id: "newShell", label: "New local shell tab", keys: "Ctrl+Shift+T", combos: [ctrlShift("t")] },
  { id: "newSession", label: "New session…", keys: "Ctrl+Shift+N", combos: [ctrlShift("n")] },
  { id: "sessions", label: "Saved sessions…", keys: "Ctrl+Shift+O", combos: [ctrlShift("o")] },
  { id: "closePane", label: "Close pane or tab", keys: "Ctrl+Shift+W", combos: [ctrlShift("w")] },
  { id: "splitRow", label: "Split pane right", keys: "Ctrl+Shift+D", combos: [ctrlShift("d")] },
  { id: "splitColumn", label: "Split pane down", keys: "Ctrl+Shift+E", combos: [ctrlShift("e")] },
  {
    id: "nextTab",
    label: "Next tab",
    keys: "Ctrl+Tab",
    combos: [
      { ctrl: true, shift: false, key: "Tab" },
      { ctrl: true, shift: false, key: "PageDown" },
    ],
  },
  {
    id: "prevTab",
    label: "Previous tab",
    keys: "Ctrl+Shift+Tab",
    combos: [
      { ctrl: true, shift: true, key: "Tab" },
      { ctrl: true, shift: false, key: "PageUp" },
    ],
  },
  {
    id: "goToTab",
    label: "Go to tab 1–9",
    keys: "Ctrl+Shift+1…9",
    combos: ["1", "2", "3", "4", "5", "6", "7", "8", "9"].map((n) => ({ ctrl: true, shift: true, code: `Digit${n}` })),
  },
  { id: "nextPane", label: "Next pane", keys: "Ctrl+Shift+J", combos: [ctrlShift("j")] },
  { id: "prevPane", label: "Previous pane", keys: "Ctrl+Shift+K", combos: [ctrlShift("k")] },
  { id: "renameTab", label: "Rename tab…", keys: "Ctrl+Shift+L", combos: [ctrlShift("l")] },
  { id: "reconnect", label: "Reconnect", keys: "Ctrl+Shift+R", combos: [ctrlShift("r")] },
  { id: "saveAsSession", label: "Save tab as session…", keys: "Ctrl+Shift+S", combos: [ctrlShift("s")] },
  { id: "files", label: "Files (SFTP)", keys: "Ctrl+Shift+F", combos: [ctrlShift("f")] },
  { id: "toggleSidebar", label: "Toggle sidebar", keys: "Ctrl+Shift+B", combos: [ctrlShift("b")] },
  { id: "settings", label: "Settings", keys: "Ctrl+Shift+,", combos: [{ ctrl: true, shift: true, code: "Comma" }] },
  { id: "shortcuts", label: "Keyboard shortcuts", keys: "Ctrl+Shift+H", combos: [ctrlShift("h")] },
];

function comboMatches(combo: Combo, event: KeyboardEvent): boolean {
  if (event.ctrlKey !== combo.ctrl || event.shiftKey !== combo.shift || event.altKey || event.metaKey) return false;
  if (combo.code) return event.code === combo.code;
  return !!combo.key && event.key.toLowerCase() === combo.key.toLowerCase();
}

/** The shortcut `event` triggers, if any. For `goToTab`, `tabIndex` is the
 * zero-based tab it names. */
export function matchShortcut(event: KeyboardEvent): { id: ShortcutId; tabIndex?: number } | null {
  for (const shortcut of SHORTCUTS) {
    for (const combo of shortcut.combos) {
      if (comboMatches(combo, event)) {
        return shortcut.id === "goToTab" ? { id: shortcut.id, tabIndex: Number(combo.code!.slice(-1)) - 1 } : { id: shortcut.id };
      }
    }
  }
  return null;
}

export function shortcutKeys(id: ShortcutId): string {
  return SHORTCUTS.find((s) => s.id === id)?.keys ?? "";
}
