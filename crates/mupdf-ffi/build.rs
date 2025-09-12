use std::env;
use std::path::PathBuf;

fn main() {
    // Manually link mupdf and its dependencies.
    // The order can be important for the linker.
    println!("cargo:rustc-link-lib=mupdf");
    println!("cargo:rustc-link-lib=mupdf-third");
    println!("cargo:rustc-link-lib=freetype");
    println!("cargo:rustc-link-lib=jpeg");
    println!("cargo:rustc-link-lib=openjp2");
    println!("cargo:rustc-link-lib=jbig2dec");
    println!("cargo:rustc-link-lib=mujs");
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=m");

    // Find mupdf with pkg-config to get include paths, but don't print linker flags
    // as we are doing that manually.
    let library = pkg_config::Config::new()
        .cargo_metadata(false)
        .probe("mupdf")
        .expect("Failed to find mupdf. Have you installed the mupdf development package?");

    println!("cargo:rerun-if-changed=wrapper.h");

    // Build the bindgen arguments.
    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    // Add include paths from pkg-config.
    for path in &library.include_paths {
        builder = builder.clang_arg(format!("-I{}", path.to_string_lossy()));
    }

    // Generate the bindings.
    let bindings = builder
        .blocklist_item("FP_NAN")
        .blocklist_item("FP_INFINITE")
        .blocklist_item("FP_ZERO")
        .blocklist_item("FP_SUBNORMAL")
        .blocklist_item("FP_NORMAL")
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
