use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

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
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=c++");
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=stdc++");

    // Generate bindings
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

    // Rebuild triggers
    println!("cargo:rerun-if-changed=shim/otio_shim.h");
    println!("cargo:rerun-if-changed=shim/otio_shim.cpp");
    println!("cargo:rerun-if-changed=shim/CMakeLists.txt");
}
