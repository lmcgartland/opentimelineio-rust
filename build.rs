use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    #[cfg(feature = "vendored")]
    build_vendored(&out_dir, &manifest_dir);

    #[cfg(feature = "system")]
    build_system(&out_dir, &manifest_dir);

    // Generate bindings (same for both modes)
    generate_bindings(&out_dir, &manifest_dir);

    // Rebuild triggers
    println!("cargo:rerun-if-changed=shim/otio_shim.h");
    println!("cargo:rerun-if-changed=shim/otio_shim.cpp");
    println!("cargo:rerun-if-changed=shim/CMakeLists.txt");
}

#[cfg(feature = "vendored")]
fn build_vendored(out_dir: &Path, manifest_dir: &Path) {
    // Build OTIO + shim via CMake
    let dst = cmake::Config::new(manifest_dir.join("shim"))
        .define("CMAKE_BUILD_TYPE", "Release")
        .build();

    // Link paths
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-search=native={}/lib64", dst.display());

    // Link libraries
    println!("cargo:rustc-link-lib=static=otio_shim");
    println!("cargo:rustc-link-lib=static=opentimelineio");
    println!("cargo:rustc-link-lib=static=opentime");

    // C++ stdlib (platform-specific)
    link_cpp_stdlib();

    let _ = out_dir; // suppress unused warning
}

#[cfg(feature = "system")]
fn build_system(out_dir: &Path, manifest_dir: &Path) {
    // Find system OpenTimelineIO via pkg-config
    let otio = pkg_config::Config::new()
        .atleast_version("0.15")
        .probe("opentimelineio")
        .expect(
            "OpenTimelineIO not found via pkg-config. \
             Install OpenTimelineIO system-wide or use the 'vendored' feature instead.",
        );

    // Build just the shim against system OTIO
    let mut cmake_config = cmake::Config::new(manifest_dir.join("shim"));
    cmake_config.define("CMAKE_BUILD_TYPE", "Release");
    cmake_config.define("USE_SYSTEM_OTIO", "ON");

    // Pass OTIO include paths to CMake
    let include_paths: Vec<_> = otio.include_paths.iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    if !include_paths.is_empty() {
        cmake_config.define("OTIO_INCLUDE_DIRS", include_paths.join(";"));
    }

    // Pass OTIO library paths to CMake
    let lib_paths: Vec<_> = otio.link_paths.iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    if !lib_paths.is_empty() {
        cmake_config.define("OTIO_LIB_DIRS", lib_paths.join(";"));
    }

    let dst = cmake_config.build();

    // Link paths
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-search=native={}/lib64", dst.display());

    // Link shim (static)
    println!("cargo:rustc-link-lib=static=otio_shim");

    // Link OTIO libraries (already configured by pkg-config, but we may need dynamic)
    for lib in &otio.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    // C++ stdlib (platform-specific)
    link_cpp_stdlib();

    let _ = out_dir; // suppress unused warning
}

fn link_cpp_stdlib() {
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=c++");
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=stdc++");
}

fn generate_bindings(out_dir: &Path, manifest_dir: &Path) {
    let bindings = bindgen::Builder::default()
        .header(manifest_dir.join("shim/otio_shim.h").to_string_lossy())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("otio_.*")
        .allowlist_type("Otio.*")
        .generate()
        .expect("Failed to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Failed to write bindings");
}
