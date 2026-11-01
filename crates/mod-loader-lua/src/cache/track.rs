use std::{io::Read, path::Path};

use mod_loader::{Capability, ModManifest};
use walkdir::WalkDir;

use crate::{cache::FileStateCache, log::warn};

// The cache decides whether Lua patching has to run again.
//
// Runtime mods write into their own folder on launch (logs, a menu resaving
// its config). Creating a file also updates parent directory mtimes, so
// recording every path under mods/ by mtime misses on every launch.
//
// Record inputs that can change a patch. Export mods are included because a
// cache hit skips the whole run when runtime loading is off, and that run is
// also what produces exports. Other mods contribute their manifest and Lua
// sources: a patch mod can `require` those sources. Directories are not
// entries. Mod files are fingerprinted by content, so a rewrite with the
// same bytes does not invalidate the cache. Game .dat, .kfc, and .exe files
// stay on mtime; they are large and are not rewritten every launch.

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

enum ManifestTracking {
    /// No manifest. The loader does not run this directory.
    Skip,
    /// Patch or export mod, or a manifest we could not classify.
    Tree,
    /// Runtime-only (or otherwise not a patch/export mod).
    Sources,
}

impl FileStateCache {

    pub fn track_game_files(
        &mut self,
        game_dir: impl AsRef<Path>,
    ) {
        let game_dir = game_dir.as_ref();

        let game_files = match std::fs::read_dir(game_dir) {
            Ok(entries) => entries,
            Err(e) => {
                warn!(
                    error = %e,
                    path = game_dir.display().to_string(),
                    "Failed to read game directory"
                );
                return;
            }
        };

        for entry in game_files {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!(
                        error = %e,
                        path = game_dir.display().to_string(),
                        "Failed to read game directory entry, skipping"
                    );
                    continue;
                }
            };

            let path = entry.path();
            let extension = path.extension().and_then(|ext| ext.to_str());

            if let Some(ext) = extension {
                if ext == "dat" || ext == "kfc" || ext == "exe" {
                    self.track(path);
                }
            }
        }
    }

    pub fn track_mod_files(
        &mut self,
        mods_dir: impl AsRef<Path>,
    ) {
        let mods_dir = mods_dir.as_ref();

        let entries = match std::fs::read_dir(mods_dir) {
            Ok(entries) => entries,
            Err(e) => {
                warn!(
                    error = %e,
                    path = mods_dir.display().to_string(),
                    "Failed to read mods directory"
                );
                return;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    warn!(
                        error = %e,
                        path = mods_dir.display().to_string(),
                        "Failed to read mods directory entry, skipping"
                    );
                    continue;
                }
            };

            let path = entry.path();
            let file_name = path.file_name().unwrap_or_default();

            // The loader ignores hidden entries in the mods root.
            if file_name.to_str().is_some_and(|name| name.starts_with('.')) {
                continue;
            }

            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(e) => {
                    warn!(
                        error = %e,
                        path = path.display().to_string(),
                        "Failed to read mods directory entry type, skipping"
                    );
                    continue;
                }
            };

            if file_type.is_dir() {
                match manifest_tracking(&path.join("mod.json")) {
                    ManifestTracking::Skip => continue,
                    ManifestTracking::Tree => self.track_directory(&path),
                    ManifestTracking::Sources => self.track_sources(&path),
                }
            } else if file_type.is_file() && is_zip(&path) {
                // A zip is the whole mod. Runtime code cannot write a log
                // inside it without changing the archive bytes.
                self.track_content(&path);
            }
        }
    }

    pub fn track_directory(
        &mut self,
        dir: impl AsRef<Path>,
    ) {
        self.track_tree(dir.as_ref(), false);
    }

    fn track_sources(
        &mut self,
        dir: &Path,
    ) {
        self.track_tree(dir, true);
    }

    fn track_tree(
        &mut self,
        root: &Path,
        sources_only: bool,
    ) {
        let mod_files = WalkDir::new(root)
            .follow_links(true)
            .follow_root_links(true)
            .into_iter()
            .filter_entry(|entry| {
                let relative = entry.path().strip_prefix(root).unwrap_or(entry.path());

                !is_volatile(relative)
            });

        for mod_files in mod_files {
            let entry = match mod_files {
                Ok(entry) => entry,
                Err(e) => {
                    let path = e.path();

                    if let Some(ancestor) = e.loop_ancestor() {
                        warn!(
                            path = path.map_or_else(
                                || "unknown".into(),
                                |p| p.display().to_string()
                            ),
                            ancestor = ancestor.display().to_string(),
                            error = %e,
                            "Skipping directory loop"
                        );
                    } else {
                        warn!(
                            path = path.map_or_else(
                                || "unknown".into(),
                                |p| p.display().to_string()
                            ),
                            error = %e,
                            "Failed to read directory entry, skipping"
                        );
                    }

                    continue;
                }
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.into_path();
            let relative = path.strip_prefix(root).unwrap_or(&path);

            if sources_only && !is_source(relative) {
                continue;
            }

            self.track_content(&path);
        }
    }

    fn track_content(
        &mut self,
        path: &Path,
    ) {
        self.files.insert(path.to_path_buf(), content_hash(path));
    }

    #[cfg(test)]
    fn contains_path(
        &self,
        path: &Path,
    ) -> bool {
        self.files.contains_key(path)
    }

}

fn manifest_tracking(mod_json: &Path) -> ManifestTracking {
    let bytes = match std::fs::read(mod_json) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return ManifestTracking::Skip;
        }
        Err(_) => return ManifestTracking::Tree,
    };

    // Notepad saves a leading BOM. serde_json will not parse past it, and a
    // misclassified runtime mod would walk files that do not affect patching.
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);

    match serde_json::from_slice::<ModManifest>(bytes) {
        Ok(manifest) if tracks_output(&manifest) => ManifestTracking::Tree,
        Ok(_) => ManifestTracking::Sources,
        // Unreadable as a manifest. Walk the tree so a real input is not missed.
        Err(_) => ManifestTracking::Tree,
    }
}

fn tracks_output(manifest: &ModManifest) -> bool {
    manifest.capabilities.contains(&Capability::Patch)
        || manifest.capabilities.contains(&Capability::Export)
}

fn is_zip(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("zip")
}

fn is_source(relative: &Path) -> bool {
    if relative
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("mod.json"))
    {
        return true;
    }

    relative
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("lua"))
}

fn is_volatile(relative: &Path) -> bool {
    for component in relative.components() {
        let Some(name) = component.as_os_str().to_str() else {
            continue;
        };

        if is_volatile_name(name) {
            return true;
        }
    }

    false
}

fn is_volatile_name(name: &str) -> bool {
    if name.eq_ignore_ascii_case("logs")
        || name.eq_ignore_ascii_case("log")
        || name.eq_ignore_ascii_case("crashes")
        || name.eq_ignore_ascii_case(".git")
        || name.get(..6).is_some_and(|prefix| prefix.eq_ignore_ascii_case("crash-"))
    {
        return true;
    }

    let Some((_, ext)) = name.rsplit_once('.') else {
        return false;
    };

    const EXTENSIONS: &[&str] = &["log", "bak", "tmp", "old", "dmp", "vortex_backup"];

    EXTENSIONS.iter().any(|candidate| ext.eq_ignore_ascii_case(candidate))
}

fn content_hash(path: &Path) -> u64 {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return 0,
    };
    let mut file = std::io::BufReader::new(file);
    let mut hash = FNV_OFFSET;
    let mut len: u64 = 0;
    let mut buf = [0u8; 8192];

    loop {
        match file.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                len += n as u64;

                for byte in &buf[..n] {
                    hash ^= u64::from(*byte);
                    hash = hash.wrapping_mul(FNV_PRIME);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return 0,
        }
    }

    hash ^ len.rotate_left(32)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{Duration, SystemTime},
    };

    use super::*;

    static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "eml-cache-{}-{n}",
                std::process::id(),
            ));

            fs::create_dir_all(&path).unwrap();

            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write_file(path: &Path, bytes: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::write(path, bytes).unwrap();
    }

    fn manifest(capabilities: &[&str]) -> String {
        let capabilities = capabilities
            .iter()
            .map(|capability| format!("\"{capability}\""))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            r#"{{"id":"example","name":"Example","version":"1.0.0","capabilities":[{capabilities}]}}"#
        )
    }

    fn bump_mtime(path: &Path) {
        let file = fs::File::options().write(true).open(path).unwrap();
        let later = SystemTime::now() + Duration::from_secs(10);

        file.set_modified(later).unwrap();
    }

    fn track_mods(dir: &Path) -> FileStateCache {
        let mut cache = FileStateCache::default();

        cache.track_mod_files(dir);
        cache
    }

    fn assert_same(before: &FileStateCache, after: &FileStateCache) {
        let diff = before.diff(after);

        assert!(
            diff.is_none(),
            "expected no cache invalidation (build_id={}, files={})",
            diff.build_id_changed(),
            diff.files_changed(),
        );
    }

    fn assert_files_changed(before: &FileStateCache, after: &FileStateCache) {
        assert!(
            before.diff(after).files_changed(),
            "expected tracked files to change",
        );
    }

    #[test]
    fn runtime_mod_output_does_not_invalidate() {
        let dir = TempDir::new();
        let mod_dir = dir.path().join("creative");

        write_file(&mod_dir.join("mod.json"), manifest(&["runtime"]).as_bytes());
        write_file(&mod_dir.join("config.txt"), b"enabled=true\n");
        write_file(&mod_dir.join("src").join("lib.lua"), b"return {}\n");
        write_file(&mod_dir.join("creative.dll"), b"dll");

        let before = track_mods(dir.path());

        assert!(before.contains_path(&mod_dir.join("mod.json")));
        assert!(before.contains_path(&mod_dir.join("src").join("lib.lua")));
        assert!(!before.contains_path(&mod_dir.join("config.txt")));
        assert!(!before.contains_path(&mod_dir.join("creative.dll")));

        write_file(&mod_dir.join("logs").join("session.log"), b"started\n");
        write_file(&mod_dir.join("Logs").join("other.LOG"), b"again\n");
        write_file(&mod_dir.join("crash-1.dmp"), b"dump");
        write_file(&mod_dir.join("config.txt"), b"enabled=true\nlast=now\n");
        write_file(&mod_dir.join("creative.dll"), b"dll-rewritten");
        bump_mtime(&mod_dir.join("mod.json"));

        assert_same(&before, &track_mods(dir.path()));

        write_file(&mod_dir.join("mod.json"), manifest(&["patch"]).as_bytes());

        let patched = track_mods(dir.path());

        assert_files_changed(&before, &patched);
        assert!(patched.contains_path(&mod_dir.join("config.txt")));

        write_file(
            &mod_dir.join("src").join("lib.lua"),
            b"return { changed = true }\n",
        );

        assert_files_changed(&patched, &track_mods(dir.path()));
    }

    #[test]
    fn patch_mod_ignores_byproducts_and_identical_rewrites() {
        let dir = TempDir::new();
        let mod_dir = dir.path().join("auto-loot");

        write_file(
            &mod_dir.join("mod.json"),
            manifest(&["patch", "runtime"]).as_bytes(),
        );
        write_file(&mod_dir.join("src").join("mod.lua"), b"patch()\n");
        write_file(&mod_dir.join("config.txt"), b"radius=4\n");
        write_file(&mod_dir.join("data").join("table.json"), b"{}\n");

        let before = track_mods(dir.path());

        assert!(before.contains_path(&mod_dir.join("config.txt")));
        assert!(before.contains_path(&mod_dir.join("data").join("table.json")));

        write_file(&mod_dir.join("config.txt"), b"radius=4\n");
        bump_mtime(&mod_dir.join("config.txt"));
        bump_mtime(&mod_dir.join("src").join("mod.lua"));
        write_file(&mod_dir.join("logs").join("loot.log"), b"line\n");
        write_file(&mod_dir.join("notes.bak"), b"bak");
        write_file(&mod_dir.join("notes.tmp"), b"tmp");
        write_file(&mod_dir.join("notes.old"), b"old");
        write_file(&mod_dir.join("save.vortex_backup"), b"backup");
        write_file(&mod_dir.join("crashes").join("crash-2.dmp"), b"dump");

        let after = track_mods(dir.path());

        assert_same(&before, &after);
        assert!(!after.contains_path(&mod_dir.join("logs").join("loot.log")));
        assert!(!after.contains_path(&mod_dir));

        write_file(&mod_dir.join("config.txt"), b"radius=8\n");
        assert_files_changed(&before, &track_mods(dir.path()));

        write_file(&mod_dir.join("src").join("mod.lua"), b"patch_more()\n");
        assert_files_changed(&before, &track_mods(dir.path()));
    }

    #[test]
    fn export_mod_inputs_invalidate_and_removed_files_do_too() {
        let dir = TempDir::new();
        let mod_dir = dir.path().join("exporter");

        write_file(&mod_dir.join("mod.json"), manifest(&["export"]).as_bytes());
        write_file(&mod_dir.join("src").join("mod.lua"), b"export()\n");

        let before = track_mods(dir.path());

        write_file(&mod_dir.join("src").join("mod.lua"), b"export_more()\n");
        assert_files_changed(&before, &track_mods(dir.path()));

        let with_edit = track_mods(dir.path());

        fs::remove_file(mod_dir.join("src").join("mod.lua")).unwrap();
        assert_files_changed(&with_edit, &track_mods(dir.path()));
    }

    #[test]
    fn bom_and_unparsable_manifests() {
        let dir = TempDir::new();
        let runtime = dir.path().join("notepad-mod");
        let broken = dir.path().join("broken-mod");

        let mut bom_manifest = String::from("\u{feff}");
        bom_manifest.push_str(&manifest(&["runtime"]));
        write_file(&runtime.join("mod.json"), bom_manifest.as_bytes());
        write_file(&runtime.join("config.txt"), b"a\n");

        write_file(&broken.join("mod.json"), b"{not json");
        write_file(&broken.join("src").join("mod.lua"), b"patch()\n");
        write_file(&broken.join("table.json"), b"{}\n");

        let before = track_mods(dir.path());

        assert!(before.contains_path(&runtime.join("mod.json")));
        assert!(!before.contains_path(&runtime.join("config.txt")));
        assert!(before.contains_path(&broken.join("table.json")));

        write_file(&runtime.join("config.txt"), b"b\n");
        assert_same(&before, &track_mods(dir.path()));

        write_file(&broken.join("table.json"), b"{ \"n\": 1 }\n");
        assert_files_changed(&before, &track_mods(dir.path()));
    }

    #[test]
    fn ignored_root_entries_and_zip_bytes() {
        let dir = TempDir::new();

        write_file(&dir.path().join("loose.txt"), b"nope");
        write_file(&dir.path().join("not-a-mod").join("file.txt"), b"nope");
        write_file(
            &dir.path().join(".hidden").join("mod.json"),
            manifest(&["patch"]).as_bytes(),
        );
        write_file(&dir.path().join("example.zip"), b"zip-bytes");

        let before = track_mods(dir.path());

        assert!(before.contains_path(&dir.path().join("example.zip")));
        assert!(!before.contains_path(&dir.path().join("loose.txt")));
        assert!(!before.contains_path(&dir.path().join("not-a-mod").join("file.txt")));
        assert!(!before.contains_path(&dir.path().join(".hidden").join("mod.json")));

        write_file(&dir.path().join("loose.txt"), b"changed");
        write_file(&dir.path().join("example.zip"), b"zip-bytes");
        bump_mtime(&dir.path().join("example.zip"));
        assert_same(&before, &track_mods(dir.path()));

        write_file(&dir.path().join("example.zip"), b"zip-bytes-v2");
        assert_files_changed(&before, &track_mods(dir.path()));
    }

    #[test]
    fn game_files_still_use_mtime() {
        let dir = TempDir::new();

        write_file(&dir.path().join("enshrouded.dat"), b"data");
        write_file(&dir.path().join("enshrouded.kfc"), b"kfc");
        write_file(&dir.path().join("enshrouded.exe"), b"exe");
        write_file(&dir.path().join("readme.txt"), b"nope");

        let mut before = FileStateCache::default();
        before.track_game_files(dir.path());

        bump_mtime(&dir.path().join("readme.txt"));

        let mut unchanged = FileStateCache::default();
        unchanged.track_game_files(dir.path());
        assert_same(&before, &unchanged);

        bump_mtime(&dir.path().join("enshrouded.dat"));

        let mut changed = FileStateCache::default();
        changed.track_game_files(dir.path());
        assert_files_changed(&before, &changed);
    }
}
