mod adapter;
pub mod commands;
mod hyprland;
mod rdp_state;
mod sftp_state;
mod tray;

use adapter::AppState;
use rdp_state::RdpState;
use sftp_state::SftpState;

pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        // Must be registered first, so a repeat launch hands its args to the
        // running instance and exits before touching anything else.
        // `portus --toggle` shows/hides the window: on Hyprland the global
        // hotkey never sees keys pressed in other Wayland windows, so a
        // compositor keybinding runs that instead. Any other repeat launch
        // just brings the window forward.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if args.iter().any(|arg| arg == tray::TOGGLE_FLAG) {
                tray::toggle_main_window(app);
            } else {
                tray::show_main_window(app);
            }
        }))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tray::global_shortcut_plugin())
        .manage(AppState::default())
        .manage(SftpState::default())
        .manage(RdpState::default())
        .invoke_handler(tauri::generate_handler![
            commands::session_open,
            commands::session_write,
            commands::session_resize,
            commands::session_close,
            commands::ssh_trust_host_key,
            commands::get_config,
            commands::save_config,
            commands::list_serial_ports,
            commands::list_wsl_distros,
            commands::save_session,
            commands::delete_session,
            commands::resolve_session_secret,
            commands::save_group,
            commands::delete_group,
            commands::set_group_collapsed,
            commands::set_group_slug,
            commands::reorder_session,
            commands::reorder_group,
            commands::sftp_connect,
            commands::sftp_list,
            commands::sftp_read_file,
            commands::sftp_download_file,
            commands::sftp_download_dir,
            commands::sftp_upload_file,
            commands::sftp_upload_dir,
            commands::sftp_remove_file,
            commands::sftp_create_dir,
            commands::sftp_remove_dir,
            commands::sftp_disconnect,
            commands::rdp_connect,
            commands::rdp_disconnect,
            commands::rdp_trust_host_key,
            commands::platform_name,
            commands::is_wayland_session,
            hyprland::uses_tiling_wm,
            hyprland::tiling_window_state,
            hyprland::toggle_window_maximized,
        ])
        .setup(|app| {
            tray::setup(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running portus");
}
