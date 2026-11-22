//! Builds the page: compiles the crate for `wasm32-unknown-unknown` (release) and runs
//! `wasm-bindgen --target web` into `tools/web-preview/www/pkg/` (generated, never committed).
//! Needs the wasm32 target and `wasm-bindgen-cli` matching the `wasm-bindgen` crate version.
#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::process::ExitCode {
    use std::path::PathBuf;
    use std::process::{Command, ExitCode};
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = crate_dir.join("../..");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let build = Command::new(cargo)
        .current_dir(&root)
        .args([
            "build",
            "-p",
            "web-preview",
            "--lib",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .status();
    if !matches!(build, Ok(s) if s.success()) {
        eprintln!("web-preview-build: cargo build for wasm32 failed ({build:?})");
        return ExitCode::FAILURE;
    }
    let target_dir = std::env::var("CARGO_TARGET_DIR").map_or_else(|_| root.join("target"), PathBuf::from);
    let wasm = target_dir.join("wasm32-unknown-unknown/release/web_preview.wasm");
    let out = crate_dir.join("www/pkg");
    let bindgen = Command::new("wasm-bindgen")
        .arg("--target")
        .arg("web")
        .arg("--no-typescript")
        .arg("--out-dir")
        .arg(&out)
        .arg(&wasm)
        .status();
    if !matches!(bindgen, Ok(s) if s.success()) {
        eprintln!("web-preview-build: wasm-bindgen failed ({bindgen:?}); install it with `cargo install wasm-bindgen-cli --version <the wasm-bindgen version in Cargo.lock>`");
        return ExitCode::FAILURE;
    }
    println!("web-preview built into {}", out.display());
    ExitCode::SUCCESS
}

#[cfg(target_arch = "wasm32")]
fn main() {}
