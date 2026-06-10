// Prevents additional console window on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod errors;
mod serial;
mod vfs;
mod parsers;


fn main() {
    env_logger::init();

    tauri::Builder::default()
        .manage(super::serial::new_state())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Local filesystem
            commands::list_directory,
            commands::move_file,
            commands::find_files,
            commands::create_file_from_template,
            commands::local_read_file,
            commands::local_write_file,
            // Serial (async)
            commands::serial_list_ports,
            commands::serial_connect,
            commands::serial_disconnect,
            commands::serial_read_file,
            commands::serial_write_file,
            commands::serial_list_dir,
            commands::serial_is_connected,
            // VFS
            commands::fs_index_device,
            commands::fs_get_cached_tree,
            // Parsers
            commands::parser_parse_sub,
            commands::parser_parse_ir,
            commands::parser_parse_nfc,
            // uFBT
            commands::ufbt_new_project,
            commands::ufbt_compile,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            vfs::init_cache(&app_handle).map_err(|e| {
                eprintln!("VFS init error: {}", e);
                e
            })?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("Fatal error: {}", e);
            std::process::exit(1);
        });
}
