// Working-directory helpers for folder actions (see FolderAction in
// bridge.ts): reading a pane's directory out of what its shell prints, and
// matching it against an action's paths.

/** The path from an OSC 7 report (`file://host/path`, percent-encoded),
 * the standard way a shell announces its directory to the terminal. */
export function parseOsc7(data: string): string | null {
  const match = /^file:\/\/[^/]*(\/.*)$/.exec(data.trim());
  if (!match) return null;
  try {
    return decodeURIComponent(match[1]);
  } catch {
    return match[1];
  }
}

/** The directory in a `user@host:dir` window title - what most distros'
 * default bash prompt sets (Arch writes `user@host:~/dir`, Debian/Ubuntu
 * `user@host: ~/dir`). The fallback for SSH, where Hop can't look at the
 * remote shell's process. */
export function cwdFromTitle(title: string): string | null {
  const match = /^[^\s@]+@[^\s:]+:\s?([~/].*)$/.exec(title.trim());
  return match ? match[1] : null;
}

/** Expands a leading `~` against `home`; left as is without one. */
export function expandHome(path: string, home: string | null): string {
  if (!home || !path.startsWith("~")) return path;
  if (path === "~") return trimSlash(home);
  if (path.startsWith("~/")) return trimSlash(home) + path.slice(1);
  return path;
}

/** The reverse of expandHome, for showing a path the short way. */
export function collapseHome(path: string, home: string | null): string {
  if (!home) return path;
  const base = trimSlash(home);
  if (path === base) return "~";
  if (path.startsWith(base + "/")) return "~" + path.slice(base.length);
  return path;
}

/** Whether `cwd` is `base` itself or somewhere inside it. */
export function pathWithin(cwd: string, base: string): boolean {
  const c = trimSlash(cwd);
  const b = trimSlash(base);
  if (!b) return false;
  return c === b || c.startsWith(b === "/" ? "/" : b + "/");
}

function trimSlash(path: string): string {
  const trimmed = path.trim();
  return trimmed.length > 1 ? trimmed.replace(/\/+$/, "") : trimmed;
}
