use std::env;
use std::path::PathBuf;

use cmake::Config;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // build libpocketsphinx.a
    let dst = Config::new("pocketsphinx")
        .define("CMAKE_INSTALL_PREFIX", &out_path)
        .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=pocketsphinx");

    println!("cargo:rustc-link-search={}", out_path.display());
    println!("cargo:rerun-if-changed=wrapper.h");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}/include/include", dst.display()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
