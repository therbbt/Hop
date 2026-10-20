# Hop

A lightweight, cross-platform, multi-protocol connection manager and
terminal. One window, tabbed sessions over SSH, serial, and a local shell,
plus SFTP file transfer and view-only RDP — built in Rust (Tauri) and
Svelte, with a UI deliberately kept close to a plain terminal app rather
than a heavyweight IDE.

## What it does

- **Local shell** tabs backed by a real PTY (portable-pty), with automatic
  tab numbering that reuses freed slots and double-click-to-rename.
- **SSH** sessions (russh) with password or private-key authentication and
  trust-on-first-use host-key verification, tracked in a `known_hosts.json`
  sibling to the config file.
- **Serial** connections (serialport) with auto-detected available ports
  and common baud-rate presets.
- **SFTP** file browsing and transfer over an existing SSH connection —
  list, upload/download, delete, create/remove directories — in a side
  panel next to the terminal.
- **RDP** (IronRDP) — connects and renders the remote framebuffer; currently
  view-only, no keyboard/mouse input forwarding yet.
- **Saved sessions**: save a connection's address/credentials from any
  connect dialog, reconnect with one click from the sidebar, and edit a
  saved session's details later without retyping its password (only its
  handle is kept in the OS keychain — password/passphrase can be left blank
  on edit to keep the stored value).
- **Settings** for the terminal's font family, size, and per-machine ANSI
  color palette — applied live to every open terminal, not just new ones.
- A window, sidebar, and unified action bar styled to match FlashPad, a
  sibling app, including the resizable sidebar, transparent rounded window
  chrome, and matching accent color.

Not yet implemented: telnet (scaffolded crate, no working session yet),
RDP input forwarding, and host groups/folders in the sidebar.

## Layout

```
crates/hop-core    the Session trait, event/command types, config, keychain
crates/hop-shell   portable-pty: local shell Session
crates/hop-ssh     russh: SSH Session + host-key (TOFU) verification
crates/hop-serial  serialport: serial Session
crates/hop-sftp    SFTP over an SSH session
crates/hop-rdp     ironrdp: view-only RDP client
crates/hop-telnet  telnet Session (scaffolded, not yet implemented)
app/hop            Tauri app: commands, event bridge, window
frontend/          Svelte + xterm.js
```

The app crate only ever depends on `hop-core`'s `Session` trait — never on
a protocol crate's internals directly. Every protocol lives behind that
trait in its own crate, and the on-disk config is a single hand-editable
`config.json`; secrets are never written to it, only opaque keychain handles
are.

## Arch / Hyprland (e.g. Omarchy)

There's no Arch package yet, so build and install for your user with:

```bash
sudo pacman -S --needed base-devel webkit2gtk-4.1 gtk3 librsvg libayatana-appindicator gnome-keyring
./scripts/install-linux-local.sh   # installs to ~/.local; re-run to update
```

The in-app updater can't update this install (on Linux it only replaces
AppImages), so re-run the script to update.

Under Hyprland, Hop runs natively on Wayland, lets the compositor
manage the window, and switches to a keyboard-first layout: no title bar,
buttons or sidebar, just the terminal and a slim tab line once two or more
tabs are open. Everything is a hotkey (`Ctrl+Shift+H` lists them), and
`Ctrl+Shift+P` opens a command palette that searches saved sessions, open
tabs and every action, including managing sessions and folders. Settings →
Terminal → Keyboard-first layout turns it on or off on any desktop. A tiled
window drops its rounded corners and shadow. Hyprland doesn't let apps grab global
hotkeys, so the built-in `Alt+T` doesn't work there; bind the hotkey in your
Hyprland config to run `hop --toggle` instead, which shows or hides the
running instance. Launching `hop` again while it's running
brings the existing window forward. On Omarchy:

```lua
-- ~/.config/hypr/bindings.lua
o.bind("ALT + T", "Hop", "~/.local/bin/hop --toggle")
```

## Development

Requires Rust, Node, and (on Linux) `webkit2gtk-4.1` plus a Secret
Service provider (GNOME Keyring, KWallet) for the keychain backend.

```sh
npm install --prefix frontend
npm run tauri --prefix frontend -- dev
```

(`tauri.conf.json` lives in `app/hop`, not next to `frontend/package.json`,
so the `tauri` script `cd`s there before invoking the CLI — that's also why
a bare `npx tauri dev` from `frontend/` won't find the config.)

`cargo check --workspace` and `cd frontend && npx svelte-check` cover the
Rust and frontend sides independently. Each protocol crate also has its own
integration tests that spin up a real counterpart (a local sshd, a virtual
serial pair, etc.) rather than mocking the protocol.
  
