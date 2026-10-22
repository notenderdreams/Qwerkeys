use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/app-icon.svg");
    println!("cargo:rerun-if-changed=assets/app-icon.png");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let png = Path::new(&manifest_dir).join("assets/app-icon.png");

    if !png.exists() {
        let script = Path::new(&manifest_dir).join("scripts/generate-icon.swift");
        let _ = Command::new("swift").arg(script).status();
    }
}
