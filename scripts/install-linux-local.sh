#!/usr/bin/env bash
# Builds Hop from source and installs it for the current user only
# (~/.local), for distros the release .deb/.rpm/AppImage don't cover well -
# e.g. Arch-based ones like Omarchy. Re-run it to update: the in-app updater
# only knows how to replace AppImages on Linux, so it can't update this
# install.
#
# Needs: base-devel, webkit2gtk-4.1, gtk3, librsvg, libayatana-appindicator,
# a Secret Service provider (gnome-keyring or KWallet), plus Node.js and a
# Rust toolchain.
set -euo pipefail

repo="$(cd "$(dirname "$0")/.." && pwd)"
prefix="${PREFIX:-$HOME/.local}"

cd "$repo"
npm install --prefix frontend
npm run tauri --prefix frontend -- build --no-bundle

install -Dm755 target/release/hop "$prefix/bin/hop"
install -Dm644 app/hop/icons/128x128.png "$prefix/share/icons/hicolor/128x128/apps/hop.png"
install -Dm644 app/hop/icons/hop-icon.svg "$prefix/share/icons/hicolor/scalable/apps/hop.svg"
install -Dm644 /dev/stdin "$prefix/share/applications/hop.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Hop
Comment=SSH, serial, shell and RDP connection manager
Exec=$prefix/bin/hop
Icon=hop
Terminal=false
Categories=System;TerminalEmulator;Network;
StartupWMClass=hop
DESKTOP

echo "Installed to $prefix/bin/hop"
echo "On Hyprland, bind your hotkey to: $prefix/bin/hop --toggle"
