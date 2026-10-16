#!/usr/bin/env bash
# Builds Portus from source and installs it for the current user only
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

install -Dm755 target/release/portus "$prefix/bin/portus"
install -Dm644 app/portus/icons/128x128.png "$prefix/share/icons/hicolor/128x128/apps/portus.png"
install -Dm644 app/portus/icons/portus-icon.svg "$prefix/share/icons/hicolor/scalable/apps/portus.svg"
install -Dm644 /dev/stdin "$prefix/share/applications/portus.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Portus
Comment=SSH, serial, shell and RDP connection manager
Exec=$prefix/bin/portus
Icon=portus
Terminal=false
Categories=System;TerminalEmulator;Network;
StartupWMClass=portus
DESKTOP

echo "Installed to $prefix/bin/portus"
echo "On Hyprland, bind your hotkey to: $prefix/bin/portus --toggle"
