extern crate bindgen;

#[cfg(not(windows))]
use pkg_config;
use std::env;
use std::path::PathBuf;
#[cfg(windows)]
use vcpkg;

#[cfg(windows)]
fn find_tesseract_system_lib() -> Vec<String> {
    let lib = vcpkg::Config::new().find_package("tesseract").unwrap();
    lib.include_paths
        .iter()
        .map(|x| x.to_string_lossy().into_owned())
        .collect::<Vec<String>>()
}

#[cfg(not(windows))]
fn find_tesseract_system_lib() -> Vec<String> {
    let pk = pkg_config::Config::new()
        .statik(cfg!(feature = "enable-static"))
        .probe("tesseract")
        .unwrap();
    // Tell cargo to tell rustc to link the system proj shared library.
    for path in pk.link_paths {
        println!("cargo:rustc-link-search=native={:?}", path);
    }
    for lib in pk.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    // C++
    #[cfg(all(target_os = "macos", feature = "enable-static"))]
    println!("cargo:rustc-link-lib=c++");

    #[cfg(all(target_os = "linux", feature = "enable-static"))]
    println!("cargo:rustc-link-lib=stdc++");

    // openmp (for GOMP_*, omp_*)
    #[cfg(all(target_os = "linux", feature = "enable-static"))]
    println!("cargo:rustc-link-lib=gomp");

    // for vDSP_dotpr/vDSP_dotprD
    #[cfg(all(target_os = "macos", feature = "enable-static"))]
    println!("cargo:rustc-link-lib=framework=Accelerate");

    let mut include_paths = pk.include_paths.clone();
    include_paths
        .iter_mut()
        .map(|x| {
            if !x.ends_with("include") {
                x.pop();
            }
            x
        })
        .map(|x| x.to_string_lossy())
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
}

fn main() {
    // Tell cargo to tell rustc to link the system tesseract
    // and leptonica shared libraries.
    let clang_extra_include = find_tesseract_system_lib();

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let mut capi_bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper_capi.h")
        .allowlist_function("^Tess.*")
        .blocklist_type("Boxa")
        .blocklist_type("Pix")
        .blocklist_type("Pixa")
        .blocklist_type("_IO_FILE")
        .blocklist_type("_IO_codecvt")
        .blocklist_type("_IO_marker")
        .blocklist_type("_IO_wide_data");

    for inc in &clang_extra_include {
        capi_bindings = capi_bindings.clang_arg(format!("-I{}", *inc));
    }

    // Finish the builder and generate the bindings.
    let capi_bindings = capi_bindings
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate capi bindings");

    let mut public_types_bindings = bindgen::Builder::default()
        .header("wrapper_public_types.hpp")
        .allowlist_var("^k.*")
        .blocklist_item("kPolyBlockNames");

    for inc in &clang_extra_include {
        public_types_bindings = public_types_bindings.clang_arg(format!("-I{}", *inc));
    }

    let public_types_bindings = public_types_bindings
        .clang_arg("-std=c++17")
        .generate()
        .expect("Unable to generate public types bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    capi_bindings
        .write_to_file(out_path.join("capi_bindings.rs"))
        .expect("Couldn't write capi bindings!");
    public_types_bindings
        .write_to_file(out_path.join("public_types_bindings.rs"))
        .expect("Couldn't write public types bindings!");
}
