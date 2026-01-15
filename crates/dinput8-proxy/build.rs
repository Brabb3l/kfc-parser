use std::path::Path;

fn main() {
    let target = std::env::var("TARGET").unwrap();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let def_path = Path::new(&manifest_dir)
        .join("def")
        .join("dinput8.def");
    let def_path = def_path
        .to_str()
        .expect("Failed to convert def path to string");

    match target.as_str() {
        "x86_64-pc-windows-gnu" => {
            println!("cargo:rustc-link-arg={def_path}");
        }
        "x86_64-pc-windows-msvc" => {
            println!("cargo:rustc-link-arg=/DEF:{def_path}");
        }
        _ => {
            panic!("Unsupported target: {target}");
        }
    }

    println!("cargo:rerun-if-changed={def_path}");
}
