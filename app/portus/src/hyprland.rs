//! Hyprland integration. Hyprland tiles windows, never draws title bars,
//! and ignores a client's own maximize request (Omarchy suppresses it for
//! every window by default), so under it the window's state is read from and
//! changed through the compositor itself via `hyprctl`. Same approach as
//! FlashPad's Hyprland support.

use std::process::Command;

use serde::Serialize;

pub fn on_hyprland() -> bool {
    cfg!(target_os = "linux") && std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some()
}

/// Our own window's entry in `hyprctl clients -j`, matched by process id.
fn own_client() -> Option<serde_json::Value> {
    let output = Command::new("hyprctl").args(["clients", "-j"]).output().ok()?;
    let clients: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).ok()?;
    let pid = std::process::id() as u64;
    clients.into_iter().find(|c| c["pid"].as_u64() == Some(pid))
}

/// Whether a keyboard-driven tiling compositor is managing the window, where
/// the title bar's minimize/maximize/close buttons are out of place - the
/// frontend hides them there.
#[tauri::command]
pub fn uses_tiling_wm() -> bool {
    on_hyprland()
}

#[derive(Serialize)]
pub struct TilingWindowState {
    floating: bool,
    fullscreen: bool,
}

/// How Hyprland is currently placing the window - the frontend squares the
/// shell's corners off unless it's floating, since a tiled window sits flush
/// against its neighbours. `None` when not on Hyprland or the query fails.
/// GTK's own is_maximized() can't be used here: it reports true for a plain
/// floating window under Hyprland.
#[tauri::command]
pub async fn tiling_window_state() -> Option<TilingWindowState> {
    if !on_hyprland() {
        return None;
    }
    let client = own_client()?;
    Some(TilingWindowState {
        floating: client["floating"].as_bool().unwrap_or(false),
        fullscreen: client["fullscreen"].as_u64().unwrap_or(0) != 0,
    })
}

/// Maximize toggle that works under Hyprland too. There it asks the
/// compositor directly, acting on the active window - always Portus, since
/// this only runs from a click on its own title bar.
#[tauri::command]
pub async fn toggle_window_maximized(window: tauri::WebviewWindow) -> Result<(), String> {
    if on_hyprland() {
        let lua = Command::new("hyprctl")
            .args(["dispatch", r#"hl.dsp.window.fullscreen({ mode = "maximized" })"#])
            .output()
            .map_err(|e| e.to_string())?;
        // Hyprland before its Lua config (0.55) only understands the old
        // dispatcher syntax.
        if String::from_utf8_lossy(&lua.stdout).trim() != "ok" {
            Command::new("hyprctl").args(["dispatch", "fullscreen", "1"]).output().map_err(|e| e.to_string())?;
        }
        return Ok(());
    }
    if window.is_maximized().map_err(|e| e.to_string())? {
        window.unmaximize().map_err(|e| e.to_string())
    } else {
        window.maximize().map_err(|e| e.to_string())
    }
}
