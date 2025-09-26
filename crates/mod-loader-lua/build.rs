use std::fs;
use std::env;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

fn main() -> std::io::Result<()> {
    generate_definitions_file()?;

    // does not change unless definitions change, so this may need to be changed in the future
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    println!("cargo:rustc-env=BUILD_ID={now}");

    Ok(())
}

fn generate_definitions_file() -> std::io::Result<()> {
    let definitions_dir = Path::new("definitions");

    println!("cargo:rerun-if-changed={}", definitions_dir.display());

    let mut lua_files = Vec::new();

    collect_lua_files(definitions_dir, &mut lua_files)?;

    lua_files.sort();

    for file in &lua_files {
        println!("cargo:rerun-if-changed={}", file.display());
    }

    let mut combined = String::new();

    combined.push_str("--- @meta\n");

    for file in lua_files {
        let content = fs::read_to_string(&file)?;

        if let Some(pos) = content.find('\n') {
            if content[..pos].trim().contains("@meta") {
                combined.push_str(&content[pos + 1..]);
            } else {
                combined.push_str(&content);
            }
        } else {
            combined.push_str(&content);
        }
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("definitions.lua");

    let need_write = match fs::read_to_string(&out_path) {
        Ok(existing) => existing != combined,
        Err(_) => true,
    };

    if need_write {
        std::fs::write(&out_path, combined.as_bytes())?;
    }

    Ok(())
}

fn collect_lua_files(
    dir: &Path,
    out: &mut Vec<PathBuf>
) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let p = entry?.path();

        if p.is_dir() {
            collect_lua_files(&p, out)?;
        } else if p.extension().and_then(|s| s.to_str()) == Some("lua") {
            out.push(p);
        }
    }
    Ok(())
}
