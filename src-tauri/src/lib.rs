pub mod errors;
pub mod commands;
pub mod serial;
pub mod ufbt;
pub mod vfs;
pub mod parsers;
pub mod reverse_engineer;
pub mod proto_bus;

// Re-export commonly used items
pub use commands::{list_directory, find_files, create_file_from_template, move_file};
pub use errors::AppError;
pub use parsers::{parse_sub, parse_ir, parse_nfc, ParsedFile};
pub use serial::{encode_varint, read_varint, base64_encode, parse_list_output};
pub use serial::{PortInfo, FlipperState, FlipperConnection};
pub use serial::{list_ports, connect, disconnect, is_connected};
pub use serial::{read_file_text, write_file_text, list_dir};
pub use serial::{delete_path, mkdir_path, stat_path, autodetect_connect, find_flipper};
pub use serial::{new_state};

pub use reverse_engineer::{analyze, reverse_engineer_analyze, reverse_engineer_analyze_file};

pub use proto_bus::{proto_list_dir, proto_read_file, proto_write_file, proto_mkdir, proto_delete, proto_device_info, proto_ping, rpc_command, RpcMessage, RpcContent};
