fn main() {
    let dst = cmake::Config::new("source/src")
        .define("LIBAPI", "ON")
        .define("NOHQ", "ON")
        .build();

    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!(
        "cargo:rustc-link-search=native={}/build/osal",
        dst.display()
    );
    println!("cargo:rustc-link-lib=static=gliden64");
    println!("cargo:rustc-link-lib=static=osald");
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=GL");
    println!("cargo:rustc-link-lib=freetype");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
