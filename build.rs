use std::process::Command;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo::rustc-link-search={manifest_dir}/lib");
    println!("cargo::rustc-link-lib=mapdata");
    println!("cargo::rustc-link-arg=-Wl,-rpath,{manifest_dir}/lib");

    let commit = match Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        Ok(output) => String::from_utf8(output.stdout).unwrap(),
        Err(_) => String::from("unknown"),
    };

    println!("cargo::rustc-env=GIT_HASH={commit}");
}
