pub mod commands;
pub mod errors;
pub mod parsers;
pub mod proto_bus;
pub mod reverse_engineer;
pub mod rpc;
pub mod serial;
pub mod transport;
pub mod ufbt;
pub mod vfs;

// Re-export commonly used items
pub use commands::{copy_file, create_file_from_template, delete_file, find_files};
pub use commands::{get_file_content, list_directory, move_file, rename_file, write_file_content};
pub use errors::AppError;
pub use parsers::{ParsedFile, parse_ir, parse_nfc, parse_sub};
pub use serial::new_state;
pub use serial::{FlipperConnection, FlipperState, PortInfo};
pub use serial::{autodetect_connect, delete_path, find_flipper, mkdir_path, stat_path};
pub use serial::{base64_decode, base64_encode, encode_varint, parse_list_output, read_varint};
pub use serial::{connect, disconnect, is_connected, list_ports};
pub use serial::{list_dir, read_file_text, write_file_text};

pub use reverse_engineer::{analyze, reverse_engineer_analyze, reverse_engineer_analyze_file};

pub use proto_bus::{
    RpcContent, RpcMessage, proto_delete, proto_device_info, proto_list_dir, proto_mkdir,
    proto_ping, proto_read_file, proto_write_file, rpc_command,
};

/// Builds and runs the Tauri application. The sole entry point used by
/// `main.rs`, so the binary and the `flipperzero_tool_lib` crate never
/// drift into two different module/handler trees again.
///
/// On iOS/Android there is no `main.rs` to call this: the platform launches the
/// app through the generated native shell, which is what `mobile_entry_point`
/// wires up.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .manage(serial::new_state())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Local filesystem
            commands::list_directory,
            commands::move_file,
            commands::find_files,
            commands::create_file_from_template,
            commands::local_read_file,
            commands::local_write_file,
            commands::rename_file,
            commands::delete_file,
            commands::copy_file,
            commands::get_file_content,
            commands::write_file_content,
            commands::get_app_paths,
            commands::open_in_system,
            // Serial (async)
            commands::serial_list_ports,
            commands::serial_connect,
            commands::serial_disconnect,
            commands::serial_read_file,
            commands::serial_write_file,
            commands::serial_list_dir,
            commands::serial_is_connected,
            commands::serial_delete,
            commands::serial_mkdir,
            commands::serial_stat,
            commands::serial_autodetect_connect,
            commands::serial_upload,
            commands::serial_download,
            // VFS
            commands::fs_index_device,
            commands::fs_get_cached_tree,
            commands::fs_cache_file,
            commands::fs_get_cached_file,
            commands::fs_insert_file,
            commands::fs_remove_file,
            commands::fs_clear_cache,
            // Parsers
            commands::parser_parse_sub,
            commands::parser_parse_ir,
            commands::parser_parse_nfc,
            // Typed parser variants, called by the Sub-GHz/IR/NFC detail views
            commands::parser_parse_sub_struct,
            commands::parser_parse_ir_struct,
            commands::parser_parse_nfc_struct,
            // Templates
            commands::template_get,
            commands::template_list,
            commands::template_create,
            // uFBT
            commands::ufbt_new_project,
            commands::ufbt_compile,
            commands::ufbt_is_installed,
            commands::ufbt_get_version,
            commands::ufbt_get_sdk_version,
            commands::ufbt_install,
            commands::ufbt_update,
            commands::ufbt_create,
            commands::ufbt_build,
            commands::ufbt_deploy,
            commands::ufbt_clean,
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
        // `process::exit` skips destructors and is not a legal way to leave a
        // mobile app: iOS treats a self-terminating process as a crash. Panic
        // instead, which unwinds and surfaces the real cause on every platform.
        .unwrap_or_else(|e| panic!("Fatal error while running the application: {}", e));
}
