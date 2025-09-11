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

    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I/usr/include/mupdf")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
