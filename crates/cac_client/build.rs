fn main() {
    let crate_dir = std::env!("CARGO_MANIFEST_DIR");
    let mut config: cbindgen::Config = Default::default();
    config.language = cbindgen::Language::C;
    cbindgen::generate_with_config(crate_dir, config)
        .expect("Failed to generate bindings")
        .write_to_file("../../headers/libcac_client.h");

    csbindgen::Builder::default()
        .input_extern_file("src/interface.rs")
        .csharp_dll_name("libcac_client")
        .csharp_class_name("Client")
        .csharp_namespace("LibCacClient")
        .generate_csharp_file("../../clients/csharp/libs/LibCacClient.g.cs")
        .unwrap();
}
