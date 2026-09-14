fn main() {
    println!("cargo:rerun-if-changed=interface/CoreService.h");

    let dst = cmake::Config::new(".")
        .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=lyra");
    println!("cargo:rerun-if-changed=interface/CoreService.h");

    let header_path = dst.join("interface").join("CoreService.h");

    let bindings = bindgen::Builder::default()
        .header(header_path.to_str().unwrap())
        .wrap_unsafe_ops(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("failed to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("failed to write bindings");
}
