//! Offline reproduction of `unpack --stdout | repack --stdin` against copies.
//! Requires GAME_DIR and optionally GAME_FILE_NAME=enshrouded_server.
//! This verifies package data, not whether the game accepts the repacked files.

use std::{
    collections::BTreeSet,
    env,
    fs::{self, File},
    io,
    path::{Component, Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, ensure};
use kfc::container::KFCReader;

struct RunningChild {
    child: Child,
    reaped: bool,
}

impl RunningChild {
    fn wait(&mut self) -> io::Result<ExitStatus> {
        let status = self.child.wait()?;
        self.reaped = true;
        Ok(status)
    }
}

impl Drop for RunningChild {
    fn drop(&mut self) {
        if !self.reaped {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn cli(executable: &Path, fixture: &Path, operation: &str, game_dir: &Path, name: &str) -> Command {
    let mut command = Command::new(executable);
    command
        .current_dir(fixture)
        .args(["--threads", "4", operation, "-g"])
        .arg(game_dir)
        .args(["--file-name", name]);
    command
}

fn run_pipeline(executable: &Path, fixture: &Path, game_dir: &Path, name: &str) -> Result<()> {
    // Only the resource JSON stream uses a pipe. Diagnostic streams go directly
    // to files, so waiting for either child cannot fill an unread log pipe.
    let mut unpack = RunningChild {
        child: cli(executable, fixture, "unpack", game_dir, name)
            .arg("--stdout")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(File::create(fixture.join("unpack.stderr.log"))?)
            .spawn()
            .context("starting unpack")?,
        reaped: false,
    };
    let stream = unpack
        .child
        .stdout
        .take()
        .context("unpack stdout pipe missing")?;
    let mut repack = RunningChild {
        child: cli(executable, fixture, "repack", fixture, name)
            .arg("--stdin")
            .stdin(Stdio::from(stream))
            .stdout(File::create(fixture.join("repack.stdout.log"))?)
            .stderr(File::create(fixture.join("repack.stderr.log"))?)
            .spawn()
            .context("starting repack")?,
        reaped: false,
    };
    let unpack_status = unpack.wait().context("waiting for unpack")?;
    let repack_status = repack.wait().context("waiting for repack")?;
    ensure!(
        unpack_status.success() && repack_status.success(),
        "CLI pipeline failed: unpack={unpack_status}, repack={repack_status}"
    );
    Ok(())
}

fn compare_packages(game_dir: &Path, fixture: &Path, name: &str) -> Result<()> {
    let original = KFCReader::new(game_dir, name)?;
    let repacked = KFCReader::new(fixture, name)?;
    let before = original.file();
    let after = repacked.file();
    ensure!(before.format() == after.format(), "package format changed");
    ensure!(
        before.game_version() == after.game_version(),
        "game version changed"
    );
    let original_ids = before.resources().keys().iter().collect::<BTreeSet<_>>();
    let repacked_ids = after.resources().keys().iter().collect::<BTreeSet<_>>();
    ensure!(original_ids == repacked_ids, "resource ID set changed");
    ensure!(
        before.contents().len() == after.contents().len(),
        "content count changed"
    );
    for (hash, entry) in before.contents().iter() {
        let other = after
            .contents()
            .get(hash)
            .context("content hash missing after repack")?;
        ensure!(
            (entry.offset, entry.flags, entry.container_index)
                == (other.offset, other.flags, other.container_index),
            "content entry changed: {hash}"
        );
    }
    ensure!(
        before.containers().len() == after.containers().len(),
        "container count changed"
    );
    for (entry, other) in before.containers().iter().zip(after.containers()) {
        ensure!(
            (entry.size, entry.count) == (other.size, other.count),
            "container metadata changed"
        );
    }
    ensure!(
        before.resource_bundles().len() == after.resource_bundles().len(),
        "bundle count changed"
    );
    for (hash, entry) in before.resource_bundles().iter() {
        let other = after
            .resource_bundles()
            .get(hash)
            .context("resource bundle missing after repack")?;
        ensure!(
            (entry.internal_hash, entry.count) == (other.internal_hash, other.count),
            "resource bundle metadata changed: {hash:08x}"
        );
        ensure!(
            before.resources_by_type(*hash).collect::<BTreeSet<_>>()
                == after.resources_by_type(*hash).collect::<BTreeSet<_>>(),
            "resource bundle membership changed: {hash:08x}"
        );
    }

    let original_end = before
        .resource_chunks()
        .iter()
        .map(|chunk| chunk.uncompressed_offset + chunk.uncompressed_size)
        .max()
        .unwrap_or(0);
    let mut ids = before.resources().keys().iter().collect::<Vec<_>>();
    ids.sort_unstable_by_key(|id| before.resources().get(id).unwrap().offset);
    let mut original_cursor = original.new_cursor()?;
    let mut repacked_cursor = repacked.new_cursor()?;
    let mut expected = Vec::new();
    let mut actual = Vec::new();
    for id in ids {
        // Repack can log individual serialization errors without returning a
        // failing status. Verify that every ID was actually rewritten, rather
        // than silently accepting unchanged entries from the backup directory.
        ensure!(
            after.resources().get(id).unwrap().offset >= original_end,
            "resource was not repacked: {id:?}"
        );
        expected.clear();
        actual.clear();
        ensure!(
            original_cursor.read_resource_into(id, &mut expected)?,
            "original resource missing: {id:?}"
        );
        ensure!(
            repacked_cursor.read_resource_into(id, &mut actual)?,
            "repacked resource missing: {id:?}"
        );
        ensure!(
            expected == actual,
            "resource bytes changed: {id:?}, original={}, repacked={}",
            expected.len(),
            actual.len()
        );
    }
    Ok(())
}

fn roundtrip(game_dir: &Path, fixture: &Path, name: &str) -> Result<()> {
    for extension in ["kfc", "kfc_resources"] {
        let filename = format!("{name}.{extension}");
        fs::copy(game_dir.join(&filename), fixture.join(&filename))
            .with_context(|| format!("copying {filename} into the fixture"))?;
    }
    let executable = fixture.join(format!("kfc-parser{}", env::consts::EXE_SUFFIX));
    fs::copy(env!("CARGO_BIN_EXE_kfc-parser"), &executable)
        .context("copying the CLI executable")?;

    // extract-types writes beside current_exe(), so the original installation
    // and Cargo's shared reflection_data.json are never output destinations.
    let status = cli(&executable, fixture, "extract-types", game_dir, name)
        .stdin(Stdio::null())
        .stdout(File::create(fixture.join("extract-types.stdout.log"))?)
        .stderr(File::create(fixture.join("extract-types.stderr.log"))?)
        .status()
        .context("extracting matching reflection metadata")?;
    ensure!(status.success(), "extract-types failed: {status}");
    ensure!(
        fixture.join("reflection_data.json").is_file(),
        "matching reflection data was not written"
    );

    run_pipeline(&executable, fixture, game_dir, name)?;
    compare_packages(game_dir, fixture, name)
}

#[test]
#[ignore = "requires GAME_DIR and performs a full resource unpack/repack on temporary copies"]
fn unpack_repack_preserves_resource_package() -> Result<()> {
    let game_dir =
        PathBuf::from(env::var_os("GAME_DIR").context("GAME_DIR environment variable not set")?)
            .canonicalize()
            .context("resolving GAME_DIR")?;
    let name = env::var("GAME_FILE_NAME").unwrap_or_else(|_| "enshrouded".to_owned());
    let mut components = Path::new(&name).components();
    ensure!(
        matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none(),
        "GAME_FILE_NAME must be a file name without directory components"
    );
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let fixture = env::temp_dir().join(format!("kfc-unpack-repack-{}-{nonce}", std::process::id()));
    fs::create_dir(&fixture).context("creating the temporary fixture")?;

    match roundtrip(&game_dir, &fixture, &name) {
        Ok(()) => fs::remove_dir_all(&fixture).context("removing the successful temporary fixture"),
        Err(error) => Err(error.context(format!(
            "Unpack/repack verification failed. Fixture and CLI logs retained at {}",
            fixture.display()
        ))),
    }
}
