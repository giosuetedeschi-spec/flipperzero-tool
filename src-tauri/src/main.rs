// Prevents additional console window on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod serial;
mod vfs;
mod parsers;

use tauri::Manager;

fn main() {
    env_logger::init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::serial_list_ports,
            commands::serial_connect,
            commands::serial_disconnect,
            commands::serial_read_file,
            commands::serial_write_file,
            commands::serial_list_dir,
            commands::fs_index_device,
            commands::fs_get_cached_tree,
            commands::parser_parse_sub,
            commands::parser_parse_ir,
            commands::parser_parse_nfc,
            commands::ufbt_new_project,
            commands::ufbt_compile,
            commands::ufbt_launch,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            // Initialize SQLite VFS cache
            vfs::init_cache(&app_handle)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
