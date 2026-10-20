import type { TerminalColors } from "./bridge";

// The id Settings stores in `activeThemeId` for this theme: a plain
// sentinel, like Default's, never a real saved theme's randomUUID().
export const THERBBT_THEME_ID = "therbbt";

// Hop's second built-in theme — therbbt's own branding, matching the
// Therbbthole Omarchy theme so Hop looks like the rest of the desktop.
// ANSI slots are exactly what Omarchy generates for its terminals from
// that theme's colors.toml (color0 = background, color8 = muted, ...), so
// a listing in Hop and in kitty/alacritty reads the same. The window and
// text slots follow the same file: background / dark_background /
// lighter_background for the surfaces, foreground / dark_foreground /
// muted for the text tiers. The two accents keep their meanings: pink
// (the theme's accent) marks "this is a live session" (sshIndicator,
// URLs), cyan marks "this is a place" (folders, local sessions), so a
// folder row and an SSH row stay distinguishable at a glance.
// Deliberately dark-only, like the Omarchy theme itself.
const THERBBT_PINK = "#e955a0";
const THERBBT_CYAN = "#7ad7e0";
export const THERBBT_COLORS: Record<keyof TerminalColors, string> = {
  black: "#170c29",
  red: "#ff5c8a",
  green: "#9be2b0",
  yellow: "#f7c873",
  blue: "#9d8cf0",
  magenta: "#c85aac",
  cyan: THERBBT_CYAN,
  white: "#fde3e3",
  brightBlack: "#6e5291",
  brightRed: "#ff8aab",
  brightGreen: "#bdf0cb",
  brightYellow: "#fbdba0",
  brightBlue: "#beb2f7",
  brightMagenta: "#e69ece",
  brightCyan: "#a6e8ee",
  brightWhite: "#feeeed",
  highlightGreen: "#9be2b0",
  highlightGet: THERBBT_CYAN,
  highlightUrl: THERBBT_PINK,
  highlightIpv6: "#9d8cf0",
  sidebarBackground: "#110920",
  folderIcon: THERBBT_CYAN,
  sshIndicator: THERBBT_PINK,
  defaultIndicator: THERBBT_CYAN,
  windowBackground: "#170c29",
  panelBackground: "#291236",
  // Hover is the theme's selection color; active and disabled sit one step
  // further along the same violet ramp, since colors.toml has no slot of
  // its own for them.
  hoverBackground: "#3a1f55",
  activeBackground: "#4a2a6b",
  textPrimary: "#fde3e3",
  textSecondary: "#a07fb5",
  textTertiary: "#6e5291",
  textDisabled: "#4a3566",
  statusConnecting: "#9d8cf0",
  statusDisconnected: "#6e5291",
  statusError: "#ff5c8a",
};
