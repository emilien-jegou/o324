use std::env;
use std::fs::create_dir_all;
use std::path::PathBuf;
use wayland_scanner::{generate_code, Side};

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let protocols_dir = PathBuf::from("./protocols");

    // Create the directory for the generated code if it doesn't exist
    let generated_dir = out_dir.join("wayland_protocols");
    create_dir_all(&generated_dir).unwrap();

    println!("cargo:rerun-if-changed=protocols");

    generate_code(
        protocols_dir.join("wlr-foreign-toplevel-management-unstable-v1.xml"),
        generated_dir.join("wlr_foreign_toplevel_management.rs"),
        Side::Client,
    );
}
