fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo::rustc-link-search={manifest_dir}/lib");
    println!("cargo::rustc-link-lib=mapdata");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{manifest_dir}/lib");
}
