fn main() {
    tauri_build::build();

    // Generate Rust types from .proto files
    let proto_dir = std::path::Path::new("proto");
    if proto_dir.exists() {
        let protos: Vec<_> = proto_dir
            .read_dir()
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "proto"))
            .map(|e| e.path())
            .collect();

        if !protos.is_empty() {
            let out_dir = std::path::Path::new("src/proto_gen");
            std::fs::create_dir_all(out_dir).unwrap();

            prost_build::Config::new()
                .out_dir(out_dir)
                .compile_protos(&protos, &[proto_dir])
                .unwrap();

            // Tell cargo to rerun if proto files change
            for proto in &protos {
                println!("cargo:rerun-if-changed={}", proto.display());
            }
        }
    }
}
