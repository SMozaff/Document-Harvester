//! Place the host-platform PDFium library next to the built executable.
//!
//! The library is committed under `vendor/pdfium`; this build script only
//! copies bytes locally. Nothing is downloaded at build or run time. The app
//! resolves the library from its own executable directory via
//! `dh_part1::renderer::bundled_library_path()`.

use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let Some(out_dir) = env::var_os("OUT_DIR") else {
        return;
    };
    let out_dir = PathBuf::from(out_dir);
    // cargo: OUT_DIR is below <target>/<profile>/build/...; find that profile
    // directory rather than assuming a fixed number of path components.
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_owned());
    let profile_dir = out_dir
        .ancestors()
        .find(|path| path.file_name().and_then(|name| name.to_str()) == Some(profile.as_str()))
        .unwrap_or(&out_dir);

    let (platform_directory, library_name) = if cfg!(target_os = "windows") {
        ("win-x64", "pdfium.dll")
    } else if cfg!(target_os = "macos") {
        ("libpdfium.dylib", "libpdfium.dylib")
    } else {
        ("linux-x64", "libpdfium.so")
    };
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/pdfium")
        .join(platform_directory)
        .join(library_name);
    if !source.is_file() {
        println!(
            "cargo:warning=no vendored PDFium library for this platform at {}",
            source.display()
        );
        return;
    }
    println!("cargo:rerun-if-changed={}", source.display());
    for destination in [out_dir.join(library_name), profile_dir.join(library_name)] {
        fs::copy(&source, &destination).unwrap_or_else(|error| {
            panic!(
                "cannot copy bundled PDFium '{}' to '{}': {error}",
                source.display(),
                destination.display()
            )
        });
    }
}
