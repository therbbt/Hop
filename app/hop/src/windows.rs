//! Extra windows for the keyboard-first layout. There Hop stands in for a
//! terminal, so a repeat `hop` launch (a compositor keybinding like
//! Omarchy's Super+Return) opens another window in the running instance,
//! the way a terminal does, instead of bringing the one window forward.

use std::sync::atomic::{AtomicUsize, Ordering};

use tauri::{AppHandle, WebviewWindowBuilder};

use crate::hyprland;

/// Number for the next extra window - "main" counts as the first.
static NEXT_WINDOW: AtomicUsize = AtomicUsize::new(2);

/// Whether the keyboard-first layout is on - same resolution as the
/// frontend's (App.svelte): the saved setting, or automatic, which means on
/// under a tiling compositor. Read fresh each time, so a change in Settings
/// applies to the next launch without a restart.
pub fn keyboard_ui() -> bool {
    hop_core::config::load()
        .ok()
        .and_then(|config| config.settings.keyboard_ui)
        .unwrap_or_else(hyprland::on_hyprland)
}

/// Opens another window, built from the main window's config in
/// tauri.conf.json. Its title doubles as its identity in `hyprctl clients`
/// (see hyprland.rs), so each one gets its own number. Unlike the main
/// window, closing it really closes it rather than hiding it to the tray.
pub fn open_window(app: &AppHandle) -> tauri::Result<()> {
    let Some(mut config) = app.config().app.windows.iter().find(|w| w.label == "main").cloned() else {
        return Ok(());
    };
    let n = NEXT_WINDOW.fetch_add(1, Ordering::SeqCst);
    config.label = format!("hop-{n}");
    config.title = format!("Hop {n}");
    let window = WebviewWindowBuilder::from_config(app, &config)?.build()?;
    hyprland::release_min_size(&window)
}
